

use chord_common::point::PointRunner;

use crate::model::app::{AppContext, AppContextStruct};

use self::task::arg::TaskArgStruct;
use chord_common::flow::Flow;
use chord_common::value::Json;
use chord_common::task::TaskAssess;
use async_std::sync::Arc;

mod task;
mod case;
mod point;

pub async fn run(app_ctx: Arc<dyn AppContext>,
                 flow: Flow,
                 data: Vec<Json>,
                 id: &str
) -> Box<dyn TaskAssess> {
    let task_assess = task::run(app_ctx, &TaskArgStruct::new(flow, data, id)).await;
    return Box::new(task_assess);
}

pub async fn create_app_context(pt_runner: Box<dyn PointRunner>) -> Arc<dyn AppContext> {
    Arc::new(AppContextStruct::<'_>::new(pt_runner))
}

