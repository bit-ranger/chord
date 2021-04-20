use chrono::Utc;
use futures::future::join_all;
use itertools::Itertools;
use log::{debug, warn, trace};

use chord_common::rerr;
use chord_common::error::Error;
use chord_common::task::{TaskState};
use chord_common::value::{Json, Map};
use res::TaskAssessStruct;

use crate::flow::case;
use crate::flow::case::arg::CaseArgStruct;
use crate::flow::task::arg::TaskArgStruct;
use crate::model::app::AppContext;
use chord_common::case::{CaseAssess, CaseState};
use chord_common::point::PointState;
use async_std::sync::Arc;
use async_std::task::{Builder, JoinHandle};

pub mod arg;
pub mod res;

pub async fn run(app_ctx: Arc<dyn AppContext>, arg: &TaskArgStruct) -> TaskAssessStruct {
    trace!("task start {}", arg.id());
    let start = Utc::now();

    let pre_ctx=  match pre_ctx(app_ctx.as_ref(), arg).await{
        Ok(pc) => pc,
        Err(e) => {
            warn!("task Err {}", arg.id());
            return TaskAssessStruct::new(arg.id(), start, Utc::now(), TaskState::Err(e))
        }
    };

    let data_case_arg_vec = match arg.case_arg_vec(pre_ctx){
        Ok(v) => v,
        Err(e) => {
            warn!("task Err {}", arg.id());
            return TaskAssessStruct::new(arg.id(), start, Utc::now(), TaskState::Err(e))
        }
    };

    trace!("task load data {}, {}", arg.id(), data_case_arg_vec.len());

    let mut futures = data_case_arg_vec.into_iter()
        .map(|case_arg| case_spawn(app_ctx.clone(), case_arg))
        .collect_vec();

    futures.reserve(0);
    let mut case_assess_vec  = Vec::<Box<dyn CaseAssess>>::new();
    let limit_concurrency = arg.limit_concurrency();
    loop {
        if futures.len() >  limit_concurrency{
            let off = futures.split_off(futures.len() - limit_concurrency);
            case_assess_vec.extend(join_all(off).await);
        } else {
            case_assess_vec.extend(join_all(futures).await);
            break;
        }
    }

    let any_fail = case_assess_vec.iter()
        .any(|ca| !ca.state().is_ok());

    return if any_fail {
        warn!("task Fail {}", arg.id());
        TaskAssessStruct::new(arg.id(), start, Utc::now(),
                              TaskState::Fail(case_assess_vec))
    } else {
        debug!("task Ok {}", arg.id());
        TaskAssessStruct::new(arg.id(), start, Utc::now(),
                              TaskState::Ok(case_assess_vec))
    };
}


async fn pre_ctx(app_ctx: &dyn AppContext, arg: &TaskArgStruct) -> Result<Vec<(String, Json)>, Error>{
    let mut case_ctx = vec![];
    let pre_arg = arg.pre_arg();
    if pre_arg.is_none() {
        return Ok(case_ctx);
    }
    let pre_arg = pre_arg.unwrap();

    let pre_assess = case_run(app_ctx, &pre_arg).await;
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
            debug!("task pre {} - {}", arg.id(), pre);
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
    let builder = Builder::new()
        .name(format!("case_{}", case_arg.id()))
        .spawn(case_run_arc(app_ctx, case_arg));
    return builder.unwrap();
}

async fn case_run_arc(app_ctx: Arc<dyn AppContext>, case_arg: CaseArgStruct) -> Box<dyn CaseAssess>{
    case_run(app_ctx.as_ref(), &case_arg).await
}