use std::collections::BTreeMap;

use common::value::Json;

use crate::model::app::AppContext;
use common::task::TaskResult;

use self::task::arg::TaskArgStruct;

mod task;
mod case;
mod point;

pub async fn run(app_context: &dyn AppContext,
                 config: Json,
                 data: Vec<BTreeMap<String, String>>,
                 id: &str,
) -> TaskResult {
    let task_context = TaskArgStruct::new(config, data, id);
    return task::run_task(app_context, &task_context).await;
}