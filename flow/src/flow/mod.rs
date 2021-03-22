

use common::point::PointRunner;

use crate::model::app::{AppContext, AppContextStruct};

use self::task::arg::TaskArgStruct;
use common::flow::Flow;
use common::value::Json;
use common::task::TaskAssess;

mod task;
mod case;
mod point;

pub async fn run(app_context: &dyn AppContext,
                 flow: Flow,
                 data: Vec<Json>,
                 id: &str
) -> Box<dyn TaskAssess> {
    let task_assess = task::run_task(app_context, &TaskArgStruct::new(flow, data, id)).await;
    return Box::new(task_assess);
}

pub async fn create_app_context(pt_runner: Box<dyn PointRunner>) -> Box<dyn AppContext> {
    Box::new(AppContextStruct::<'_>::new(pt_runner))
}

