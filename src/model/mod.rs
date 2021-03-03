use std::collections::{BTreeMap, HashMap};
use async_std::sync::Arc;
use std::borrow::Borrow;
use std::ops::Index;
use handlebars::Handlebars;
use serde::Serialize;
use serde_yaml::{Value, to_value};

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
                CaseContext::new(
                    self.clone(),
                    idx
                )
            })
            .collect();
    }
}

#[derive(Debug)]
pub struct CaseContext {

    task_context: Arc<TaskContext>,
    data_index: usize,
    point_context_register: HashMap<String, HashMap<String,Value>>
}


impl CaseContext {

    fn new(task_context: Arc<TaskContext>, data_index: usize) -> CaseContext{
        let context = CaseContext {
            task_context,
            data_index,
            point_context_register: HashMap::new()
        };

        return context;
    }


    pub fn get_point_vec(&self) -> Vec<&str>{
        let task_point_chain_seq = self.task_context.config["task"]["chain"].as_sequence().unwrap();
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
                PointContext::new(
                    self.clone(),
                    String::from(point_id)
                )
            })
            .collect();
    }


}


#[derive(Debug)]
pub struct PointContext{
    case_context: Arc<CaseContext>,
    point_id: String,
    context_register: HashMap<String,Value>
}


impl PointContext {

    fn new(case_context: Arc<CaseContext>, point_id: String) -> PointContext{
        let context = PointContext {
            case_context,
            point_id: String::from(point_id),
            context_register: HashMap::new()
        };
        return context;
    }

    pub fn get_config_str(&self, path: Vec<&str>) -> Option<String>
    {
        let config = self.case_context.task_context.config["point"][&self.point_id].borrow();

        let raw_config = path.iter()
            .fold(config, |acc, k| acc[k].borrow());

        return raw_config.as_str().map(|x| self.render(x, &Value::Null));
    }


    fn render<T>(&self, text: &str, ext_data: &T) -> String
        where
            T: Serialize
    {
        let mut data :HashMap<&str, Value> = HashMap::new();

        let config_def = self.case_context.task_context.config["task"]["def"].as_mapping();
        match config_def{
            Some(def) => {
                data.insert("def", to_value(def).unwrap());
            },
            None => {}
        }

        let case_data = self.case_context.task_context.data[self.case_context.data_index].borrow();
        data.insert("data", to_value(case_data).unwrap());


        data.insert("ext", to_value(ext_data).unwrap());

        let mut handlebars = Handlebars::new();
        let render = handlebars.render_template(text, &data).unwrap();
        return render;
    }

    pub fn assert <T>(&self, condition: &str, ext_data: &T) -> bool
        where
            T: Serialize
    {
        let template = format!(
            "{{{{#if {condition}}}}}true{{{{else}}}}false{{{{/if}}}}",
            condition = condition
        );

        let result = self.render(&template, ext_data);
        return if result.eq("true") {true} else {false};
    }

    pub fn register_context<T>(&mut self, name: String, context: T)
        where
            T: Serialize{
        self.context_register.insert(name, to_value(context).unwrap());
    }


}



