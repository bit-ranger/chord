use handlebars::handlebars_helper;
use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};

use chord::value::{to_string_pretty, Value};

handlebars_helper!(start_with: |x: Json, y: Json|
    x.is_string() && y.is_string() && x.as_str().unwrap().starts_with(y.as_str().unwrap())
);

handlebars_helper!(end_with: |x: Json, y: Json|
    x.is_string() && y.is_string() && x.as_str().unwrap().ends_with(y.as_str().unwrap())
);

handlebars_helper!(contains: |x: Json, y: Json|{
    x.is_string() && y.is_string() && x.as_str().unwrap().contains(y.as_str().unwrap())
});

pub static STR_HELPER: StrHelper = StrHelper {};
pub static SUBSTRING_HELPER: SubStringHelper = SubStringHelper {};

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
        let txt = to_string_pretty(param)?;
        Ok(Some(ScopedJson::Derived(Value::String(txt))))
    }
}

#[derive(Clone, Copy)]
pub struct SubStringHelper;

impl HelperDef for SubStringHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let param = h.params();
        let str = param[0].value().as_str().ok_or(RenderError::new(
            "Param invalid for helper \"str_substring\"",
        ))?;

        if param.len() == 2 {
            let start = param[1].value().as_f64().ok_or(RenderError::new(
                "Param invalid for helper \"str_substring\"",
            ))? as usize;
            return Ok(Some(ScopedJson::Derived(Value::String(
                str[start..].to_owned(),
            ))));
        } else if param.len() == 3 {
            let start = param[1].value().as_f64().ok_or(RenderError::new(
                "Param invalid for helper \"str_substring\"",
            ))? as usize;
            let end = param[2].value().as_f64().ok_or(RenderError::new(
                "Param invalid for helper \"str_substring\"",
            ))? as usize;
            return Ok(Some(ScopedJson::Derived(Value::String(
                str[start..end].to_owned(),
            ))));
        } else {
            return Err(RenderError::new(
                "Param invalid for helper \"str_substring\"",
            ));
        }
    }
}
