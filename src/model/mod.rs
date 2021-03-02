use std::collections::{BTreeMap, HashMap};
use async_std::sync::Arc;
use serde_yaml::Value;
use std::borrow::Borrow;
use std::ops::Index;
use handlebars::Handlebars;

#[derive(Debug)]
pub struct TaskContext {
    data: Vec<BTreeMap<String,String>>,
    config: Value
}


impl TaskContext {

    pub fn new(config: Value, data: Vec<BTreeMap<String,String>>) -> TaskContext {
        let context = TaskContext {
            config,
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
        let task_point_chain_seq = self.task_context.config["task"]["point"]["chain"].as_sequence().unwrap();
        let task_point_chain_vec:Vec<&str> = task_point_chain_seq.iter()
            .map(|e| {
                e.as_str().unwrap()
            })
            .collect();

        return task_point_chain_vec;
    }

    pub fn create_point(self: Arc<CaseContext>) -> Vec<PointContext>{
        return self.get_point_vec().into_iter()
            .filter(|point_id| {
                let none = self.task_context.config["point"][point_id].as_mapping().is_none();
                if none {
                    panic!("missing point config {}", point_id);
                } else {
                    return true;
                }
            })
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


impl PointContext {

    pub fn get_config(&self) -> &Value{
        return self.case_context.task_context.config["point"][&self.point_id].borrow();
    }

    pub fn render_placeholder(&self, text: &str, more_data: Option<HashMap<String,String>>) -> String{
        let mut handlebars = Handlebars::new();
        let mut data :HashMap<String,String> = HashMap::new();

        let config_def = self.case_context.task_context.config["task"]["def"].as_mapping();
        match config_def{
            Some(def) => {
                def.iter()
                    .for_each(|(k, v)| {
                        data.insert(String::from(k.as_str().unwrap()),
                                    String::from(v.as_str().unwrap()));
                    })
            },
            None => {}
        }

        let case_data = self.case_context.task_context.data[self.case_context.data_index].borrow();
        case_data.iter()
            .for_each(|(k, v)| {
                data.insert(String::from(k),
                            String::from(v));
            });

        match more_data{
            Some(md) => {
                md.iter()
                    .for_each(|(k, v)| {
                        data.insert(String::from(k),
                                    String::from(v));
                    });
            }
            None => {}
        };

        let render = handlebars.render_template(text, &data).unwrap();
        return render;
    }

}



