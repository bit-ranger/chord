use std::collections::BTreeMap;

use crate::model::context::AppContext;
use crate::model::context::TaskResult;
use crate::model::value::Json;

use self::task::model::TaskContextStruct;

mod task;
mod case;
mod point;

pub async fn run(app_context: &dyn AppContext,
                 config: Json,
                 data: Vec<BTreeMap<String,String>>,
) -> TaskResult{

    let task_context = TaskContextStruct::new(config, data);
    return task::run_task(app_context, &task_context).await;
}