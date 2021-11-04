use chord::value::{from_str, Number, Value};
use handlebars::handlebars_helper;
use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};

handlebars_helper!(CONTAINS: |x: Json, y: Json|{
    x.is_array() && x.as_array().unwrap().contains(y)
});

pub static ARR: ArrHelper = ArrHelper {};
pub static LEN: LenHelper = LenHelper {};
pub static SUB: SubHelper = SubHelper {};
pub static GET: GetHelper = GetHelper {};

#[derive(Clone, Copy)]
pub struct ArrHelper {}

impl HelperDef for ArrHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let param = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"arr\""))?;

        match param.value() {
            Value::String(txt) => Ok(Some(ScopedJson::Derived(Value::Array(from_str(txt)?)))),
            Value::Array(arr) => Ok(Some(ScopedJson::Derived(Value::Array(arr.clone())))),
            _ => Err(RenderError::new("\"arr\" can not convert ")),
        }
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
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let params = h.params();
        let arr = params[0]
            .value()
            .as_array()
            .ok_or(RenderError::new("Param invalid for helper \"arr_sub\""))?;

        if params.len() == 2 {
            let start = params[1]
                .value()
                .as_f64()
                .ok_or(RenderError::new("Param invalid for helper \"arr_sub\""))?
                as usize;
            let mut a = Vec::<Value>::new();
            a.clone_from_slice(&arr[start..]);
            return Ok(Some(ScopedJson::Derived(Value::Array(a))));
        } else if params.len() == 3 {
            let start = params[1]
                .value()
                .as_f64()
                .ok_or(RenderError::new("Param invalid for helper \"arr_sub\""))?
                as usize;
            let end = params[2]
                .value()
                .as_f64()
                .ok_or(RenderError::new("Param invalid for helper \"arr_sub\""))?
                as usize;
            let mut a = Vec::<Value>::new();
            a.clone_from_slice(&arr[start..end]);
            return Ok(Some(ScopedJson::Derived(Value::Array(a))));
        } else {
            return Err(RenderError::new("Param invalid for helper \"arr_sub\""));
        }
    }
}

#[derive(Clone, Copy)]
pub struct LenHelper {}

impl HelperDef for LenHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let params = h.params();
        let arr = params[0]
            .value()
            .as_array()
            .ok_or(RenderError::new("Param invalid for helper \"arr_len\""))?;

        Ok(Some(ScopedJson::Derived(Value::Number(Number::from(
            arr.len(),
        )))))
    }
}

#[derive(Clone, Copy)]
pub struct GetHelper {}

impl HelperDef for GetHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let params = h.params();
        let arr = params[0]
            .value()
            .as_array()
            .ok_or(RenderError::new("Param invalid for helper \"arr_get\""))?;

        if params.len() == 2 {
            let start = params[1]
                .value()
                .as_f64()
                .ok_or(RenderError::new("Param invalid for helper \"arr_get\""))?
                as usize;
            let result = if arr.len() > 0 {
                arr[start].clone()
            } else {
                Value::Null
            };
            return Ok(Some(ScopedJson::Derived(result)));
        } else {
            return Err(RenderError::new("Param invalid for helper \"arr_get\""));
        }
    }
}
