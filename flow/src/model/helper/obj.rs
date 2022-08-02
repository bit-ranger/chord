use handlebars::handlebars_helper;
use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};

use chord_core::value::{from_str, Value};

handlebars_helper!(OBJ_CONTAINS_KEY: |x: Json, y: Json|{
    x.is_object() && y.is_string() && x.as_object().unwrap().contains_key(y.as_str().unwrap())
});

pub static OBJ: ObjHelper = ObjHelper {};

#[derive(Clone, Copy)]
pub struct ObjHelper {}

impl HelperDef for ObjHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'reg, 'rc>, RenderError> {
        let param = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"obj\""))?;

        match param.value() {
            Value::String(txt) => Ok(ScopedJson::Derived(Value::Object(from_str(txt)?))),
            Value::Object(obj) => Ok(ScopedJson::Derived(Value::Object(obj.clone()))),
            _ => Err(RenderError::new("\"obj\" can not convert ")),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ObjContainsKeyHelper {}

impl HelperDef for ObjContainsKeyHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'reg, 'rc>, RenderError> {
        let obj = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"obj_contains_key\""))?;

        let key = h
            .param(1)
            .ok_or_else(|| RenderError::new("Param not found for helper \"obj_contains_key\""))?;

        match obj.value() {
            Value::Object(obj) => match key.value() {
                Value::String(s) => Ok(ScopedJson::Derived(Value::Bool(obj.contains_key(s)))),
                _ => Err(RenderError::new(
                    "Param invalid for helper \"obj_contains_key\"",
                )),
            },
            _ => Err(RenderError::new(
                "Param invalid for helper \"obj_contains_key\"",
            )),
        }
    }
}
