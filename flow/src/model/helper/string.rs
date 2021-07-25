use chord::value::{Number, Value};
use handlebars::handlebars_helper;
use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};
use snailquote::unescape;

handlebars_helper!(start_with: |x: Json, y: Json|
    x.is_string() && y.is_string() && x.as_str().unwrap().starts_with(y.as_str().unwrap())
);

handlebars_helper!(end_with: |x: Json, y: Json|
    x.is_string() && y.is_string() && x.as_str().unwrap().ends_with(y.as_str().unwrap())
);

handlebars_helper!(contains: |x: Json, y: Json|{
    x.is_string() && y.is_string() && x.as_str().unwrap().contains(y.as_str().unwrap())
});

pub static STR: StrHelper = StrHelper {};
pub static LEN: LenHelper = LenHelper {};
pub static SUB: SubHelper = SubHelper {};
pub static ESCAPE: EscapeHelper = EscapeHelper {};
pub static UNESCAPE: UnescapeHelper = UnescapeHelper {};

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

        let param = param.value();
        Ok(Some(ScopedJson::Derived(Value::String(param.to_string()))))
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
pub struct UnescapeHelper;

impl HelperDef for UnescapeHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let param = h.params();
        let str = param[0].value().as_str().ok_or(RenderError::new(
            "Param invalid for helper \"str_unescape\"",
        ))?;
        let txt = unescape(str)
            .map_err(|_| RenderError::new("Param invalid for helper \"str_unescape\""))?;
        return Ok(Some(ScopedJson::Derived(Value::String(txt))));
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
