use std::cell::RefCell;
use std::mem::replace;

use async_std::sync::Arc;
use async_std::task_local;
use handlebars::Handlebars;

use chord::action::Factory;
use chord::err;
use chord::Error;
pub use task::arg::TaskIdSimple;
pub use task::TaskRunner;

use crate::model::app::{FlowApp, FlowAppStruct, RenderContext};
use async_std::path::Path;
use chord::value::{from_str, Number, Value};
use std::str::FromStr;

mod case;
mod step;
mod task;

task_local! {
    pub static CTX_ID: RefCell<String> = RefCell::new(String::new());
}

pub async fn context_create(action_factory: Box<dyn Factory>) -> Arc<dyn FlowApp> {
    Arc::new(FlowAppStruct::<'_>::new(action_factory))
}

fn render(
    handlebars: &Handlebars<'_>,
    render_ctx: &RenderContext,
    text: &str,
) -> Result<String, Error> {
    let render = handlebars.render_template_with_context(text, render_ctx);
    return match render {
        Ok(r) => Ok(r),
        Err(e) => Err(err!("tpl", format!("{}", e))),
    };
}

fn render_value(
    handlebars: &Handlebars,
    render_ctx: &RenderContext,
    task_path: &Path,
    value: &mut Value,
) -> Result<(), Error> {
    render_dollar_value(render_ctx.data(), task_path, value)?;
    match value {
        Value::String(v) => {
            let v_str = render(handlebars, render_ctx, v)?;
            let _ = replace(value, Value::String(v_str));
            Ok(())
        }
        Value::Object(v) => {
            for (_, v) in v.iter_mut() {
                render_value(handlebars, render_ctx, task_path, v)?;
            }
            Ok(())
        }
        Value::Array(v) => {
            for i in v {
                render_value(handlebars, render_ctx, task_path, i)?;
            }
            Ok(())
        }
        Value::Null => Ok(()),
        Value::Bool(_) => Ok(()),
        Value::Number(_) => Ok(()),
    }
}

fn render_dollar_value(context: &Value, task_path: &Path, value: &mut Value) -> Result<(), Error> {
    if value.is_object() {
        if !value["$num"].is_null() {
            if value.as_object().unwrap().len() != 1 {
                return Err(err!("001", "invalid args $num"));
            }
            if value["$num"].is_number() {
                let target = value["$num"].clone();
                let _ = replace(value, target);
            } else if value["$num"].is_string() {
                let _ = replace(
                    value,
                    Value::Number(
                        Number::from_str(value["$num"].as_str().unwrap())
                            .map_err(|_| err!("001", "invalid args $num"))?,
                    ),
                );
            } else {
                return Err(err!("001", "invalid args $num"));
            }
        } else if !value["$bool"].is_null() {
            if value.as_object().unwrap().len() != 1 {
                return Err(err!("001", "invalid args $bool"));
            }
            if value["$bool"].is_boolean() {
                let target = value["$bool"].clone();
                let _ = replace(value, target);
            } else if value["$bool"].is_string() {
                let _ = replace(
                    value,
                    Value::Bool(
                        bool::from_str(value["$bool"].as_str().unwrap())
                            .map_err(|_| err!("001", "invalid args $bool"))?,
                    ),
                );
            } else {
                return Err(err!("001", "invalid args $bool"));
            }
        } else if !value["$str"].is_null() {
            if value.as_object().unwrap().len() != 1 {
                return Err(err!("001", "invalid args $str"));
            }
            if value["$str"].is_string() {
                let target = value.as_object_mut().unwrap().remove("$str").unwrap();
                let _ = replace(value, target);
            } else {
                let _ = replace(value, Value::String(value["$str"].to_string()));
            }
        } else if !value["$obj"].is_null() {
            if value.as_object().unwrap().len() != 1 {
                return Err(err!("001", "invalid args $obj"));
            }
            if value["$obj"].is_object() {
                let mut target = value.as_object_mut().unwrap().remove("$obj").unwrap();
                render_dollar_value(context, task_path, &mut target)?;
                if !target.is_object() {
                    return Err(err!("001", "invalid args $obj"));
                }
                let _ = replace(value, target);
            } else if value["$obj"].is_string() {
                let _ = replace(
                    value,
                    Value::Object(
                        from_str(value["$obj"].as_str().unwrap())
                            .map_err(|_| err!("001", "invalid args $obj"))?,
                    ),
                );
            } else {
                return Err(err!("001", "invalid args $obj"));
            }
        } else if !value["$arr"].is_null() {
            if value.as_object().unwrap().len() != 1 {
                return Err(err!("001", "invalid args $arr"));
            }
            if value["$arr"].is_array() {
                let mut target = value.as_object_mut().unwrap().remove("$arr").unwrap();
                render_dollar_value(context, task_path, &mut target)?;
                if !target.is_array() {
                    return Err(err!("001", "invalid args $arr"));
                }
                let _ = replace(value, target);
            } else if value["$arr"].is_string() {
                let _ = replace(
                    value,
                    Value::Array(
                        from_str(value["$arr"].as_str().unwrap())
                            .map_err(|_| err!("001", "invalid args $arr"))?,
                    ),
                );
            } else {
                return Err(err!("001", "invalid args $arr"));
            }
        } else if !value["$ref"].is_null() {
            if value.as_object().unwrap().len() != 1 {
                return Err(err!("001", "invalid args $ref"));
            }
            if value["$ref"].is_string() {
                let ref_path = value["$ref"].as_str().unwrap();
                let path: Vec<&str> = ref_path.split(".").collect();
                let mut ref_val = context;
                for p in path {
                    ref_val = &ref_val[p];
                }
                let _ = replace(value, ref_val.clone());
            } else {
                return Err(err!("001", "invalid args $ref"));
            }
        } else if !value["$file"].is_null() {
            if value.as_object().unwrap().len() != 1 {
                return Err(err!("001", "invalid args $file"));
            }
            if value["$file"].is_string() {
                let file_path = value["$file"].as_str().unwrap();
                let path: Vec<&str> = file_path.split(".").collect();
                let mut ref_val = context;
                for p in path {
                    ref_val = &ref_val[p];
                }
                let _ = replace(value, ref_val.clone());
            } else {
                return Err(err!("001", "invalid args $ref"));
            }
        }
        Ok(())
    } else if value.is_array() {
        let arr = value.as_array_mut().unwrap();
        for i in arr {
            render_dollar_value(context, task_path, i)?;
        }
        Ok(())
    } else {
        Ok(())
    }
}
