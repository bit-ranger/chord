use chrono::Utc;
use futures::future::join_all;
use itertools::Itertools;
use log::info;

use common::err;
use common::error::Error;
use common::task::{TaskState};
use common::value::{Json, Map};
use res::TaskAssessStruct;

use crate::flow::case;
use crate::flow::case::arg::CaseArgStruct;
use crate::flow::task::arg::TaskArgStruct;
use crate::model::app::AppContext;
use common::case::{CaseAssess, CaseState};
use common::point::PointState;

pub mod arg;
pub mod res;

pub async fn run_task(app_context: &dyn AppContext, task_context: &TaskArgStruct) -> TaskAssessStruct {
    let start = Utc::now();

    let pre_ctx=  match pre_ctx(app_context, task_context).await{
        Ok(pc) => pc,
        Err(e) => return TaskAssessStruct::new(task_context.id(), start, Utc::now(), TaskState::Err(e))
    };

    let mut data_case_arg_vec = match task_context.data_case_vec(&pre_ctx){
        Ok(v) => v,
        Err(e) => return TaskAssessStruct::new(task_context.id(), start, Utc::now(), TaskState::Err(e))
    };

    let mut futures = data_case_arg_vec.iter_mut().
        map(|case_arg| case_run(app_context, case_arg))
        .collect_vec();

    futures.reserve(0);
    let mut case_assess_vec  = Vec::<Box<dyn CaseAssess>>::new();
    let limit_concurrency = task_context.limit_concurrency();
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
        TaskAssessStruct::new(task_context.id(), start, Utc::now(),
                              TaskState::Fail(case_assess_vec))
    } else {
        TaskAssessStruct::new(task_context.id(), start, Utc::now(),
                              TaskState::Ok(case_assess_vec))
    };
}


async fn pre_ctx(app_context: &dyn AppContext, task_context: &TaskArgStruct) -> Result<Vec<(String, Json)>, Error>{
    let mut case_ctx = vec![];
    let pre_case = task_context.pre_case();
    if pre_case.is_none() {
        return Ok(case_ctx);
    }
    let mut pre_case = pre_case.unwrap();

    let pre_assess = case_run(app_context, &mut pre_case).await;
    match pre_assess.state() {
        CaseState::Ok(pa_vec) => {
            let mut pre_ctx = Map::new();
            for pa in pa_vec {
                match pa.state() {
                    PointState::Ok(pv) => {
                        pre_ctx.insert(String::from(pa.id()), pv.clone());
                    },
                    _ => return err!("012", "pre point run failure")
                }
            }
            let pre = Json::Object(pre_ctx);
            info!("pre_case: {:?}", pre);
            case_ctx.push((String::from("pre"), pre));
            Ok(case_ctx)
        }
        CaseState::Fail(pa_vec) => {
            let pa_last = pa_vec.last().unwrap();
            err!("020", format!("pre Fail : {}", pa_last.id()).as_str())
        },
        CaseState::Err(e) => {
            err!("021", format!("pre Err  : {}", e.to_string()).as_str())
        }
    }


}

async fn case_run(app_context: &dyn AppContext, case_arg: &mut CaseArgStruct<'_,'_,'_>) -> Box<dyn CaseAssess>{
    Box::new(case::run(app_context, case_arg).await)
}