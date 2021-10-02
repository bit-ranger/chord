use chord::value::{Number, Value};
use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};
use std::str::FromStr;

pub static NUM: NumHelper = NumHelper {};
pub static NUM_ADD: NumAddHelper = NumAddHelper {};
pub static NUM_SUB: NumSubHelper = NumSubHelper {};
pub static NUM_MUL: NumMulHelper = NumMulHelper {};
pub static NUM_DIV: NumDivHelper = NumDivHelper {};

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

        if !p1.is_number() || !p2.is_number() {
            return Err(RenderError::new(format!(
                "num_add can not apply {} {}",
                p1, p2
            )));
        }

        if p1.is_f64() || p2.is_f64() {
            let sum = p1.as_f64().unwrap() + p2.as_f64().unwrap();
            return Ok(Some(ScopedJson::Derived(Value::Number(
                Number::from_f64(sum)
                    .ok_or_else(|| RenderError::new("Return NaN for helper \"num_add\""))?,
            ))));
        }

        let sum = p1.as_i64().unwrap() + p2.as_i64().unwrap();
        return Ok(Some(ScopedJson::Derived(Value::Number(Number::from(sum)))));
    }
}

#[derive(Clone, Copy)]
pub struct NumSubHelper;

impl HelperDef for NumSubHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let p1 = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"num_sub\""))?;
        let p2 = h
            .param(1)
            .ok_or_else(|| RenderError::new("Param not found for helper \"num_sub\""))?;

        let p1 = p1.value();
        let p2 = p2.value();

        if !p1.is_number() || !p2.is_number() {
            return Err(RenderError::new(format!(
                "num_sub can not apply {} {}",
                p1, p2
            )));
        }

        if p1.is_f64() || p2.is_f64() {
            let sum = p1.as_f64().unwrap() - p2.as_f64().unwrap();
            return Ok(Some(ScopedJson::Derived(Value::Number(
                Number::from_f64(sum)
                    .ok_or_else(|| RenderError::new("Return NaN for helper \"num_sub\""))?,
            ))));
        }

        let sum = p1.as_i64().unwrap() - p2.as_i64().unwrap();
        return Ok(Some(ScopedJson::Derived(Value::Number(Number::from(sum)))));
    }
}

#[derive(Clone, Copy)]
pub struct NumMulHelper;

impl HelperDef for NumMulHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let p1 = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"num_mul\""))?;
        let p2 = h
            .param(1)
            .ok_or_else(|| RenderError::new("Param not found for helper \"num_mul\""))?;

        let p1 = p1.value();
        let p2 = p2.value();

        if !p1.is_number() || !p2.is_number() {
            return Err(RenderError::new(format!(
                "num_mul can not apply {} {}",
                p1, p2
            )));
        }

        if p1.is_f64() || p2.is_f64() {
            let sum = p1.as_f64().unwrap() * p2.as_f64().unwrap();
            return Ok(Some(ScopedJson::Derived(Value::Number(
                Number::from_f64(sum)
                    .ok_or_else(|| RenderError::new("Return NaN for helper \"num_mul\""))?,
            ))));
        }

        let sum = p1.as_i64().unwrap() * p2.as_i64().unwrap();
        return Ok(Some(ScopedJson::Derived(Value::Number(Number::from(sum)))));
    }
}

#[derive(Clone, Copy)]
pub struct NumDivHelper;

impl HelperDef for NumDivHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let p1 = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"num_dib\""))?;
        let p2 = h
            .param(1)
            .ok_or_else(|| RenderError::new("Param not found for helper \"num_dib\""))?;

        let p1 = p1.value();
        let p2 = p2.value();

        if !p1.is_number() || !p2.is_number() {
            return Err(RenderError::new(format!(
                "num_dib can not apply {} {}",
                p1, p2
            )));
        }

        if p1.is_f64() || p2.is_f64() {
            let sum = p1.as_f64().unwrap() / p2.as_f64().unwrap();
            return Ok(Some(ScopedJson::Derived(Value::Number(
                Number::from_f64(sum)
                    .ok_or_else(|| RenderError::new("Return NaN for helper \"num_dib\""))?,
            ))));
        }

        let sum = p1.as_i64().unwrap() / p2.as_i64().unwrap();
        return Ok(Some(ScopedJson::Derived(Value::Number(Number::from(sum)))));
    }
}
