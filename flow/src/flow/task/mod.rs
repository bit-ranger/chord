use std::cell::RefCell;

use async_std::sync::Arc;
use async_std::task::{Builder, JoinHandle};
use async_std::task_local;
use chrono::Utc;
use futures::future::join_all;
use handlebars::Context;
use log::{debug, trace, warn};

use chord_common::case::{CaseAssess, CaseState};
use chord_common::error::Error;
use chord_common::flow::Flow;
use chord_common::point::{PointRunner, PointState};
use chord_common::rerr;
use chord_common::task::{TaskAssess, TaskState, TaskId};
use chord_common::value::{Json, Map, to_json};
use res::TaskAssessStruct;

use crate::CASE_ID;
use crate::flow::case;
use crate::flow::case::arg::CaseArgStruct;
use crate::flow::point::arg::CreateArgStruct;
use crate::model::app::FlowContext;
use crate::flow::task::arg::TaskIdStruct;

pub mod res;
pub mod arg;

task_local! {
    pub static TASK_ID: RefCell<String> = RefCell::new(String::new());
}


pub struct Runner {
    flow_ctx: Arc<dyn FlowContext>,
    flow: Arc<Flow>,
    point_runner_vec: Arc<Vec<(String, Box<dyn PointRunner>)>>,
    id: Arc<TaskIdStruct>,
    pre_ctx: Arc<Vec<(String, Json)>>,
    case_id_offset: usize,
}

impl Runner {


    pub async fn new(
        flow_ctx: Arc<dyn FlowContext>,
        flow: Arc<Flow>,
        id: Arc<TaskIdStruct>,
    ) -> Result<Runner, Error> {
        let pre_ctx =
            pre_ctx_create(flow_ctx.clone(), flow.clone(), id.clone()).await?;
        let pre_ctx = Arc::new(pre_ctx);

        let point_runner_vec = point_runner_vec_create(
            flow_ctx.clone(),
            flow.clone(),
            pre_ctx.clone(),
            flow.case_point_id_vec()?,
            id.clone(),
        ).await?;
        let runner = Runner {
            flow_ctx,
            flow,
            point_runner_vec: Arc::new(point_runner_vec),
            id,
            pre_ctx,
            case_id_offset: 1,
        };
        Ok(runner)
    }

    pub fn id(&self) -> Arc<dyn TaskId>{
        self.id.clone()
    }

    pub async fn run(&mut self, data: Vec<Json>) -> Box<dyn TaskAssess> {
        Box::new(self.run0(data).await)
    }

    async fn run0(&mut self, data: Vec<Json>) -> TaskAssessStruct {
        let data_len = data.len();
        trace!("task start {}", self.id);
        let start = Utc::now();

        let ca_vec = match self.case_arg_vec(data) {
            Ok(v) => v,
            Err(e) => {
                warn!("task Err {}", self.id);
                return TaskAssessStruct::new(
                    self.id.clone(),
                    start,
                    Utc::now(),
                    TaskState::Err(e),
                );
            }
        };
        self.case_id_offset = self.case_id_offset + data_len;
        trace!("task load data {}, {}", self.id, ca_vec.len());

        let mut case_assess_vec = Vec::<Box<dyn CaseAssess>>::new();
        let limit_concurrency = self.flow.limit_concurrency();
        let mut futures = vec![];
        for ca in ca_vec {
            let f = case_spawn(self.flow_ctx.clone(), ca);
            futures.push(f);
            if futures.len() >= limit_concurrency {
                let case_assess = join_all(futures.split_off(0)).await;
                case_assess_vec.extend(case_assess);
            }
        }
        if !futures.is_empty() {
            let case_assess = join_all(futures).await;
            case_assess_vec.extend(case_assess);
        }

        let any_fail = case_assess_vec.iter().any(|ca| !ca.state().is_ok());

        return if any_fail {
            warn!("task Fail {}", self.id);
            TaskAssessStruct::new(
                self.id.clone(),
                start,
                Utc::now(),
                TaskState::Fail(case_assess_vec),
            )
        } else {
            debug!("task Ok {}", self.id);
            TaskAssessStruct::new(
                self.id.clone(),
                start,
                Utc::now(),
                TaskState::Ok(case_assess_vec),
            )
        };
    }

    pub fn case_arg_vec<'p>(&self, data: Vec<Json>) -> Result<Vec<CaseArgStruct>, Error> {
        let vec = data
            .into_iter()
            .enumerate()
            .map(|(i, d)| {
                CaseArgStruct::new(
                    self.flow.clone(),
                    self.point_runner_vec.clone(),
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
    return if flow.pre_point_id_vec().is_none() {
        Ok(None)
    } else {
        let point_runner_vec = point_runner_vec_create(
            flow_ctx.clone(),
            flow.clone(),
            Arc::new(Vec::new()),
            flow.pre_point_id_vec().unwrap(),
            task_id.clone(),
        )
            .await?;

        Ok(Some(CaseArgStruct::new(
            flow.clone(),
            Arc::new(point_runner_vec),
            Json::Null,
            Arc::new(Vec::new()),
            task_id.clone(),
            0,
        )))
    };
}

async fn pre_ctx_create(
    flow_ctx: Arc<dyn FlowContext>,
    flow: Arc<Flow>,
    task_id: Arc<TaskIdStruct>,
) -> Result<Vec<(String, Json)>, Error> {
    let mut case_ctx = vec![];
    let pre_arg = pre_arg(flow_ctx.clone(), flow, task_id.clone()).await?;
    if pre_arg.is_none() {
        return Ok(case_ctx);
    }
    let pre_arg = pre_arg.unwrap();

    let pre_assess = case_run(flow_ctx.as_ref(), pre_arg).await;
    match pre_assess.state() {
        CaseState::Ok(pa_vec) => {
            let mut pre_ctx = Map::new();
            for pa in pa_vec {
                match pa.state() {
                    PointState::Ok(pv) => {
                        pre_ctx.insert(String::from(pa.id().point_id()), pv.clone());
                    }
                    _ => return rerr!("012", "pre point run failure"),
                }
            }
            let pre = Json::Object(pre_ctx);
            debug!("task pre {} - {}", task_id, pre);
            case_ctx.push((String::from("pre"), pre));
            Ok(case_ctx)
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

async fn point_runner_vec_create(
    flow_ctx: Arc<dyn FlowContext>,
    flow: Arc<Flow>,
    render_ctx_ext: Arc<Vec<(String, Json)>>,
    point_id_vec: Vec<String>,
    task_id: Arc<TaskIdStruct>,
) -> Result<Vec<(String, Box<dyn PointRunner>)>, Error> {
    let render_context = render_context_create(flow_ctx.clone(), flow.clone(), render_ctx_ext.clone());
    let mut point_runner_vec = vec![];
    for pid in point_id_vec {
        let pr = point_runner_create(
            flow_ctx.as_ref(),
            flow.as_ref(),
            &render_context,
            task_id.clone(),
            pid.clone(),
        ).await?;
        point_runner_vec.push((pid, pr));
    }
    Ok(point_runner_vec)
}

fn render_context_create(
    _: Arc<dyn FlowContext>,
    flow: Arc<Flow>,
    render_ctx_ext: Arc<Vec<(String, Json)>>,
) -> Context {
    let mut render_data: Map = Map::new();
    let config_def = flow.task_def();
    match config_def {
        Some(def) => {
            render_data.insert(String::from("def"), to_json(def).unwrap());
        }
        None => {}
    }

    for (k, v) in render_ctx_ext.iter() {
        render_data.insert(k.clone(), v.clone());
    }

    return Context::wraps(render_data).unwrap();
}

async fn point_runner_create(
    flow_ctx: &dyn FlowContext,
    flow: &Flow,
    render_context: &Context,
    task_id: Arc<TaskIdStruct>,
    point_id: String,
) -> Result<Box<dyn PointRunner>, Error> {
    let kind = flow.point_kind(point_id.as_ref());
    let create_arg = CreateArgStruct::new(
        flow,
        flow_ctx.get_handlebars(),
        render_context,
        task_id,
        kind.to_owned(),
        point_id,
    );

    flow_ctx
        .get_point_runner_factory()
        .create(&create_arg)
        .await
}

async fn case_run(flow_ctx: &dyn FlowContext, case_arg: CaseArgStruct) -> Box<dyn CaseAssess> {
    Box::new(case::run(flow_ctx, case_arg).await)
}

fn case_spawn(
    flow_ctx: Arc<dyn FlowContext>,
    case_arg: CaseArgStruct,
) -> JoinHandle<Box<dyn CaseAssess>> {
    let task_id = TASK_ID
        .try_with(|c| c.borrow().clone())
        .unwrap_or("".to_owned());
    let builder = Builder::new()
        .name(format!("case_{}", case_arg.id()))
        .spawn(case_run_arc(flow_ctx, task_id, case_arg));
    return builder.unwrap();
}

async fn case_run_arc(
    flow_ctx: Arc<dyn FlowContext>,
    task_id: String,
    case_arg: CaseArgStruct,
) -> Box<dyn CaseAssess> {
    TASK_ID.with(|tid| tid.replace(task_id));
    CASE_ID.with(|cid| cid.replace(case_arg.id().to_string()));
    case_run(flow_ctx.as_ref(), case_arg).await
}
