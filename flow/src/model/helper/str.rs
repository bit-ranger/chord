use chord::value::{Number, Value};
use handlebars::{
    handlebars_helper, Context, Handlebars, Helper, HelperDef, RenderContext, RenderError,
    ScopedJson,
};

handlebars_helper!(START_WITH: |x: Json, y: Json|
    x.is_string() && y.is_string() && x.as_str().unwrap().starts_with(y.as_str().unwrap())
);

handlebars_helper!(END_WITH: |x: Json, y: Json|
    x.is_string() && y.is_string() && x.as_str().unwrap().ends_with(y.as_str().unwrap())
);

handlebars_helper!(CONTAINS: |x: Json, y: Json|{
    x.is_string() && y.is_string() && x.as_str().unwrap().contains(y.as_str().unwrap())
});

pub static STR: StrHelper = StrHelper {};
pub static LEN: LenHelper = LenHelper {};
pub static SUB: SubHelper = SubHelper {};
pub static ESCAPE: EscapeHelper = EscapeHelper {};

#[derive(Clone, Copy)]
pub struct StrHelper;

impl HelperDef for StrHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let param = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"str\""))?;

        let json = param.value().to_string();
        Ok(Some(ScopedJson::Derived(Value::String(json))))
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
        let param = h.params();
        let str = param[0]
            .value()
            .as_str()
            .ok_or(RenderError::new("Param invalid for helper \"str_sub\""))?;

        if param.len() == 2 {
            let start = param[1]
                .value()
                .as_f64()
                .ok_or(RenderError::new("Param invalid for helper \"str_sub\""))?
                as usize;
            return Ok(Some(ScopedJson::Derived(Value::String(
                str[start..].to_owned(),
            ))));
        } else if param.len() == 3 {
            let start = param[1]
                .value()
                .as_f64()
                .ok_or(RenderError::new("Param invalid for helper \"str_sub\""))?
                as usize;
            let end = param[2]
                .value()
                .as_f64()
                .ok_or(RenderError::new("Param invalid for helper \"str_sub\""))?
                as usize;
            return Ok(Some(ScopedJson::Derived(Value::String(
                str[start..end].to_owned(),
            ))));
        } else {
            return Err(RenderError::new("Param invalid for helper \"str_sub\""));
        }
    }
}

#[derive(Clone, Copy)]
pub struct EscapeHelper;

impl HelperDef for EscapeHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let param = h.params();
        let str = param[0]
            .value()
            .as_str()
            .ok_or(RenderError::new("Param invalid for helper \"str_escape\""))?;

        return Ok(Some(ScopedJson::Derived(Value::String(
            str.escape_debug().to_string(),
        ))));
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
        let param = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"str_len\""))?;

        match param.value() {
            Value::String(txt) => Ok(Some(ScopedJson::Derived(Value::Number(Number::from(
                txt.len(),
            ))))),
            _ => Err(RenderError::new("Param invalid for helper \"str_len\"")),
        }
    }
}
