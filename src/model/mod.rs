use std::collections::{BTreeMap, HashMap};
use std::borrow::Borrow;
use handlebars::Handlebars;
use serde::Serialize;
use serde_json::{Value, to_value};

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


    pub fn create_case<'t>(self: &'t TaskContext) -> Vec<CaseContext<'t>> {
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

    pub fn get_point_vec(self: &TaskContext) -> Vec<String>{
        let task_point_chain_arr = self.config["task"]["chain"].as_array().unwrap();
        let task_point_chain_vec:Vec<String> = task_point_chain_arr.iter()
            .map(|e| {
                e.as_str().map(|s|String::from(s)).unwrap()
            })
            .collect();

        return task_point_chain_vec;
    }

    fn get_config(self : &TaskContext) -> &Value{
        return self.config.borrow();
    }

    fn get_data(self : &TaskContext) -> &Vec<BTreeMap<String,String>>{
        return &self.data;
    }
}

#[derive(Debug)]
pub struct CaseContext<'t> {
    task_context: &'t TaskContext,
    data_index: usize,
}


impl <'t> CaseContext <'t>{

    fn new(task_context: &'t TaskContext, data_index: usize) -> CaseContext{
        let context = CaseContext {
            task_context,
            data_index
        };

        return context;
    }



    pub fn create_point<'c>(self: &'c CaseContext<'t>) -> Vec<PointContext<'t, 'c>>{
        return self.task_context.get_point_vec()
            .into_iter()
            .filter(|point_id| {
                let none = self.task_context
                    .config["point"][point_id].as_object().is_none();
                if none {
                    panic!("missing point config {}", point_id);
                } else {
                    return true;
                }
            })
            .map(|point_id| {
                PointContext::new(
                    self.task_context,
                    self,
                    String::from(point_id)
                )
            })
            .collect();
    }



    fn get_data(self: &CaseContext<'t>) -> &BTreeMap<String, String> {
        return &(self.task_context.get_data()[self.data_index]);
    }

}


#[derive(Debug)]
pub struct PointContext<'t, 'c>
where 't: 'c
{
    task_context: &'t TaskContext,
    case_context: &'c CaseContext<'t>,
    point_id: String
}


impl <'t, 'c> PointContext<'t , 'c> {

    fn new(task_context: &'t TaskContext, case_context: &'c CaseContext<'t>, point_id: String) -> PointContext<'t, 'c>{
        let context = PointContext {
            task_context,
            case_context,
            point_id: String::from(point_id)
        };
        return context;
    }

    pub fn get_id(self :&PointContext<'t,'c>) -> &str{
        return self.point_id.as_str();
    }

    pub async fn get_config_str(self: &PointContext<'t, 'c>, path: Vec<&str>) -> Option<String>
    {
        let config = self.task_context.get_config()["point"][&self.point_id]["config"].borrow();

        let raw_config = path.iter()
            .fold(config,
                  |acc, k| acc[k].borrow()
            );

        match raw_config.as_str(){
            Some(s) => Some(self.render(s, &Value::Null)),
            None=> None
        }

    }

    pub async  fn get_meta_str(&self, path: Vec<&str>) ->Option<String>
    {
        let config = self.task_context.get_config()["point"][&self.point_id].borrow();

        let raw_config = path.iter()
            .fold(config,
                  |acc, k| acc[k].borrow()
            );

        match raw_config.as_str(){
            Some(s) => Some(self.render(s, &Value::Null)),
            None=> None
        }
    }



    fn render<T>(self: &PointContext<'t, 'c>, text: &str, ext_data: &T) -> String
        where
            T: Serialize
    {
        let mut data :HashMap<&str, Value> = HashMap::new();

        let config_def = self.task_context.get_config()["task"]["def"].as_object();
        match config_def{
            Some(def) => {
                data.insert("def", to_value(def).unwrap());
            },
            None => {}
        }

        let case_data = self.case_context.get_data();
        data.insert("data", to_value(case_data).unwrap());

        data.insert("ext", to_value(ext_data).unwrap());

        let handlebars = Handlebars::new();
        let render = handlebars.render_template(text, &data).unwrap();
        return render;
    }

    pub async fn assert <T>(&self, condition: &str, ext_data: &T) -> bool
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



}


pub type PointResult = std::result::Result<Value, ()>;
pub type CaseResult = std::result::Result<Vec<(String, PointResult)>, ()>;
pub type TaskResult = std::result::Result<Vec<CaseResult>, ()>;
