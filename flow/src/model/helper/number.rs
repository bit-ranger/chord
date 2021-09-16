use chord::value::{Number, Value};
use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};
use std::str::FromStr;

pub static NUM: NumHelper = NumHelper {};
pub static NUM_ADD: NumAddHelper = NumAddHelper {};

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

#[derive(Clone, Copy)]
pub struct NumAddHelper;

impl HelperDef for NumAddHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let p1 = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"num_add\""))?;
        let p2 = h
            .param(1)
            .ok_or_else(|| RenderError::new("Param not found for helper \"num_add\""))?;

        let p1 = p1.value();
        let p2 = p2.value();

        if p1.is_i64() && p2.is_i64() {
            let sum = p1.as_i64().unwrap() + p2.as_i64().unwrap();
            return Ok(Some(ScopedJson::Derived(Value::Number(Number::from(sum)))));
        }

        return Err(RenderError::new(format!(
            "num_add can not apply {} {}",
            p1, p2
        )));
    }
}
