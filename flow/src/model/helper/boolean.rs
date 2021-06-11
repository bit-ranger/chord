use chord::value::Value;
use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};
use std::str::FromStr;

pub static BOOL_HELPER: BoolHelper = BoolHelper {};
pub static ALL_HELPER: AllHelper = AllHelper {};
pub static ANY_HELPER: AnyHelper = AnyHelper {};

#[derive(Clone, Copy)]
pub struct BoolHelper;

impl HelperDef for BoolHelper {
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
                let n = bool::from_str(x.trim()).unwrap();
                Ok(Some(ScopedJson::Derived(Value::Bool(n))))
            }
            Value::Bool(n) => Ok(Some(ScopedJson::Derived(Value::Bool(n.clone())))),
            _ => Err(RenderError::new("\"bool\" can not convert ")),
        }
    }
}

#[derive(Clone, Copy)]
pub struct AllHelper;

impl HelperDef for AllHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let mut idx = 0;
        loop {
            let param = h.param(idx);
            if param.is_none() {
                if idx == 0 {
                    return Err(RenderError::new("Param not found for helper \"all\""));
                }
                break;
            }

            let param = param.unwrap().value();

            match param {
                Value::Bool(b) => {
                    if !b {
                        return Ok(Some(ScopedJson::Derived(Value::Bool(false))));
                    }
                }
                _ => return Err(RenderError::new("\"all\" only accept bool")),
            }

            idx = idx + 1;
        }

        return Ok(Some(ScopedJson::Derived(Value::Bool(true))));
    }
}

#[derive(Clone, Copy)]
pub struct AnyHelper;

impl HelperDef for AnyHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let mut idx = 0;
        loop {
            let param = h.param(idx);
            if param.is_none() {
                if idx == 0 {
                    return Err(RenderError::new("Param not found for helper \"any\""));
                }
                break;
            }

            let param = param.unwrap().value();

            match param {
                Value::Bool(b) => {
                    if *b {
                        return Ok(Some(ScopedJson::Derived(Value::Bool(true))));
                    }
                }
                _ => return Err(RenderError::new("\"any\" only accept bool")),
            }

            idx = idx + 1;
        }

        return Ok(Some(ScopedJson::Derived(Value::Bool(false))));
    }
}
