use std::cell::RefCell;

use async_std::sync::Arc;
use async_std::task_local;

use chord::step::ActionFactory;
pub use task::arg::TaskIdSimple;
pub use task::TaskRunner;

use crate::model::app::{Context, FlowContextStruct};
use chord::rerr;
use chord::value::json;
use chord::value::Map;
use chord::Error;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

mod case;
mod step;
mod task;

task_local! {
    pub static CTX_ID: RefCell<String> = RefCell::new(String::new());
}

pub async fn context_create(action_factory: Box<dyn ActionFactory>) -> Arc<dyn Context> {
    Arc::new(FlowContextStruct::<'_>::new(action_factory))
}

pub fn task_compose(task_setting: Map) -> Result<Vec<Vec<String>>, Error> {
    let mut register: HashMap<String, TaskNode> = task_setting
        .iter()
        .map(|(n, _)| TaskNode {
            name: n.clone(),
            layer: 1,
            prev: HashSet::new(),
            next: HashSet::new(),
        })
        .fold(HashMap::new(), |mut m, n| {
            m.insert(n.name.clone(), n);
            m
        });

    for (name, node) in task_setting.iter() {
        let curr_node = register.get(name);
        if curr_node.is_none() {
            return rerr!("compose", format!("un recognized task {}", name));
        }
        let curr_node = curr_node.unwrap();

        if let Some(dep_vec) = node["depends_on"].as_array() {
            let dep_vec: Vec<String> = dep_vec
                .iter()
                .filter(|d| d.is_string())
                .map(|d| d.as_str().unwrap().to_owned())
                .collect();

            if dep_vec.is_empty() {
                continue;
            }

            // check cycle dependency
            let next_deep_set = next_deep(&register, curr_node);
            if dep_vec.contains(&curr_node.name) {
                return rerr!(
                    "compose",
                    format!("cycle dependency recognized {}", &curr_node.name)
                );
            };

            for dep in dep_vec.iter() {
                if let Some(dep_node) = register.get(dep.as_str()) {
                    if next_deep_set.contains(dep_node.name.as_str()) {
                        return rerr!(
                            "compose",
                            format!("cycle dependency recognized {}", dep_node.name.as_str())
                        );
                    }
                } else {
                    return rerr!("compose", format!("un recognized task {}", dep));
                }
            }

            // two-way poly tree
            let max_dep = dep_vec
                .iter()
                .map(|d| register.get(d).unwrap())
                .max_by_key(|d| d.layer)
                .unwrap();
            let max_dep_layer = max_dep.layer;
            let up_layer = max_dep_layer + 1 - curr_node.layer;

            let curr_node = register.get_mut(name).unwrap();
            curr_node.layer = max_dep_layer + 1;

            for dep in dep_vec {
                let dep_node = register.get_mut(dep.as_str()).unwrap();
                dep_node.next.insert(name.into());

                let curr_node = register.get_mut(name).unwrap();
                curr_node.prev.insert(dep);
            }

            for c in next_deep_set {
                let c_node = register.get_mut(c.as_str()).unwrap();
                c_node.layer = c_node.layer + up_layer;
            }
        }
    }

    let layer_max = register
        .values()
        .map(|n| n.layer)
        .fold(usize::MIN, |a, b| a.max(b));

    let mut result = Vec::new();
    for layer in 1..layer_max + 1 {
        let layer_task: Vec<String> = register
            .values()
            .filter(|v| v.layer == layer)
            .map(|v| v.name.clone())
            .sorted()
            .collect();
        result.push(layer_task);
    }

    return Ok(result);
}

struct TaskNode {
    name: String,
    layer: usize,
    prev: HashSet<String>,
    next: HashSet<String>,
}

impl Display for TaskNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(
            format!(
                "{}:{}:{:?}:{:?}",
                self.name, self.layer, self.prev, self.next
            )
            .as_str(),
        )
    }
}

fn next_deep(register: &HashMap<String, TaskNode>, node: &TaskNode) -> HashSet<String> {
    let mut deep: HashSet<String> = HashSet::new();
    next_deep_collect(register, &mut deep, node);
    return deep;
}

fn next_deep_collect(
    register: &HashMap<String, TaskNode>,
    deep: &mut HashSet<String>,
    node: &TaskNode,
) {
    for sub in &node.next {
        deep.insert(sub.clone());
        next_deep_collect(register, deep, register.get(sub).unwrap());
    }
}

#[test]
fn task_compose_test() {
    let compose = task_compose(
        json!({
            "a": {
                "depends_on": ["b", "c"]
            },
            "b": {
                "depends_on": ["d"]
            },
            "c": {
                "depends_on": []
            },
            "d": {
                "depends_on": []
            },
            "e": {
                "depends_on": ["a"]
            }
        })
        .as_object()
        .unwrap()
        .clone(),
    )
    .unwrap();

    assert_eq!(
        vec![vec!["c", "d"], vec!["b"], vec!["a"], vec!["e"]],
        compose
    );
}
