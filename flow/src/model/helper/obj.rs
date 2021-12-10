use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};

use chord_core::value::{from_str, Value};

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
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let param = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"obj\""))?;

        match param.value() {
            Value::String(txt) => Ok(Some(ScopedJson::Derived(Value::Object(from_str(txt)?)))),
            Value::Object(obj) => Ok(Some(ScopedJson::Derived(Value::Object(obj.clone())))),
            _ => Err(RenderError::new("\"obj\" can not convert ")),
        }
    }
}
