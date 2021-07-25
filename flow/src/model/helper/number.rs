use chord::value::{Number, Value};
use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};
use std::str::FromStr;

pub static NUM: NumHelper = NumHelper {};

#[derive(Clone, Copy)]
pub struct NumHelper;

impl HelperDef for NumHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let param = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"num\""))?;

        let param = param.value();

        match param {
            Value::String(x) => {
                let n = Number::from_str(x.trim()).unwrap();
                Ok(Some(ScopedJson::Derived(Value::Number(n))))
            }
            Value::Number(n) => Ok(Some(ScopedJson::Derived(Value::Number(n.clone())))),
            _ => Err(RenderError::new("\"num\" can not convert ")),
        }
    }
}
