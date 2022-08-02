use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};

use chord_core::value::{from_str, Number, Value};

pub static NUM: NumHelper = NumHelper {};
pub static ADD: AddHelper = AddHelper {};
pub static SUB: SubHelper = SubHelper {};
pub static MUL: MulHelper = MulHelper {};
pub static DIV: DivHelper = DivHelper {};

#[derive(Clone, Copy)]
pub struct NumHelper;

impl HelperDef for NumHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'reg, 'rc>, RenderError> {
        let param = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"num\""))?;

        let param = param.value();

        match param {
            Value::String(x) => Ok(ScopedJson::Derived(Value::Number(from_str(x.trim())?))),
            Value::Number(n) => Ok(ScopedJson::Derived(Value::Number(n.clone()))),
            _ => Err(RenderError::new("\"num\" can not convert ")),
        }
    }
}

#[derive(Clone, Copy)]
pub struct AddHelper;

impl HelperDef for AddHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'reg, 'rc>, RenderError> {
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
            return Ok(ScopedJson::Derived(Value::Number(
                Number::from_f64(sum)
                    .ok_or_else(|| RenderError::new("Return NaN for helper \"num_add\""))?,
            )));
        }

        let sum = p1.as_i64().unwrap() + p2.as_i64().unwrap();
        return Ok(ScopedJson::Derived(Value::Number(Number::from(sum))));
    }
}

#[derive(Clone, Copy)]
pub struct SubHelper;

impl HelperDef for SubHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'reg, 'rc>, RenderError> {
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
            return Ok(ScopedJson::Derived(Value::Number(
                Number::from_f64(sum)
                    .ok_or_else(|| RenderError::new("Return NaN for helper \"num_sub\""))?,
            )));
        }

        let sum = p1.as_i64().unwrap() - p2.as_i64().unwrap();
        return Ok(ScopedJson::Derived(Value::Number(Number::from(sum))));
    }
}

#[derive(Clone, Copy)]
pub struct MulHelper;

impl HelperDef for MulHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'reg, 'rc>, RenderError> {
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
            return Ok(ScopedJson::Derived(Value::Number(
                Number::from_f64(sum)
                    .ok_or_else(|| RenderError::new("Return NaN for helper \"num_mul\""))?,
            )));
        }

        let sum = p1.as_i64().unwrap() * p2.as_i64().unwrap();
        return Ok(ScopedJson::Derived(Value::Number(Number::from(sum))));
    }
}

#[derive(Clone, Copy)]
pub struct DivHelper;

impl HelperDef for DivHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'reg, 'rc>, RenderError> {
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
            return Ok(ScopedJson::Derived(Value::Number(
                Number::from_f64(sum)
                    .ok_or_else(|| RenderError::new("Return NaN for helper \"num_dib\""))?,
            )));
        }

        let sum = p1.as_i64().unwrap() / p2.as_i64().unwrap();
        return Ok(ScopedJson::Derived(Value::Number(Number::from(sum))));
    }
}
