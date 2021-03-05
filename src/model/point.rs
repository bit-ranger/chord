use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::ops::Deref;
use std::rc::Rc;

use serde::Serialize;
use serde_json::{to_value, Value};
use handlebars::Handlebars;

#[derive(Debug)]
pub struct PointContext<'c, 'd>
{
    config: &'c Value,
    data: &'d BTreeMap<String,String>,
    point_id: String,
    dynamic_context_register : Rc<RefCell<HashMap<String, Value>>>
}


impl <'c, 'd> PointContext<'c , 'd> {

    pub fn new(config: &'c Value,
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

        // println!("{}", to_value(&data).unwrap());

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
