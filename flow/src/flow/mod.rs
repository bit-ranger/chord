use std::cell::RefCell;

use async_std::sync::Arc;
use async_std::task_local;
use handlebars::Handlebars;

use chord::action::Factory;
use chord::err;
use chord::Error;
pub use task::arg::TaskIdSimple;
pub use task::TaskRunner;

use crate::model::app::{FlowApp, FlowAppStruct, RenderContext};
use chord::value::{from_str, Map, Number, Value};
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
    value_raw: &Value,
) -> Result<Value, Error> {
    match value_raw {
        Value::String(v) => {
            let v_str = render(handlebars, render_ctx, v)?;
            Ok(Value::String(v_str))
        }
        Value::Object(v) => {
            let mut let_value = Map::new();
            for (k, v) in v.iter() {
                let v = render_value(handlebars, render_ctx, v)?;
                let_value.insert(k.clone(), v);
            }
            let value = Value::Object(let_value);
            let value = render_dollar(&value, render_ctx.data())?;
            Ok(value)
        }
        Value::Array(v) => {
            let mut arr = Vec::new();
            for e in v {
                let av = render_value(handlebars, render_ctx, e)?;
                arr.push(av)
            }
            let value = Value::Array(arr);
            let value = render_dollar(&value, render_ctx.data())?;
            Ok(value)
        }
        Value::Null => Ok(Value::Null),
        Value::Bool(v) => Ok(Value::Bool(v.clone())),
        Value::Number(v) => Ok(Value::Number(v.clone())),
    }
}

fn render_dollar(val: &Value, ref_from: &Value) -> Result<Value, Error> {
    return match val {
        Value::Object(map) => {
            if map.contains_key("$str") {
                if map.len() != 1 {
                    return Err(err!("001", "invalid args $str"));
                }
                let v = &map["$str"];
                if v.is_string() {
                    Ok(v.clone())
                } else {
                    Ok(Value::String(map["$str"].to_string()))
                }
            } else if map.contains_key("$num") {
                if map.len() != 1 {
                    return Err(err!("001", "invalid args $num"));
                }
                let v = &map["$num"];
                if v.is_number() {
                    Ok(v.clone())
                } else if v.is_string() {
                    Ok(Value::Number(
                        Number::from_str(v.as_str().unwrap())
                            .map_err(|_| err!("001", "invalid args $num"))?,
                    ))
                } else {
                    Err(err!("001", "invalid args $num"))
                }
            } else if map.contains_key("$bool") {
                if map.len() != 1 {
                    return Err(err!("001", "invalid args $bool"));
                }
                let v = &map["$bool"];
                if v.is_boolean() {
                    Ok(v.clone())
                } else if v.is_string() {
                    Ok(Value::Bool(
                        bool::from_str(v.as_str().unwrap())
                            .map_err(|_| err!("001", "invalid args $bool"))?,
                    ))
                } else {
                    Err(err!("001", "invalid args $bool"))
                }
            } else if map.contains_key("$obj") {
                if map.len() != 1 {
                    return Err(err!("001", "invalid args $obj"));
                }
                let v = &map["$obj"];
                if v.is_object() {
                    render_dollar(v, ref_from)
                } else if v.is_string() {
                    Ok(Value::Object(
                        from_str(v.as_str().unwrap())
                            .map_err(|_| err!("001", "invalid args $obj"))?,
                    ))
                } else {
                    Err(err!("001", "invalid args $obj"))
                }
            } else if map.contains_key("$arr") {
                if map.len() != 1 {
                    return Err(err!("001", "invalid args $arr"));
                }
                let v = &map["$arr"];
                if v.is_array() {
                    render_dollar(v, ref_from)
                } else if v.is_string() {
                    Ok(Value::Array(
                        from_str(v.as_str().unwrap())
                            .map_err(|_| err!("001", "invalid args $arr"))?,
                    ))
                } else {
                    Err(err!("001", "invalid args $arr"))
                }
            } else if map.contains_key("$ref") {
                if map.len() != 1 {
                    return Err(err!("001", "invalid args $ref"));
                }
                if map["$ref"].is_string() {
                    let ref_path = map["$ref"].as_str().unwrap();
                    let path: Vec<&str> = ref_path.split(".").collect();
                    let mut ref_val = ref_from;
                    for p in path {
                        ref_val = &ref_val[p];
                    }
                    Ok(ref_val.clone())
                } else {
                    Err(err!("001", "invalid args $ref"))
                }
            } else {
                let mut render_val = Map::new();
                for (k, v) in map {
                    render_val.insert(k.to_string(), render_dollar(v, ref_from)?);
                }
                Ok(Value::Object(render_val))
            }
        }
        Value::Array(arr) => {
            let mut arr_val: Vec<Value> = Vec::with_capacity(arr.len());
            for a in arr {
                arr_val.push(render_dollar(a, ref_from)?);
            }
            Ok(Value::Array(arr_val))
        }
        _ => Ok(val.clone()),
    };
}
