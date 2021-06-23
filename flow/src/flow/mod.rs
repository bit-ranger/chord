use std::cell::RefCell;

use async_std::sync::Arc;
use async_std::task_local;

use chord::step::ActionFactory;
pub use task::arg::TaskIdSimple;
pub use task::TaskRunner;

use crate::model::app::{Context, FlowContextStruct};
use chord::rerr;
use chord::value::Map;
use chord::Error;
use std::collections::{HashMap, HashSet};

mod case;
mod step;
mod task;

task_local! {
    pub static CTX_ID: RefCell<String> = RefCell::new(String::new());
}

pub async fn context_create(action_factory: Box<dyn ActionFactory>) -> Arc<dyn Context> {
    Arc::new(FlowContextStruct::<'_>::new(action_factory))
}

pub async fn task_compose(
    task_names: Vec<String>,
    task_setting: Map,
) -> Result<Vec<Vec<String>>, Error> {
    let mut task_nodes: HashMap<String, TaskNode> = task_names
        .into_iter()
        .map(|n| TaskNode {
            name: n,
            layer: 1,
            depends_on: None,
            sub_sequence: HashSet::new(),
        })
        .fold(HashMap::new(), |mut m, n| {
            m.insert(n.name.clone(), n);
            m
        });

    for (n, s) in task_setting.iter() {
        if let Some(dep) = s["depends_on"].as_str() {
            if n == dep {
                return rerr!("compose", format!("cycle dependency recognized {}", dep));
            }

            if let Some(curr_node) = task_nodes.get(n) {
                if let Some(dep_node) = task_nodes.get(dep) {
                    if curr_node.sub_sequence.contains(dep_node.name.as_str()) {
                        return rerr!(
                            "compose",
                            format!("cycle dependency recognized {}", dep_node.name.as_str())
                        );
                    }
                } else {
                    return rerr!("compose", format!("un recognized task {}", n.as_str()));
                }
            } else {
                return rerr!("compose", format!("un recognized task {}", n.as_str()));
            }

            let dep_node = task_nodes.get_mut(dep).unwrap();
            dep_node.sub_sequence.insert(n.into());
            let dep_layer = dep_node.layer;

            let curr_node = task_nodes.get_mut(n).unwrap();
            curr_node.depends_on = Some(dep.into());
            curr_node.layer = dep_layer + 1;
        }
    }

    let layer_max = task_nodes
        .values()
        .map(|n| n.layer)
        .fold(usize::MIN, |a, b| a.max(b));

    let mut result = Vec::new();
    for layer in 1..layer_max {
        let layer_task: Vec<String> = task_nodes
            .values()
            .filter(|v| v.layer == layer)
            .map(|v| v.name.clone())
            .collect();
        result.push(layer_task);
    }

    return Ok(result);
}

struct TaskNode {
    name: String,
    layer: usize,
    depends_on: Option<String>,
    sub_sequence: HashSet<String>,
}
