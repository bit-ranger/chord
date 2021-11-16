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
use chord::step::{StepAssess, StepState};
use chord::task::{TaskAssess, TaskId, TaskState};
use chord::value::{json, Map, Value};
use res::TaskAssessStruct;

use crate::flow::case;
use crate::flow::case::arg::CaseArgStruct;
use crate::flow::render_assign_object;
use crate::flow::step::arg::CreateArgStruct;
use crate::flow::task::arg::TaskIdSimple;
use crate::flow::task::Error::*;
use crate::model::app::{FlowApp, RenderContext};
use crate::CTX_ID;
use handlebars::TemplateRenderError;

pub mod arg;
pub mod res;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("{0} render error: {1}")]
    Render(String, TemplateRenderError),

    #[error("step create error: {0}")]
    StepCreate(String, Box<dyn std::error::Error + Sync + Send>),

    #[error("pre run error: {0}")]
    PreErr(String),

    #[error("pre run fail at step: {0}")]
    PreFail(String),

    #[error("report assess error: {0}")]
    Report(Box<dyn std::error::Error + Sync + Send>),

    #[error("load case error in stage {0}: {1}")]
    Load(String, Box<dyn std::error::Error + Sync + Send>),

    #[error("case empty in stage {0}")]
    CaseEmpty(String),
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
    case_store: Box<dyn CaseStore>,
    id: Arc<TaskIdSimple>,
    flow_app: Arc<dyn FlowApp>,
    flow: Arc<Flow>,
}

impl TaskRunner {
    pub fn new(
        case_store: Box<dyn CaseStore>,
        assess_report: Box<dyn Report>,
        flow_app: Arc<dyn FlowApp>,
        flow: Arc<Flow>,
        id: Arc<TaskIdSimple>,
    ) -> TaskRunner {
        let runner = TaskRunner {
            step_vec: Arc::new(TailDropVec::from(vec![])),
            stage_id: Arc::new("".into()),
            stage_round_no: 0,

            stage_state: TaskState::Ok,

            pre_ctx: None,
            pre_assess: None,
            pre_step_vec: None,

            task_state: TaskState::Ok,

            def_ctx: None,
            assess_report,
            case_store,
            id,
            flow_app,
            flow,
        };

        runner
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
                TaskState::Err(Box::new(Report(e))),
            ));
        }

        if let Some(def_raw) = self.flow.def() {
            let rc: Value = json!({
               "__meta__": self.flow.meta()
            });
            let rc = RenderContext::wraps(rc).unwrap();
            let rso = render_assign_object(self.flow_app.get_handlebars(), &rc, def_raw, false);
            if let Err(e) = rso {
                error!("task Err {}", self.id);
                return Box::new(TaskAssessStruct::new(
                    self.id.clone(),
                    start,
                    Utc::now(),
                    TaskState::Err(Box::new(Render("def".to_string(), e))),
                ));
            } else {
                self.def_ctx = Some(Arc::new(rso.unwrap()));
            }
        }

        if let Some(pre_ste_id_vec) = self.flow.pre_step_id_vec() {
            let pre_step_vec = step_vec_create(
                self.flow_app.clone(),
                self.flow.clone(),
                self.def_ctx.clone(),
                None,
                pre_ste_id_vec.into_iter().map(|s| s.to_owned()).collect(),
                self.id.clone(),
            )
            .await;
            if let Err(e) = pre_step_vec {
                error!("task Err {}", self.id);
                return Box::new(TaskAssessStruct::new(
                    self.id.clone(),
                    start,
                    Utc::now(),
                    TaskState::Err(Box::new(e)),
                ));
            }

            let pre_action_vec = Arc::new(TailDropVec::from(pre_step_vec.unwrap()));
            let pre_arg = pre_arg(
                self.flow.clone(),
                self.id.clone(),
                self.def_ctx.clone(),
                pre_action_vec.clone(),
            )
            .await;
            if let Err(e) = pre_arg {
                error!("task Err {}", self.id);
                return Box::new(TaskAssessStruct::new(
                    self.id,
                    start,
                    Utc::now(),
                    TaskState::Err(Box::new(e)),
                ));
            }

            let pre_assess = case_run(self.flow_app.as_ref(), pre_arg.unwrap()).await;

            match pre_assess.state() {
                CaseState::Err(e) => {
                    error!("task Err {}", self.id);
                    return Box::new(TaskAssessStruct::new(
                        self.id,
                        start,
                        Utc::now(),
                        TaskState::Err(Box::new(PreErr(e.to_string()))),
                    ));
                }

                CaseState::Fail(v) => {
                    error!("task Err {}", self.id);
                    return Box::new(TaskAssessStruct::new(
                        self.id,
                        start,
                        Utc::now(),
                        TaskState::Err(Box::new(PreFail(
                            v.last().unwrap().id().step().to_string(),
                        ))),
                    ));
                }
                CaseState::Ok(sa_vec) => {
                    let pre_ctx = pre_ctx_create(sa_vec.as_ref()).await;
                    self.pre_ctx = Some(Arc::new(pre_ctx));
                    self.pre_assess = Some(pre_assess);
                    self.pre_step_vec = Some(pre_action_vec);
                }
            }
        };

        let result = self.start_run().await;

        let task_assess = if let Err(e) = result {
            error!("task Err {}", self.id);
            TaskAssessStruct::new(
                self.id.clone(),
                start,
                Utc::now(),
                TaskState::Err(Box::new(e)),
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

        if let Err(e) = self.assess_report.end(&task_assess).await {
            error!("task Err {}", self.id);
            return Box::new(TaskAssessStruct::new(
                self.id,
                start,
                Utc::now(),
                TaskState::Err(Box::new(Report(e))),
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
            self.flow_app.clone(),
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
            .case_store
            .create(case_name)
            .await
            .map_err(|e| Load(stage_id.to_string(), e))?;
        let mut load_times = 0;
        loop {
            let case_data_vec: Vec<(String, Value)> = data_load
                .load(concurrency)
                .await
                .map_err(|e| Load(stage_id.to_string(), e))?;
            load_times = load_times + 1;
            if case_data_vec.len() == 0 {
                return if load_times == 1 {
                    Err(CaseEmpty(stage_id.to_string()))
                } else {
                    trace!("task exhaust data {}, {}", self.id, stage_id);
                    Ok(())
                };
            }

            trace!(
                "task load data {}, {}, {}",
                self.id,
                stage_id,
                case_data_vec.len()
            );

            let case_assess_vec = self.case_data_vec_run(case_data_vec, concurrency).await;
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
            self.assess_report
                .report(&case_assess_vec)
                .await
                .map_err(|e| Report(e))?;
        }
    }

    async fn case_data_vec_run(
        &mut self,
        case_vec: Vec<(String, Value)>,
        concurrency: usize,
    ) -> Vec<Box<dyn CaseAssess>> {
        let ca_vec = self.case_arg_vec(case_vec);

        let mut case_assess_vec = Vec::<Box<dyn CaseAssess>>::new();
        let mut futures = vec![];
        for ca in ca_vec {
            let f = case_spawn(self.flow_app.clone(), ca);
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
        case_assess_vec
    }

    fn case_arg_vec<'p>(&self, data: Vec<(String, Value)>) -> Vec<CaseArgStruct> {
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
        return vec;
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

async fn pre_ctx_create(sa_vec: &Vec<Box<dyn StepAssess>>) -> Map {
    let mut pre_ctx = Map::new();
    pre_ctx.insert("step".to_owned(), Value::Object(Map::new()));
    for sa in sa_vec.iter() {
        if let StepState::Ok(pv) = sa.state() {
            pre_ctx["step"][sa.id().step()]["value"] = pv.as_value().clone();
        }
    }
    pre_ctx
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
                render_assign_object(flow_app.get_handlebars(), render_ctx, let_raw, true)
                    .map_err(|e| Render(format!("step.{}.let", step_id), e))?;
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
        step_id.clone(),
    );

    flow_app
        .get_action_factory()
        .create(&create_arg)
        .await
        .map_err(|e| StepCreate(step_id, e))
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
