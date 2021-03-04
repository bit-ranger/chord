use std::collections::{BTreeMap, HashMap};
use std::borrow::Borrow;
use handlebars::Handlebars;
use serde::Serialize;
use serde_json::{Value, to_value};
use std::cell::RefCell;
use std::rc::Rc;
use std::ops::Deref;

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


    pub fn create_case(self: &TaskContext) -> Vec<CaseContext<'_, '_>> {
        return self.data.iter()
            .enumerate()
            .map(|(idx,_)| {
                CaseContext::new(
                    &self.config,
                    &self.data[idx]
                )
            })
            .collect();
    }

    fn get_config(self : &TaskContext) -> &Value{
        return self.config.borrow();
    }

    fn get_data(self : &TaskContext) -> &Vec<BTreeMap<String,String>>{
        return &self.data;
    }
}

#[derive(Debug)]
pub struct CaseContext<'c, 'd> {
    config: &'c Value,
    data: &'d BTreeMap<String,String>,
    dynamic_context_register : Rc<RefCell<HashMap<String, Value>>>
}


impl <'c, 'd> CaseContext <'c, 'd>{

    fn new(config: &'c Value, data: &'d BTreeMap<String,String>) -> CaseContext<'c, 'd>{
        let context = CaseContext {
            config,
            data,
            dynamic_context_register: Rc::new(RefCell::new(HashMap::new()))
        };

        return context;
    }



    pub fn create_point(self: &CaseContext<'c, 'd>) -> Vec<PointContext<'c, 'd>>{
        return self.get_point_vec()
            .into_iter()
            .filter(|point_id| {
                let none = self.config["point"][point_id].as_object().is_none();
                if none {
                    panic!("missing point config {}", point_id);
                } else {
                    return true;
                }
            })
            .map(|point_id| {
                PointContext::new(
                    self.config,
                    self.data,
                    point_id,
                    self.dynamic_context_register.clone()
                )
            })
            .collect();
    }

    pub fn register_dynamic_context(self: &CaseContext<'c, 'd>, name: &str, result: &Value) {
        self.dynamic_context_register.deref().borrow_mut().insert(String::from(name),to_value(result).unwrap());
    }

    fn get_point_vec(self: &CaseContext<'c,'d>) -> Vec<String>{
        let task_point_chain_arr = self.config["task"]["chain"].as_array().unwrap();
        let task_point_chain_vec:Vec<String> = task_point_chain_arr.iter()
            .map(|e| {
                e.as_str().map(|s|String::from(s)).unwrap()
            })
            .collect();

        return task_point_chain_vec;
    }

}


#[derive(Debug)]
pub struct PointContext<'c, 'd>
{
    config: &'c Value,
    data: &'d BTreeMap<String,String>,
    point_id: String,
    dynamic_context_register : Rc<RefCell<HashMap<String, Value>>>
}


impl <'c, 'd> PointContext<'c , 'd> {

    fn new(config: &'c Value,
           data: &'d BTreeMap<String,String>,
           point_id: String,
           dynamic_context_register : Rc<RefCell<HashMap<String, Value>>>
    ) -> PointContext<'c, 'd>{
        let context = PointContext {
            config,
            data,
            point_id: String::from(point_id),
            dynamic_context_register
        };
        return context;
    }

    pub fn get_id(self :&PointContext<'c,'d>) -> &str{
        return self.point_id.as_str();
    }

    pub async fn get_config_str(self: &PointContext<'c, 'd>, path: Vec<&str>) -> Option<String>
    {
        let config = self.config["point"][&self.point_id]["config"].borrow();

        let raw_config = path.iter()
            .fold(config,
                  |acc, k| acc[k].borrow()
            );

        match raw_config.as_str(){
            Some(s) => Some(self.render(s)),
            None=> None
        }

    }

    pub async fn get_meta_str(self : &PointContext<'c, 'd>, path: Vec<&str>) ->Option<String>
    {
        let config = self.config["point"][&self.point_id].borrow();

        let raw_config = path.iter()
            .fold(config,
                  |acc, k| acc[k].borrow()
            );

        match raw_config.as_str(){
            Some(s) => Some(self.render(s)),
            None=> None
        }
    }

    fn render(self: &PointContext<'c, 'd>, text: &str) -> String {
        return self.render_with(text, Option::<(&str, &Value)>::None);
    }

    fn render_with<T>(self: &PointContext<'c, 'd>, text: &str, with_data: Option<(&str, &T)>) -> String
        where
            T: Serialize
    {
        let mut data :HashMap<&str, Value> = HashMap::new();

        let config_def = self.config["task"]["def"].as_object();
        match config_def{
            Some(def) => {
                data.insert("def", to_value(def).unwrap());
            },
            None => {}
        }

        data.insert("data", to_value(self.data).unwrap());

        data.insert("dyn", to_value(self.dynamic_context_register.deref().borrow().deref()).unwrap());

        if with_data.is_some(){
            let (n, d) = with_data.unwrap();
            data.insert(n, to_value(d).unwrap());
        }

        let handlebars = Handlebars::new();
        let render = handlebars.render_template(text, &data).unwrap();
        return render;
    }

    pub async fn assert <T>(&self, condition: &str, with_data: &T) -> bool
        where
            T: Serialize
    {
        let template = format!(
            "{{{{#if {condition}}}}}true{{{{else}}}}false{{{{/if}}}}",
            condition = condition
        );

        let result = self.render_with(&template, Some(("result", with_data)));

        println!("assert {:?}", result);

        return if result.eq("true") {true} else {false};
    }



}


pub type PointResult = std::result::Result<Value, ()>;
pub type CaseResult = std::result::Result<Vec<(String, PointResult)>, ()>;
pub type TaskResult = std::result::Result<Vec<CaseResult>, ()>;
