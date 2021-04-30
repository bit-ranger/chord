use std::borrow::Borrow;


use handlebars::{Handlebars, Context};
use log::info;
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
                match render(self.handlebars, self.render_context, s){
                    Ok(s) => Some(s),
                    Err(_) => None
                }
            },
            None=> None
        }
    }



    pub fn timeout(&self) -> Duration{
        self.flow.point_timeout(self.id())
    }


    pub fn kind(&self) -> &str{
        self.flow.point_kind(self.id())
    }

}


pub async fn assert(handlebars: &Handlebars<'_>,
                    render_context: &Context,
                    condition: &str) -> bool
{
    let template = format!(
        "{{{{#if {condition}}}}}true{{{{else}}}}false{{{{/if}}}}",
        condition = condition
    );

    let result = render(handlebars, render_context, &template);
    match result {
        Ok(result) => if result.eq("true") { true } else { false },
        Err(e) => {
            info!("assert failure: {} >>> {}", condition, e);
            false
        }
    }
}

pub fn render(handlebars: & Handlebars<'_>,
              render_context: & Context,
              text: &str) -> Result<String, Error> {
    let render = handlebars.render_template_with_context(
        text, render_context);
    return match render {
        Ok(r) => Ok(r),
        Err(e) => rerr!("tpl", format!("{}", e))
    };
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
                match render(self.handlebars, self.render_context, s){
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
        return render(self.handlebars, self.render_context, text);
    }
}

unsafe impl <'c, 'h, 'reg, 'r> Send for PointArgStruct<'c, 'h, 'reg, 'r>
{
}

unsafe impl <'c, 'h, 'reg, 'r> Sync for PointArgStruct<'c, 'h, 'reg, 'r>
{
}