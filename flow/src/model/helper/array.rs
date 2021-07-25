use chord::value::{Number, Value};
use handlebars::handlebars_helper;
use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};

handlebars_helper!(contains: |x: Json, y: Json|{
    x.is_array() && x.as_array().unwrap().contains(y)
});

pub static LEN: LenHelper = LenHelper {};
pub static SUB: SubHelper = SubHelper {};
pub static GET: GetHelper = GetHelper {};

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
            return Ok(Some(ScopedJson::Derived(arr[start].clone())));
        } else {
            return Err(RenderError::new("Param invalid for helper \"arr_get\""));
        }
    }
}
