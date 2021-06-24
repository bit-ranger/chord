use async_std::future::timeout;
use async_std::sync::Arc;
use async_std::task::{Builder, JoinHandle};
use futures::future::join_all;
use log::{debug, info, trace, warn};

use chord::case::{CaseAssess, CaseState};
use chord::flow::Flow;
use chord::input::CaseLoad;
use chord::output::AssessReport;
use chord::output::Utc;
use chord::rerr;
use chord::step::{Action, StepState};
use chord::task::{TaskAssess, TaskId, TaskState};
use chord::value::{to_value, Map, Value};
use chord::Error;
use res::TaskAssessStruct;

use crate::flow::case;
use crate::flow::case::arg::CaseArgStruct;
use crate::flow::step::arg::CreateArgStruct;
use crate::flow::task::arg::TaskIdSimple;
use crate::model::app::{Context, RenderContext};
use crate::CTX_ID;

pub mod arg;
pub mod res;

pub struct TaskRunner {
    flow_ctx: Arc<dyn Context>,
    flow: Arc<Flow>,
    action_vec: Arc<Vec<(String, Box<dyn Action>)>>,
    id: Arc<TaskIdSimple>,
    pre_ctx: Arc<Value>,
    case_exec_id: Arc<String>,
    assess_report: Box<dyn AssessReport>,
    case_load: Box<dyn CaseLoad>,
    task_state: TaskState,
    stage_state: TaskState,
}

impl TaskRunner {
    pub async fn new(
        case_load: Box<dyn CaseLoad>,
        assess_report: Box<dyn AssessReport>,
        flow_ctx: Arc<dyn Context>,
        flow: Arc<Flow>,
        id: Arc<TaskIdSimple>,
    ) -> Result<TaskRunner, Error> {
        let pre_ctx = pre_ctx_create(flow_ctx.clone(), flow.clone(), id.clone()).await?;
        let pre_ctx = Arc::new(pre_ctx);

        let runner = TaskRunner {
            assess_report,
            case_load,
            flow_ctx,
            flow,
            action_vec: Arc::new(vec![]),
            id,
            pre_ctx,
            case_exec_id: Arc::new("".into()),
            task_state: TaskState::Ok,
            stage_state: TaskState::Ok,
        };
        Ok(runner)
    }

    pub fn id(&self) -> Arc<dyn TaskId> {
        self.id.clone()
    }

    pub async fn run(&mut self) -> Result<Box<dyn TaskAssess>, Error> {
        trace!("task start {}", self.id);
        let start = Utc::now();
        self.assess_report.start(start).await?;
        let result = self.start_run().await;

        let task_assess = if let Err(e) = result {
            warn!("task Err {}", self.id);
            TaskAssessStruct::new(
                self.id.clone(),
                start,
                Utc::now(),
                TaskState::Err(e.clone()),
            )
        } else {
            match &self.task_state {
                TaskState::Ok => {
                    debug!("task Ok {}", self.id);
                    TaskAssessStruct::new(self.id.clone(), start, Utc::now(), TaskState::Ok)
                }
                TaskState::Fail => {
                    info!("task Fail {}", self.id);
                    TaskAssessStruct::new(self.id.clone(), start, Utc::now(), TaskState::Fail)
                }
                TaskState::Err(e) => {
                    warn!("task Err {}", self.id);
                    TaskAssessStruct::new(
                        self.id.clone(),
                        start,
                        Utc::now(),
                        TaskState::Err(e.clone()),
                    )
                }
            }
        };

        self.assess_report.end(&task_assess).await?;
        Ok(Box::new(task_assess))
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
            if let TaskState::Fail = self.stage_state {
                if "stage_fail" == self.flow.stage_break_on(state_id.as_str()) {
                    break;
                }
            }
        }
        Ok(())
    }

    async fn stage_run(&mut self, stage_id: &str) -> Result<(), Error> {
        self.stage_state = TaskState::Ok;
        let step_id_vec: Vec<String> = self
            .flow
            .stage_step_id_vec(stage_id)
            .into_iter()
            .map(|s| s.to_owned())
            .collect();
        let action_vec = action_vec_create(
            self.flow_ctx.clone(),
            self.flow.clone(),
            self.pre_ctx.clone(),
            step_id_vec,
            self.id.clone(),
        )
        .await?;
        self.action_vec = Arc::new(action_vec);

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
            self.case_exec_id = Arc::new(format!("{}_{}", stage_id, round_count + 1));
            self.stage_data_vec_run_remaining(stage_id, concurrency)
                .await?;
            self.case_load.reset().await?;
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
        loop {
            let case_data_vec: Vec<(String, Value)> =
                self.stage_data_vec_load(stage_id, concurrency).await?;

            if case_data_vec.len() == 0 {
                return Ok(());
            }

            trace!("task load data {}, {}", self.id, case_data_vec.len());

            let case_assess_vec = self.case_data_vec_run(case_data_vec, concurrency).await?;
            let any_fail = case_assess_vec.iter().any(|ca| !ca.state().is_ok());
            if any_fail {
                self.stage_state = TaskState::Fail;
                self.task_state = TaskState::Fail;
            }
            self.assess_report
                .report(stage_id, &case_assess_vec)
                .await?;
        }
    }

    async fn stage_data_vec_load(
        &mut self,
        stage_id: &str,
        size: usize,
    ) -> Result<Vec<(String, Value)>, Error> {
        let case_data_vec: Vec<(String, Value)> = match self.flow.stage_case_filter(stage_id) {
            Some(filter) => {
                let mut ccdv: Vec<(String, Value)> = vec![];
                loop {
                    let cdv = self.case_load.load(size - ccdv.len()).await?;
                    if cdv.len() == 0 {
                        break;
                    }

                    for (cid, cd) in cdv {
                        let mut ctx =
                            render_context_create(self.flow.clone(), self.pre_ctx.clone());

                        if let Value::Object(d) = ctx.data_mut() {
                            d.insert("case".into(), cd.clone());
                            let filter_ok =
                                crate::flow::assert(self.flow_ctx.get_handlebars(), &ctx, filter)
                                    .await;
                            if filter_ok {
                                ccdv.push((cid, cd));
                            }
                        }
                    }

                    if ccdv.len() >= size {
                        break;
                    }
                }
                ccdv
            }
            None => self.case_load.load(size).await?,
        };

        return Ok(case_data_vec);
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
                    self.action_vec.clone(),
                    d,
                    self.pre_ctx.clone(),
                    self.id.clone(),
                    id,
                    self.case_exec_id.clone(),
                )
            })
            .collect();
        return Ok(vec);
    }
}

async fn pre_arg(
    flow_ctx: Arc<dyn Context>,
    flow: Arc<Flow>,
    task_id: Arc<TaskIdSimple>,
) -> Result<Option<CaseArgStruct>, Error> {
    return if flow.pre_step_id_vec().is_none() {
        Ok(None)
    } else {
        let action_vec = action_vec_create(
            flow_ctx.clone(),
            flow.clone(),
            Arc::new(Value::Null),
            flow.pre_step_id_vec()
                .unwrap()
                .into_iter()
                .map(|s| s.to_owned())
                .collect(),
            task_id.clone(),
        )
        .await?;

        Ok(Some(CaseArgStruct::new(
            flow.clone(),
            Arc::new(action_vec),
            Value::Null,
            Arc::new(Value::Null),
            task_id.clone(),
            "pre".into(),
            Arc::new("pre".into()),
        )))
    };
}

async fn pre_ctx_create(
    flow_ctx: Arc<dyn Context>,
    flow: Arc<Flow>,
    task_id: Arc<TaskIdSimple>,
) -> Result<Value, Error> {
    let pre_arg = pre_arg(flow_ctx.clone(), flow, task_id.clone()).await?;
    if pre_arg.is_none() {
        return Ok(Value::Null);
    }
    let pre_arg = pre_arg.unwrap();

    let pre_assess = case_run(flow_ctx.as_ref(), pre_arg).await;
    match pre_assess.state() {
        CaseState::Ok(pa_vec) => {
            let mut pre_ctx = Map::new();
            pre_ctx.insert("step".to_owned(), Value::Object(Map::new()));
            for pa in pa_vec {
                match pa.state() {
                    StepState::Ok(pv) => {
                        pre_ctx["step"][pa.id().step_id()]["value"] = pv.clone();
                    }
                    _ => return rerr!("012", "pre step run failure"),
                }
            }
            // debug!("task pre {} - {}", task_id, pre_ctx);
            Ok(Value::Object(pre_ctx))
        }
        CaseState::Fail(pa_vec) => {
            let pa_last = pa_vec.last().unwrap();
            rerr!("020", format!("pre Fail : {}", pa_last.id()))
        }
        CaseState::Err(e) => {
            rerr!("021", format!("pre Err  : {}", e.to_string()))
        }
    }
}

async fn action_vec_create(
    flow_ctx: Arc<dyn Context>,
    flow: Arc<Flow>,
    pre_ctx: Arc<Value>,
    step_id_vec: Vec<String>,
    task_id: Arc<TaskIdSimple>,
) -> Result<Vec<(String, Box<dyn Action>)>, Error> {
    let render_context = render_context_create(flow.clone(), pre_ctx.clone());
    let mut action_vec = vec![];
    for sid in step_id_vec {
        let pr = action_create(
            flow_ctx.as_ref(),
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

fn render_context_create(flow: Arc<Flow>, pre_ctx: Arc<Value>) -> RenderContext {
    let mut render_data: Map = Map::new();
    let config_def = flow.def();
    match config_def {
        Some(def) => {
            render_data.insert(String::from("def"), to_value(def).unwrap());
        }
        None => {}
    }

    render_data.insert("pre".to_owned(), pre_ctx.as_ref().clone());
    return RenderContext::wraps(render_data).unwrap();
}

async fn action_create(
    flow_ctx: &dyn Context,
    flow: &Flow,
    render_context: &RenderContext,
    task_id: Arc<TaskIdSimple>,
    step_id: String,
) -> Result<Box<dyn Action>, Error> {
    let action = flow.step_action(step_id.as_ref());
    let create_arg = CreateArgStruct::new(
        flow,
        flow_ctx.get_handlebars(),
        render_context,
        task_id,
        action.into(),
        step_id,
    );

    flow_ctx.get_action_factory().create(&create_arg).await
}

async fn case_run(flow_ctx: &dyn Context, case_arg: CaseArgStruct) -> Box<dyn CaseAssess> {
    Box::new(case::run(flow_ctx, case_arg).await)
}

fn case_spawn(
    flow_ctx: Arc<dyn Context>,
    case_arg: CaseArgStruct,
) -> JoinHandle<Box<dyn CaseAssess>> {
    let builder = Builder::new()
        .name(format!("case_{}", case_arg.id()))
        .spawn(case_run_arc(flow_ctx, case_arg));
    return builder.unwrap();
}

async fn case_run_arc(flow_ctx: Arc<dyn Context>, case_arg: CaseArgStruct) -> Box<dyn CaseAssess> {
    CTX_ID.with(|cid| cid.replace(case_arg.id().to_string()));
    case_run(flow_ctx.as_ref(), case_arg).await
}
