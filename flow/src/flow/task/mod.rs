use async_std::future::timeout;
use async_std::sync::Arc;
use async_std::task::{Builder, JoinHandle};
use futures::future::join_all;
use log::{error, info, trace, warn};

use chord::action::Action;
use chord::case::{CaseAssess, CaseState};
use chord::collection::TailDropVec;
use chord::flow::Flow;
use chord::input::CaseStore;
use chord::output::Report;
use chord::output::Utc;
use chord::step::StepState;
use chord::task::{TaskAssess, TaskId, TaskState};
use chord::value::{json, Map, Value};
use chord::{err, Error};
use res::TaskAssessStruct;

use crate::flow::case;
use crate::flow::case::arg::CaseArgStruct;
use crate::flow::render_assign_object;
use crate::flow::step::arg::CreateArgStruct;
use crate::flow::task::arg::TaskIdSimple;
use crate::flow::task::Error::{ActionCreate, DefRender};
use crate::model::app::{FlowApp, RenderContext};
use crate::CTX_ID;

pub mod arg;
pub mod res;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("def render error: {0}")]
    DefRender(Box<dyn std::error::Error>),

    #[error("action create error: {0}")]
    ActionCreate(Box<dyn std::error::Error>),
}

pub struct TaskRunner {
    step_vec: Arc<TailDropVec<(String, Box<dyn Action>)>>,
    stage_round_no: usize,
    stage_id: Arc<String>,
    stage_state: TaskState,

    pre_ctx: Option<Arc<Map>>,
    #[allow(dead_code)]
    pre_assess: Option<Box<dyn CaseAssess>>,
    #[allow(dead_code)]
    pre_step_vec: Option<Arc<TailDropVec<(String, Box<dyn Action>)>>>,

    task_state: TaskState,

    def_ctx: Option<Arc<Map>>,
    assess_report: Box<dyn Report>,
    case_factory: Box<dyn CaseStore>,
    id: Arc<TaskIdSimple>,
    flow_ctx: Arc<dyn FlowApp>,
    flow: Arc<Flow>,
}

impl TaskRunner {
    pub async fn new(
        case_factory: Box<dyn CaseStore>,
        assess_report: Box<dyn Report>,
        flow_app: Arc<dyn FlowApp>,
        flow: Arc<Flow>,
        id: Arc<TaskIdSimple>,
    ) -> Result<TaskRunner, Error> {
        let def_ctx = if let Some(def_raw) = flow.def() {
            let rc: Value = json!({
               "__meta__": flow.meta()
            });
            let rc = RenderContext::wraps(rc).unwrap();
            let rso = render_assign_object(flow_app.get_handlebars(), &rc, def_raw, false)
                .map_err(|e| DefRender(Box::new(e)))?;
            Some(Arc::new(rso))
        } else {
            None
        };

        let pre_step_vec = match flow.pre_step_id_vec() {
            Some(pre_ste_id_vec) => step_vec_create(
                flow_app.clone(),
                flow.clone(),
                def_ctx.clone(),
                None,
                pre_ste_id_vec.into_iter().map(|s| s.to_owned()).collect(),
                id.clone(),
            )
            .await
            .map_err(|e| ActionCreate(Box::new(e)))?,
            None => vec![],
        };
        let pre_step_vec = Arc::new(TailDropVec::from(pre_step_vec));

        return if pre_step_vec.is_empty() {
            let runner = TaskRunner {
                step_vec: Arc::new(TailDropVec::from(vec![])),
                stage_id: Arc::new("".into()),
                stage_round_no: 0,

                stage_state: TaskState::Ok,

                pre_ctx: None,
                pre_assess: None,
                pre_step_vec: None,

                task_state: TaskState::Ok,

                def_ctx,
                assess_report,
                case_factory,
                id,
                flow_ctx: flow_app,
                flow,
            };
            Ok(runner)
        } else {
            let pre_arg = pre_arg(
                flow.clone(),
                id.clone(),
                def_ctx.clone(),
                pre_step_vec.clone(),
            )
            .await?;
            let pre_assess = case_run(flow_app.as_ref(), pre_arg).await;
            let pre_ctx = pre_ctx_create(pre_assess.as_ref()).await?;
            let runner = TaskRunner {
                step_vec: Arc::new(TailDropVec::from(vec![])),
                stage_id: Arc::new("".into()),
                stage_round_no: 0,

                stage_state: TaskState::Ok,

                pre_ctx: Some(Arc::new(pre_ctx)),
                pre_assess: Some(pre_assess),
                pre_step_vec: Some(pre_step_vec),

                task_state: TaskState::Ok,

                def_ctx,
                assess_report,
                case_factory,
                id,
                flow_ctx: flow_app,
                flow,
            };
            Ok(runner)
        };
    }

    pub fn id(&self) -> Arc<dyn TaskId> {
        self.id.clone()
    }

    pub async fn run(mut self) -> Box<dyn TaskAssess> {
        trace!("task run {}", self.id);
        let start = Utc::now();
        if let Err(e) = self.assess_report.start(start, self.flow.clone()).await {
            error!("task Err {}", self.id);
            return Box::new(TaskAssessStruct::new(
                self.id,
                start,
                Utc::now(),
                TaskState::Err(e),
            ));
        }
        let result = self.start_run().await;

        let task_assess = if let Err(e) = result {
            error!("task Err {}", self.id);
            TaskAssessStruct::new(
                self.id.clone(),
                start,
                Utc::now(),
                TaskState::Err(e.clone()),
            )
        } else {
            match self.task_state {
                TaskState::Ok => {
                    info!("task Ok {}", self.id.clone());
                    TaskAssessStruct::new(self.id.clone(), start, Utc::now(), TaskState::Ok)
                }
                TaskState::Fail(c) => {
                    warn!("task Fail {}", self.id);
                    TaskAssessStruct::new(
                        self.id.clone(),
                        start,
                        Utc::now(),
                        TaskState::Fail(c.clone()),
                    )
                }
                TaskState::Err(e) => {
                    error!("task Err {}", self.id);
                    TaskAssessStruct::new(self.id.clone(), start, Utc::now(), TaskState::Err(e))
                }
            }
        };

        if let Err(e) = self
            .assess_report
            .end(&task_assess)
            .await
            .map_err(|e| Error::Report(e))?
        {
            error!("task Err {}", self.id);
            return Box::new(TaskAssessStruct::new(
                self.id,
                start,
                Utc::now(),
                TaskState::Err(e),
            ));
        }

        Box::new(task_assess)
    }

    async fn start_run(&mut self) -> Result<(), Error> {
        let stage_id_vec: Vec<String> = self
            .flow
            .stage_id_vec()
            .into_iter()
            .map(|s| s.to_owned())
            .collect();
        for state_id in stage_id_vec {
            trace!("task stage {}, {}", self.id, state_id);
            self.stage_run(state_id.as_str()).await?;
            if let TaskState::Fail(_) = self.stage_state {
                if "stage_fail" == self.flow.stage_break_on(state_id.as_str()) {
                    break;
                }
            }
        }
        Ok(())
    }

    async fn stage_run(&mut self, stage_id: &str) -> Result<(), Error> {
        self.stage_id = Arc::new(stage_id.to_string());
        self.stage_state = TaskState::Ok;
        let step_id_vec: Vec<String> = self
            .flow
            .stage_step_id_vec(stage_id)
            .into_iter()
            .map(|s| s.to_owned())
            .collect();
        let action_vec = step_vec_create(
            self.flow_ctx.clone(),
            self.flow.clone(),
            self.def_ctx.clone(),
            self.pre_ctx.clone(),
            step_id_vec,
            self.id.clone(),
        )
        .await?;
        self.step_vec = Arc::new(TailDropVec::from(action_vec));

        let duration = self.flow.stage_duration(stage_id);
        let srr = self.stage_round_run(stage_id);
        match timeout(duration, srr).await {
            Ok(r) => r?,
            Err(_) => (),
        }
        return Ok(());
    }

    async fn stage_round_run(&mut self, stage_id: &str) -> Result<(), Error> {
        let concurrency = self.flow.stage_concurrency(stage_id);
        let round_max = self.flow.stage_round(stage_id);
        let mut round_count = 0;
        loop {
            self.stage_round_no = round_count;
            self.stage_data_vec_run_remaining(stage_id, concurrency)
                .await?;
            round_count += 1;
            if round_count >= round_max {
                break;
            }
        }
        return Ok(());
    }

    async fn stage_data_vec_run_remaining(
        &mut self,
        stage_id: &str,
        concurrency: usize,
    ) -> Result<(), Error> {
        let case_name = self.flow.stage_case_name(stage_id);
        let mut data_load = self
            .case_factory
            .create(case_name)
            .await
            .map_err(|_| err!("013", format!("stage {} failed to load case", stage_id)))?;
        let mut load_times = 0;
        loop {
            let case_data_vec: Vec<(String, Value)> = data_load.load(concurrency).await?;
            load_times = load_times + 1;
            if case_data_vec.len() == 0 {
                if load_times == 1 {
                    return Err(err!("011", format!("stage {} no case provided", stage_id)));
                } else {
                    trace!("task exhaust data {}, {}", self.id, stage_id);
                    return Ok(());
                }
            }

            trace!(
                "task load data {}, {}, {}",
                self.id,
                stage_id,
                case_data_vec.len()
            );

            let case_assess_vec = self.case_data_vec_run(case_data_vec, concurrency).await?;
            let first_fail = case_assess_vec.iter().find(|ca| !ca.state().is_ok());
            if first_fail.is_some() {
                let cause_case = first_fail.unwrap();
                let cause = match cause_case.state() {
                    CaseState::Err(_) => cause_case.id().to_string(),
                    CaseState::Fail(v) => v
                        .last()
                        .map(|s| s.id().to_string())
                        .or_else(|| Some(String::new()))
                        .unwrap(),
                    CaseState::Ok(_) => String::new(),
                };
                self.stage_state = TaskState::Fail(cause.clone());
                self.task_state = TaskState::Fail(cause);
            }
            self.assess_report.report(&case_assess_vec).await?;
        }
    }

    async fn case_data_vec_run(
        &mut self,
        case_vec: Vec<(String, Value)>,
        concurrency: usize,
    ) -> Result<Vec<Box<dyn CaseAssess>>, Error> {
        let ca_vec = self.case_arg_vec(case_vec)?;

        let mut case_assess_vec = Vec::<Box<dyn CaseAssess>>::new();
        let mut futures = vec![];
        for ca in ca_vec {
            let f = case_spawn(self.flow_ctx.clone(), ca);
            futures.push(f);
            if futures.len() >= concurrency {
                let case_assess = join_all(futures.split_off(0)).await;
                case_assess_vec.extend(case_assess);
            }
        }
        if !futures.is_empty() {
            let case_assess = join_all(futures).await;
            case_assess_vec.extend(case_assess);
        }
        Ok(case_assess_vec)
    }

    fn case_arg_vec<'p>(&self, data: Vec<(String, Value)>) -> Result<Vec<CaseArgStruct>, Error> {
        let vec = data
            .into_iter()
            .map(|(id, d)| {
                CaseArgStruct::new(
                    self.flow.clone(),
                    self.step_vec.clone(),
                    d,
                    self.pre_ctx.clone(),
                    self.def_ctx.clone(),
                    self.id.clone(),
                    self.stage_id.clone(),
                    Arc::new(self.stage_round_no.to_string()),
                    id,
                )
            })
            .collect();
        return Ok(vec);
    }
}

async fn pre_arg(
    flow: Arc<Flow>,
    task_id: Arc<TaskIdSimple>,
    def_ctx: Option<Arc<Map>>,
    pre_action_vec: Arc<TailDropVec<(String, Box<dyn Action>)>>,
) -> Result<CaseArgStruct, Error> {
    Ok(CaseArgStruct::new(
        flow.clone(),
        pre_action_vec,
        Value::Null,
        None,
        def_ctx,
        task_id.clone(),
        Arc::new("pre".into()),
        Arc::new("1".into()),
        "1".into(),
    ))
}

async fn pre_ctx_create(pre_assess: &dyn CaseAssess) -> Result<Map, Error> {
    match pre_assess.state() {
        CaseState::Ok(pa_vec) => {
            let mut pre_ctx = Map::new();
            pre_ctx.insert("step".to_owned(), Value::Object(Map::new()));
            for pa in pa_vec.iter() {
                match pa.state() {
                    StepState::Ok(pv) => {
                        pre_ctx["step"][pa.id().step()]["value"] = pv.as_value().clone();
                    }
                    _ => return Err(err!("012", "pre step run failure")),
                }
            }
            Ok(pre_ctx)
        }
        CaseState::Fail(pa_vec) => {
            let pa_last = pa_vec.last().unwrap();
            Err(err!("020", format!("pre Fail : {}", pa_last.id())))
        }
        CaseState::Err(e) => Err(err!("021", format!("pre Err  : {}", e.to_string()))),
    }
}

async fn step_vec_create(
    flow_app: Arc<dyn FlowApp>,
    flow: Arc<Flow>,
    def_ctx: Option<Arc<Map>>,
    pre_ctx: Option<Arc<Map>>,
    step_id_vec: Vec<String>,
    task_id: Arc<TaskIdSimple>,
) -> Result<Vec<(String, Box<dyn Action>)>, Error> {
    let render_context = render_context_create(flow.clone(), def_ctx, pre_ctx);
    let mut action_vec = vec![];
    for sid in step_id_vec {
        let pr = step_create(
            flow_app.as_ref(),
            flow.as_ref(),
            &render_context,
            task_id.clone(),
            sid.clone(),
        )
        .await?;
        action_vec.push((sid, pr));
    }
    Ok(action_vec)
}

fn render_context_create(
    flow: Arc<Flow>,
    def_ctx: Option<Arc<Map>>,
    pre_ctx: Option<Arc<Map>>,
) -> RenderContext {
    let mut render_data: Map = Map::new();

    render_data.insert("__meta__".to_owned(), Value::Object(flow.meta().clone()));
    if let Some(def_ctx) = def_ctx {
        render_data.insert("def".to_owned(), Value::Object(def_ctx.as_ref().clone()));
    }

    if let Some(pre_ctx) = pre_ctx {
        render_data.insert("pre".to_owned(), Value::Object(pre_ctx.as_ref().clone()));
    }

    return RenderContext::wraps(render_data).unwrap();
}

async fn step_create(
    flow_app: &dyn FlowApp,
    flow: &Flow,
    render_ctx: &RenderContext,
    task_id: Arc<TaskIdSimple>,
    step_id: String,
) -> Result<Box<dyn Action>, Error> {
    let let_raw = flow.step_let(step_id.as_ref());
    let let_value = match let_raw {
        Some(let_raw) => {
            let let_value =
                render_assign_object(flow_app.get_handlebars(), render_ctx, let_raw, true)?;
            Some(let_value)
        }
        None => None,
    };

    let action = flow.step_exec_action(step_id.as_ref());
    let create_arg = CreateArgStruct::new(
        flow,
        flow_app.get_handlebars(),
        let_value,
        task_id,
        action.into(),
        step_id,
    )?;

    flow_app.get_action_factory().create(&create_arg).await
}

async fn case_run(flow_ctx: &dyn FlowApp, case_arg: CaseArgStruct) -> Box<dyn CaseAssess> {
    Box::new(case::run(flow_ctx, case_arg).await)
}

fn case_spawn(
    flow_ctx: Arc<dyn FlowApp>,
    case_arg: CaseArgStruct,
) -> JoinHandle<Box<dyn CaseAssess>> {
    let builder = Builder::new()
        .name(format!("case_{}", case_arg.id()))
        .spawn(case_run_arc(flow_ctx, case_arg));
    return builder.unwrap();
}

async fn case_run_arc(flow_ctx: Arc<dyn FlowApp>, case_arg: CaseArgStruct) -> Box<dyn CaseAssess> {
    CTX_ID.with(|cid| cid.replace(case_arg.id().to_string()));
    case_run(flow_ctx.as_ref(), case_arg).await
}
