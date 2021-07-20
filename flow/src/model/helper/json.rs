use handlebars::{Context, Handlebars, Helper, HelperDef, Output, RenderContext, RenderError};

pub static JSON_HELPER: JsonHelper = JsonHelper {};

#[derive(Clone, Copy)]
pub struct JsonHelper {}

impl HelperDef for JsonHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> Result<(), RenderError> {
        let param = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"json\""))?;

        let json = param.value().to_string();
        out.write(json.as_ref())?;
        Ok(())
    }
}
