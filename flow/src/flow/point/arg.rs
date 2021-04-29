use std::borrow::Borrow;


use handlebars::{Handlebars};
use log::info;
use serde::Serialize;
use chord_common::value::to_json;

use chord_common::error::Error;
use chord_common::point::PointArg;
use chord_common::value::Json;
use chord_common::rerr;
use chord_common::flow::Flow;

use crate::flow::case::arg::RenderContext;
use std::time::Duration;

#[derive(Debug)]
pub struct PointArgStruct<'c, 'h, 'reg, 'r>
{
    flow: &'c Flow,
    id: String,
    handlebars: &'h Handlebars<'reg>,
    render_context: &'r RenderContext,
}


impl <'c, 'h, 'reg, 'r> PointArgStruct<'c, 'h, 'reg, 'r> {


    pub fn new(flow: &'c Flow,
               id: &str,
               handlebars: &'h Handlebars<'reg>,
               render_context: &'r RenderContext
    ) -> PointArgStruct<'c, 'h, 'reg, 'r>{

        let context = PointArgStruct {
            flow,
            id: String::from(id),
            handlebars,
            render_context
        };

        return context;
    }

    #[allow(dead_code)]
    pub fn id(self :&PointArgStruct<'c, 'h, 'reg, 'r>) -> &str{
        return self.id.as_str();
    }

    pub async fn meta_str(self : &PointArgStruct<'c, 'h, 'reg, 'r>, path: Vec<&str>) ->Option<String>
    {
        let config = self.flow.point(self.id());

        let raw_config = path.iter()
            .fold(config,
                  |acc, k| acc[k].borrow()
            );

        match raw_config.as_str(){
            Some(s) => {
                match self.render_inner(s){
                    Ok(s) => Some(s),
                    Err(_) => None
                }
            },
            None=> None
        }
    }

    fn render_inner(self: &PointArgStruct<'c, 'h, 'reg, 'r>, text: &str) -> Result<String, Error> {
        let render = self.handlebars.render_template_with_context(
            text, self.render_context);
        return match render {
            Ok(r) => Ok(r),
            Err(e) => rerr!("tpl", format!("{}", e))
        };
    }

    fn render_inner_with<T>(self: &PointArgStruct<'c, 'h, 'reg, 'r>, text: &str, with_data: (&str, &T)) -> Result<String, Error>
        where
            T: Serialize
    {
        let mut ctx = self.render_context.data().clone();

        if let Json::Object(data) = &mut ctx{
            let (n, d) = with_data;
            data.insert(String::from(n), to_json(d).unwrap());
        }

        // let handlebars = Handlebars::new();
        let render = self.handlebars.render_template(
            text, &ctx);
        return match render {
            Ok(r) => Ok(r),
            Err(e) => rerr!("tpl", format!("{}", e))
        };
    }

    pub async fn assert <T>(&self, condition: &str, with_data: &T) -> bool
        where
            T: Serialize
    {
        let template = format!(
            "{{{{#if {condition}}}}}true{{{{else}}}}false{{{{/if}}}}",
            condition = condition
        );

        let result = self.render_inner_with(&template, ("res", with_data));
        match result {
            Ok(result) => if result.eq("true") {true} else {false},
            Err(e) => {
                info!("assert failure: {} >>> {}", condition, e);
                false
            }
        }
    }

    pub fn timeout(&self) -> Duration{
        self.flow.point_timeout(self.id())
    }


    pub fn kind(&self) -> &str{
        self.flow.point_kind(self.id())
    }

}


impl <'c, 'h, 'reg, 'r> PointArg for PointArgStruct<'c, 'h, 'reg, 'r> {


    fn config_rendered(self: &PointArgStruct<'c, 'h, 'reg, 'r>, path: Vec<&str>) -> Option<String>
    {
        let config = self.flow.point_config(self.id());

        let raw_config = path.iter()
            .fold(config,
                  |acc, k| acc[k].borrow()
            );

        match raw_config.as_str(){
            Some(s) => {
                match self.render_inner(s){
                    Ok(s) => Some(s),
                    Err(_) => None
                }
            },
            None=> None
        }

    }

    fn config(&self) -> &Json {
        let config = self.flow.point_config(self.id());
        return config;
    }

    fn render(&self, text: &str) -> Result<String, Error> {
        return self.render_inner(text);
    }
}

unsafe impl <'c, 'h, 'reg, 'r> Send for PointArgStruct<'c, 'h, 'reg, 'r>
{
}

unsafe impl <'c, 'h, 'reg, 'r> Sync for PointArgStruct<'c, 'h, 'reg, 'r>
{
}