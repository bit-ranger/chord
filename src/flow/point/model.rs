use std::borrow::Borrow;
use std::collections::BTreeMap;

use handlebars::Handlebars;
use serde::Serialize;
use serde_json::to_value;

use crate::flow::case::model::RenderContext;
use crate::model::context::PointContext;
use crate::model::value::Json;

#[derive(Debug)]
pub struct PointContextStruct<'c, 'd, 'h, 'reg, 'r>
{
    config: &'c Json,
    data: &'d BTreeMap<String,String>,
    point_id: String,
    handlebars: &'h Handlebars<'reg>,
    render_context: &'r RenderContext,
}


impl <'c, 'd, 'h, 'reg, 'r> PointContextStruct<'c, 'd, 'h, 'reg, 'r> {


    pub fn new(config: &'c Json,
               data: &'d BTreeMap<String,String>,
               point_id: &str,
               handlebars: &'h Handlebars<'reg>,
               render_context: &'r RenderContext
    ) -> PointContextStruct<'c, 'd, 'h, 'reg, 'r>{

        let context = PointContextStruct {
            config,
            data,
            point_id: String::from(point_id),
            handlebars,
            render_context
        };

        return context;
    }

    #[allow(dead_code)]
    pub fn get_id(self :&PointContextStruct<'c, 'd, 'h, 'reg, 'r>) -> &str{
        return self.point_id.as_str();
    }



    pub async fn get_meta_str(self : &PointContextStruct<'c, 'd, 'h, 'reg, 'r>, path: Vec<&str>) ->Option<String>
    {
        let config = self.config["point"][&self.point_id].borrow();

        let raw_config = path.iter()
            .fold(config,
                  |acc, k| acc[k].borrow()
            );

        match raw_config.as_str(){
            Some(s) => Some(self.render_inner(s)),
            None=> None
        }
    }

    fn render_inner(self: &PointContextStruct<'c, 'd, 'h, 'reg, 'r>, text: &str) -> String {
        let render = self.handlebars.render_template_with_context(
            text, self.render_context).unwrap();
        return render;
    }

    fn render_inner_with<T>(self: &PointContextStruct<'c, 'd, 'h, 'reg, 'r>, text: &str, with_data: (&str, &T)) -> String
        where
            T: Serialize
    {
        let mut ctx = self.render_context.data().clone();

        if let Json::Object(data) = &mut ctx{
            let (n, d) = with_data;
            data.insert(String::from(n), to_value(d).unwrap());
        }

        // let handlebars = Handlebars::new();
        let render = self.handlebars.render_template(
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

        let result = self.render_inner_with(&template, ("res", with_data));

        return if result.eq("true") {true} else {false};
    }



}


impl <'c, 'd, 'h, 'reg, 'r> PointContext for PointContextStruct<'c, 'd, 'h, 'reg, 'r> {


    fn get_config_rendered(self: &PointContextStruct<'c, 'd, 'h, 'reg, 'r>, path: Vec<&str>) -> Option<String>
    {
        let config = self.config["point"][&self.point_id]["config"].borrow();

        let raw_config = path.iter()
            .fold(config,
                  |acc, k| acc[k].borrow()
            );

        match raw_config.as_str(){
            Some(s) => Some(self.render_inner(s)),
            None=> None
        }

    }

    fn get_config(&self) -> &Json {
        let config = self.config["point"][&self.point_id]["config"].borrow();
        return config;
    }

    fn render(&self, text: &str) -> String {
        return self.render_inner(text);
    }
}
