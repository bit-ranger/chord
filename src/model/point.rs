use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::{BTreeMap};
use std::ops::Deref;
use std::rc::Rc;

use serde::Serialize;
use serde_json::{to_value};
use handlebars::{Handlebars, Context};
use crate::model::Json;
use crate::model::Error;


pub trait PointContext{


    fn get_config_str(&self, path: Vec<&str>) -> Option<String>;
}


#[derive(Debug)]
pub struct PointContextStruct<'c, 'd>
{
    config: &'c Json,
    data: &'d BTreeMap<String,String>,
    point_id: String,
    render_context: Rc<RefCell<Context>>
}


impl <'c, 'd> PointContextStruct<'c , 'd> {


    pub fn new(config: &'c Json,
               data: &'d BTreeMap<String,String>,
               point_id: String,
               render_context: Rc<RefCell<Context>>
    ) -> PointContextStruct<'c, 'd>{

        let context = PointContextStruct {
            config,
            data,
            point_id: String::from(point_id),
            render_context
        };

        return context;
    }

    pub fn get_id(self :&PointContextStruct<'c,'d>) -> &str{
        return self.point_id.as_str();
    }



    pub async fn get_meta_str(self : &PointContextStruct<'c, 'd>, path: Vec<&str>) ->Option<String>
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

    fn render(self: &PointContextStruct<'c, 'd>, text: &str) -> String {
        let render_context = self.render_context.deref().borrow();
        let render_context = render_context.borrow().deref();

        let handlebars = Handlebars::new();
        let render = handlebars.render_template_with_context(
            text, render_context).unwrap();
        return render;
    }

    fn render_with<T>(self: &PointContextStruct<'c, 'd>, text: &str, with_data: (&str, &T)) -> String
        where
            T: Serialize
    {
        let mut ctx = self.render_context.borrow_mut().data().clone();

        if let Json::Object(data) = &mut ctx{
            let (n, d) = with_data;
            data.insert(String::from(n), to_value(d).unwrap());
        }

        let handlebars = Handlebars::new();
        let render = handlebars.render_template(
            text, &ctx).unwrap();
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

        let result = self.render_with(&template, ("result", with_data));

        println!("assert {:?}", result);

        return if result.eq("true") {true} else {false};
    }

    pub async fn register_dynamic(self: &PointContextStruct<'c, 'd>, result: &Json) {
        let mut x = self.render_context.borrow_mut();
        let y = x.data_mut();
        if let Json::Object(data) = y{
            data["dyn"][self.point_id.as_str()] = to_value(result).unwrap();
        }
    }

}


impl <'c, 'd> PointContext for PointContextStruct<'c,'d> {


    fn get_config_str(self: &PointContextStruct<'c, 'd>, path: Vec<&str>) -> Option<String>
    {
        let config = self.config["point"][&self.point_id]["property"].borrow();

        let raw_config = path.iter()
            .fold(config,
                  |acc, k| acc[k].borrow()
            );

        match raw_config.as_str(){
            Some(s) => Some(self.render(s)),
            None=> None
        }

    }
}


pub type PointResult = std::result::Result<Json, Error>;
