use chrono::{Utc};
use futures::future::join_all;
use itertools::Itertools;
use log::{debug, warn, trace};

use chord_common::rerr;
use chord_common::error::Error;
use chord_common::task::{TaskState, TaskAssess};
use chord_common::value::{Json, Map};
use res::TaskAssessStruct;

use crate::flow::case;
use crate::flow::case::arg::CaseArgStruct;
use crate::model::app::AppContext;
use chord_common::case::{CaseAssess, CaseState};
use chord_common::point::PointState;
use async_std::sync::Arc;
use async_std::task::{Builder, JoinHandle};
use chord_common::flow::Flow;
use async_std::task_local;
use std::cell::RefCell;
use crate::CASE_ID;

pub mod res;

task_local! {
    pub static TASK_ID: RefCell<String> = RefCell::new(String::new());
}

pub struct Runner {
    app_ctx: Arc<dyn AppContext>,
    flow: Arc<Flow>,
    id: String,
    pre_ctx: Arc<Vec<(String, Json)>>,
    case_id_offset: usize
}

impl Runner {

    pub async fn new(app_ctx: Arc<dyn AppContext>,
                     flow: Arc<Flow>,
                     id: String) -> Result<Runner, Error>{
        let pre_ctx=  pre_ctx(app_ctx.clone(), flow.clone(), id.clone()).await?;
        let runner = Runner {
            app_ctx,
            flow,
            id,
            pre_ctx: Arc::new(pre_ctx),
            case_id_offset: 1
        };
        Ok(runner)
    }


    pub async fn run(&mut self, data: Vec<Json>) -> Box<dyn TaskAssess>{
        Box::new(self.run0(data).await)
    }

    async fn run0(&mut self, data: Vec<Json>) -> TaskAssessStruct{
        let data_len = data.len();
        trace!("task start {}", self.id);
        let start = Utc::now();

        let ca_vec = match self.case_arg_vec(data){
            Ok(v) => v,
            Err(e) => {
                warn!("task Err {}", self.id);
                return TaskAssessStruct::new(self.id.as_str(), start, Utc::now(), TaskState::Err(e))
            }
        };
        self.case_id_offset = self.case_id_offset + data_len;
        trace!("task load data {}, {}", self.id, ca_vec.len());

        let mut futures = ca_vec.into_iter()
            .map(|ca| case_spawn(self.app_ctx.clone(), ca))
            .collect_vec();

        futures.reserve(0);
        let mut case_assess_vec  = Vec::<Box<dyn CaseAssess>>::new();
        let limit_concurrency =  self.flow.limit_concurrency();
        loop {
            if futures.len() >  limit_concurrency{
                let off = futures.split_off(futures.len() - limit_concurrency);
                let off_result = join_all(off).await;
                case_assess_vec.extend(off_result);
            } else {
                case_assess_vec.extend(join_all(futures).await);
                break;
            }
        }

        let any_fail = case_assess_vec.iter()
            .any(|ca| !ca.state().is_ok());

        return if any_fail {
            warn!("task Fail {}", self.id);
            TaskAssessStruct::new(self.id.as_str(), start, Utc::now(),
                                  TaskState::Fail(case_assess_vec))
        } else {
            debug!("task Ok {}", self.id);
            TaskAssessStruct::new(self.id.as_str(), start, Utc::now(),
                                  TaskState::Ok(case_assess_vec))
        };
    }



    pub fn case_arg_vec<'p>(&self, data: Vec<Json>) -> Result<Vec<CaseArgStruct>, Error> {
        let case_point_id_vec = self.flow.case_point_id_vec()?;
        let vec = data.into_iter()
            .enumerate()
            .map(|(i, d)| {
                CaseArgStruct::new(
                    self.case_id_offset + i,
                    self.flow.clone(),
                    d,
                    case_point_id_vec.clone(),
                    self.pre_ctx.clone()
                )
            })
            .collect();
        return Ok(vec);
    }
}

fn pre_arg(flow: Arc<Flow>) -> Option<CaseArgStruct> {
    let pre_pt_id_vec = flow.pre_point_id_vec();
    if pre_pt_id_vec.is_none() {
        return None
    }
    let pre_pt_id_vec = pre_pt_id_vec.unwrap();
    return if pre_pt_id_vec.is_empty() {
        None
    } else {
        Some(
            CaseArgStruct::new(
                0,
                flow.clone(),
                Json::Null,
                pre_pt_id_vec,
                Arc::new(Vec::new())
            )
        )
    }

}

async fn pre_ctx(app_ctx: Arc<dyn AppContext>,
                 flow: Arc<Flow>,
                 id: String) -> Result<Vec<(String, Json)>, Error>{
    let mut case_ctx = vec![];
    let pre_arg = pre_arg(flow);
    if pre_arg.is_none() {
        return Ok(case_ctx);
    }
    let pre_arg = pre_arg.unwrap();

    let pre_assess = case_run(app_ctx.as_ref(), &pre_arg).await;
    match pre_assess.state() {
        CaseState::Ok(pa_vec) => {
            let mut pre_ctx = Map::new();
            for pa in pa_vec {
                match pa.state() {
                    PointState::Ok(pv) => {
                        pre_ctx.insert(String::from(pa.id()), pv.clone());
                    },
                    _ => return rerr!("012", "pre point run failure")
                }
            }
            let pre = Json::Object(pre_ctx);
            debug!("task pre {} - {}", id, pre);
            case_ctx.push((String::from("pre"), pre));
            Ok(case_ctx)
        }
        CaseState::Fail(pa_vec) => {
            let pa_last = pa_vec.last().unwrap();
            rerr!("020", format!("pre Fail : {}", pa_last.id()))
        },
        CaseState::Err(e) => {
            rerr!("021", format!("pre Err  : {}", e.to_string()))
        }
    }
}

async fn case_run(app_ctx: &dyn AppContext, case_arg: &CaseArgStruct) -> Box<dyn CaseAssess>{
    Box::new(case::run(app_ctx, case_arg).await)
}

fn case_spawn(app_ctx: Arc<dyn AppContext>, case_arg: CaseArgStruct) -> JoinHandle<Box<dyn CaseAssess>>{
    let task_id = TASK_ID.try_with(|c| c.borrow().clone()).unwrap_or("".to_owned());
    let builder = Builder::new()
        .name(format!("case_{}", case_arg.id()))
        .spawn(case_run_arc(app_ctx, task_id, case_arg));
    return builder.unwrap();
}

async fn case_run_arc(app_ctx: Arc<dyn AppContext>, task_id: String, case_arg: CaseArgStruct) -> Box<dyn CaseAssess> {
    TASK_ID.with(|tid| tid.replace(task_id));
    CASE_ID.with(|cid| cid.replace(case_arg.id()));
    case_run(app_ctx.as_ref(), &case_arg).await
}