use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};
use itertools::Itertools;

use chord::value::Value;

pub static JOIN_HELPER: JoinHelper = JoinHelper {};

#[derive(Clone, Copy)]
pub struct JoinHelper;

impl HelperDef for JoinHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let param = h.params();
        if param.len() == 0 {
            return Err(RenderError::new("Param not found for helper \"join\""));
        }

        let sep = param[0].value().to_string();

        let str_join = param[1..]
            .iter()
            .map(|s| s.value().to_string())
            .join(sep.as_str());

        return Ok(Some(ScopedJson::Derived(Value::String(str_join))));
    }
}
