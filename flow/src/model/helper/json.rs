use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};
use jsonpath_rust::JsonPathFinder;

pub static JSON: JsonHelper = JsonHelper {};
pub static PATH: PathHelper = PathHelper {};

#[derive(Clone, Copy)]
pub struct JsonHelper {}

impl HelperDef for JsonHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let param = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"json\""))?;

        Ok(Some(ScopedJson::Derived(param.value().clone())))
    }
}

#[derive(Clone, Copy)]
pub struct PathHelper {}

impl HelperDef for PathHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let params = h.params();
        if params.len() != 2 {
            return Err(RenderError::new("Param invalid for helper \"json_path\""));
        }

        let value = params[0].value();

        let path = params[1]
            .value()
            .as_str()
            .ok_or(RenderError::new("Param invalid for helper \"json_path\""))?;

        let mut finder = JsonPathFinder::from_str("null", path).map_err(|e| {
            RenderError::new(format!("Param invalid for helper \"json_path\": {}", e))
        })?;
        finder.set_json(Box::new(value.clone()));

        let find = finder.find();
        Ok(Some(ScopedJson::Derived(find)))
    }
}
