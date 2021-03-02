use std::collections::BTreeMap;
use async_std::sync::Arc;
use serde_yaml::Value;

#[derive(Debug)]
pub struct TaskContext {
    data: Vec<BTreeMap<String,String>>,
    flow: Value
}


impl TaskContext {

    pub fn new(flow: Value, data: Vec<BTreeMap<String,String>>) -> TaskContext {
        let context = TaskContext {
            flow,
            data
        };
        return context;
    }

    pub fn create_case(self: Arc<TaskContext>) -> Vec<CaseContext>{
        return self.data.iter()
            .enumerate()
            .map(|(idx, _d)| {
                CaseContext{
                    task_context: self.clone(),
                    data_index: idx
                }
            })
            .collect();
    }
}

#[derive(Debug)]
pub struct CaseContext {

    task_context: Arc<TaskContext>,
    data_index: usize

}


impl CaseContext {

    pub fn get_point_vec(&self) -> Vec<&str>{
        let task_point_chain_seq = self.task_context.flow["task"]["point"]["chain"].as_sequence().unwrap();
        let task_point_chain_vec:Vec<&str> = task_point_chain_seq.iter()
            .map(|e| {
                e.as_str().unwrap()
            })
            .collect();

        return task_point_chain_vec;
    }

    pub fn create_point(self: Arc<CaseContext>) -> Vec<PointContext>{
        return self.get_point_vec().into_iter()
            .map(|point_id| {
                PointContext{
                    case_context: self.clone(),
                    point_id: String::from(point_id)
                }
            })
            .collect();
    }
}

#[derive(Debug)]
pub struct PointContext{
    case_context: Arc<CaseContext>,
    point_id: String
}




