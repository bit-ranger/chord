use async_std::future::timeout;
use async_std::sync::Arc;
use async_std::task::{Builder, JoinHandle};
use chrono::Utc;
use futures::future::join_all;
use handlebars::Context;
use log::{debug, trace, warn};

use chord_common::case::{CaseAssess, CaseState};
use chord_common::error::Error;
use chord_common::flow::Flow;
use chord_common::input::DataLoad;
use chord_common::output::AssessReport;
use chord_common::rerr;
use chord_common::step::{StepRunner, StepState};
use chord_common::task::{TaskAssess, TaskId, TaskState};
use chord_common::value::{to_json, Json, Map};
use res::TaskAssessStruct;

use crate::flow::case;
use crate::flow::case::arg::CaseArgStruct;
use crate::flow::step::arg::CreateArgStruct;
use crate::flow::task::arg::TaskIdStruct;
use crate::model::app::FlowContext;
use crate::CTX_ID;

pub mod arg;
pub mod res;

pub struct Runner {
    flow_ctx: Arc<dyn FlowContext>,
    flow: Arc<Flow>,
    step_runner_vec: Arc<Vec<(String, Box<dyn StepRunner>)>>,
    id: Arc<TaskIdStruct>,
    pre_ctx: Arc<Json>,
    case_id_offset: usize,
    assess_report: Box<dyn AssessReport>,
    data_load: Box<dyn DataLoad>,
    state: TaskState,
}

impl Runner {
    pub async fn new(
        data_load: Box<dyn DataLoad>,
        assess_report: Box<dyn AssessReport>,
        flow_ctx: Arc<dyn FlowContext>,
        flow: Arc<Flow>,
        id: Arc<TaskIdStruct>,
    ) -> Result<Runner, Error> {
        let pre_ctx = pre_ctx_create(flow_ctx.clone(), flow.clone(), id.clone()).await?;
        let pre_ctx = Arc::new(pre_ctx);

        let step_runner_vec = step_runner_vec_create(
            flow_ctx.clone(),
            flow.clone(),
            pre_ctx.clone(),
            flow.case_step_id_vec()?,
            id.clone(),
        )
        .await?;
        let runner = Runner {
            assess_report,
            data_load,
            flow_ctx,
            flow,
            step_runner_vec: Arc::new(step_runner_vec),
            id,
            pre_ctx,
            case_id_offset: 1,
            state: TaskState::Ok,
        };
        Ok(runner)
    }

    pub fn id(&self) -> Arc<dyn TaskId> {
        self.id.clone()
    }

    pub async fn run(&mut self) -> Result<Box<dyn TaskAssess>, Error> {
        trace!("task start {}", self.id);
        let start = Utc::now();
        let assess = self.run0().await;
        let assess = if let Err(e) = assess {
            warn!("task Err {}", self.id);
            let assess =
                TaskAssessStruct::new(self.id.clone(), start, Utc::now(), TaskState::Err(e));
            Box::new(assess)
        } else {
            Box::new(assess.unwrap())
        };

        self.assess_report.end(assess.as_ref()).await?;
        Ok(assess)
    }

    async fn run0(&mut self) -> Result<TaskAssessStruct, Error> {
        let start = Utc::now();

        self.assess_report.start(start).await?;

        let concurrency = self.flow.ctrl_concurrency();

        self.benchmark_run(concurrency).await?;

        let task_assess = match &self.state {
            TaskState::Ok => {
                debug!("task Ok {}", self.id);
                let assess =
                    TaskAssessStruct::new(self.id.clone(), start, Utc::now(), TaskState::Ok);
                Ok(assess)
            }
            TaskState::Fail => {
                debug!("task Fail {}", self.id);
                let assess =
                    TaskAssessStruct::new(self.id.clone(), start, Utc::now(), TaskState::Fail);
                Ok(assess)
            }
            TaskState::Err(e) => Err(e.clone()),
        };
        task_assess
    }

    async fn benchmark_run(&mut self, concurrency: usize) -> Result<(), Error> {
        let duration = self.flow.ctrl_benchmark_duration();
        let brr = self.benchmark_run_round(concurrency);
        match timeout(duration, brr).await {
            Ok(r) => r?,
            Err(_) => (),
        }
        return Ok(());
    }

    async fn benchmark_run_round(&mut self, concurrency: usize) -> Result<(), Error> {
        let round_ctrl = self.flow.ctrl_benchmark_round();
        let mut round_count = 0;
        loop {
            self.data_vec_run_remaining(concurrency).await?;
            round_count += 1;
            if round_ctrl > 0 && round_count >= round_ctrl {
                break;
            }
            self.data_load.reset().await?;
        }
        return Ok(());
    }

    async fn data_vec_run_remaining(&mut self, concurrency: usize) -> Result<(), Error> {
        loop {
            let data_vec = self.data_load.load(concurrency).await?;
            let data_len = data_vec.len();
            trace!("task load data {}, {}", self.id, data_len);
            let case_assess_vec = self.data_vec_run(data_vec).await?;
            let any_fail = case_assess_vec.iter().any(|ca| !ca.state().is_ok());
            if any_fail {
                self.state = TaskState::Fail;
            }
            self.assess_report.report(&case_assess_vec).await?;
            if data_len < concurrency {
                break;
            }
        }
        Ok(())
    }

    async fn data_vec_run(&mut self, data: Vec<Json>) -> Result<Vec<Box<dyn CaseAssess>>, Error> {
        let data_size = data.len();
        let ca_vec = self.case_arg_vec(data)?;
        self.case_id_offset = self.case_id_offset + data_size;

        let mut case_assess_vec = Vec::<Box<dyn CaseAssess>>::new();
        let mut futures = vec![];
        let concurrency = self.flow.ctrl_concurrency();
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

    fn case_arg_vec<'p>(&self, data: Vec<Json>) -> Result<Vec<CaseArgStruct>, Error> {
        let vec = data
            .into_iter()
            .enumerate()
            .map(|(i, d)| {
                CaseArgStruct::new(
                    self.flow.clone(),
                    self.step_runner_vec.clone(),
                    d,
                    self.pre_ctx.clone(),
                    self.id.clone(),
                    self.case_id_offset + i,
                )
            })
            .collect();
        return Ok(vec);
    }
}

async fn pre_arg(
    flow_ctx: Arc<dyn FlowContext>,
    flow: Arc<Flow>,
    task_id: Arc<TaskIdStruct>,
) -> Result<Option<CaseArgStruct>, Error> {
    return if flow.pre_step_id_vec().is_none() {
        Ok(None)
    } else {
        let step_runner_vec = step_runner_vec_create(
            flow_ctx.clone(),
            flow.clone(),
            Arc::new(Json::Null),
            flow.pre_step_id_vec().unwrap(),
            task_id.clone(),
        )
        .await?;

        Ok(Some(CaseArgStruct::new(
            flow.clone(),
            Arc::new(step_runner_vec),
            Json::Null,
            Arc::new(Json::Null),
            task_id.clone(),
            0,
        )))
    };
}

async fn pre_ctx_create(
    flow_ctx: Arc<dyn FlowContext>,
    flow: Arc<Flow>,
    task_id: Arc<TaskIdStruct>,
) -> Result<Json, Error> {
    let pre_arg = pre_arg(flow_ctx.clone(), flow, task_id.clone()).await?;
    if pre_arg.is_none() {
        return Ok(Json::Null);
    }
    let pre_arg = pre_arg.unwrap();

    let pre_assess = case_run(flow_ctx.as_ref(), pre_arg).await;
    match pre_assess.state() {
        CaseState::Ok(pa_vec) => {
            let mut pre_ctx = Map::new();
            pre_ctx.insert("step".to_owned(), Json::Object(Map::new()));
            for pa in pa_vec {
                match pa.state() {
                    StepState::Ok(pv) => {
                        pre_ctx["step"][pa.id().step_id()]["value"] = pv.clone();
                    }
                    _ => return rerr!("012", "pre step run failure"),
                }
            }
            // debug!("task pre {} - {}", task_id, pre_ctx);
            Ok(Json::Object(pre_ctx))
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

async fn step_runner_vec_create(
    flow_ctx: Arc<dyn FlowContext>,
    flow: Arc<Flow>,
    pre_ctx: Arc<Json>,
    step_id_vec: Vec<String>,
    task_id: Arc<TaskIdStruct>,
) -> Result<Vec<(String, Box<dyn StepRunner>)>, Error> {
    let render_context = render_context_create(flow_ctx.clone(), flow.clone(), pre_ctx.clone());
    let mut step_runner_vec = vec![];
    for sid in step_id_vec {
        let pr = step_runner_create(
            flow_ctx.as_ref(),
            flow.as_ref(),
            &render_context,
            task_id.clone(),
            sid.clone(),
        )
        .await?;
        step_runner_vec.push((sid, pr));
    }
    Ok(step_runner_vec)
}

fn render_context_create(_: Arc<dyn FlowContext>, flow: Arc<Flow>, pre_ctx: Arc<Json>) -> Context {
    let mut render_data: Map = Map::new();
    let config_def = flow.def();
    match config_def {
        Some(def) => {
            render_data.insert(String::from("def"), to_json(def).unwrap());
        }
        None => {}
    }

    render_data.insert("pre".to_owned(), pre_ctx.as_ref().clone());
    return Context::wraps(render_data).unwrap();
}

async fn step_runner_create(
    flow_ctx: &dyn FlowContext,
    flow: &Flow,
    render_context: &Context,
    task_id: Arc<TaskIdStruct>,
    step_id: String,
) -> Result<Box<dyn StepRunner>, Error> {
    let kind = flow.step_kind(step_id.as_ref());
    let create_arg = CreateArgStruct::new(
        flow,
        flow_ctx.get_handlebars(),
        render_context,
        task_id,
        kind.to_owned(),
        step_id,
    );

    flow_ctx.get_step_runner_factory().create(&create_arg).await
}

async fn case_run(flow_ctx: &dyn FlowContext, case_arg: CaseArgStruct) -> Box<dyn CaseAssess> {
    Box::new(case::run(flow_ctx, case_arg).await)
}

fn case_spawn(
    flow_ctx: Arc<dyn FlowContext>,
    case_arg: CaseArgStruct,
) -> JoinHandle<Box<dyn CaseAssess>> {
    let builder = Builder::new()
        .name(format!("case_{}", case_arg.id()))
        .spawn(case_run_arc(flow_ctx, case_arg));
    return builder.unwrap();
}

async fn case_run_arc(
    flow_ctx: Arc<dyn FlowContext>,
    case_arg: CaseArgStruct,
) -> Box<dyn CaseAssess> {
    CTX_ID.with(|cid| cid.replace(case_arg.id().to_string()));
    case_run(flow_ctx.as_ref(), case_arg).await
}
