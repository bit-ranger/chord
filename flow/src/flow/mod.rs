use std::collections::BTreeMap;

use common::point::PointRunner;
use common::task::TaskResult;
use common::value::Json;

use crate::model::app::{AppContext, AppContextStruct};

use self::task::arg::TaskArgStruct;

mod task;
mod case;
mod point;

pub async fn run(app_context: &dyn AppContext,
                 flow: Json,
                 data: Vec<BTreeMap<String, String>>,
                 id: &str
) -> TaskResult {
    return task::run_task(app_context, &TaskArgStruct::new(flow, data, id)).await;
}

pub async fn mk_app_context(point_runner: Box<dyn PointRunner>) -> Box<dyn AppContext> {
    Box::new(AppContextStruct::<'_>::new(point_runner))
}

