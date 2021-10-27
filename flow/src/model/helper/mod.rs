use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};
use jsonpath_rust::JsonPathFinder;

use chord::value::Value;

use crate::model::helper::boolean::{ALL, ANY, BOOL};

mod array;
mod boolean;
mod number;
mod string;

pub fn register(handlebars: &mut Handlebars) {
    //handlebars-3.5.4/src/registry.rs:118
    handlebars.register_helper("all", Box::new(ALL));
    handlebars.register_helper("any", Box::new(ANY));
    handlebars.register_helper("bool", Box::new(BOOL));

    //literal
    handlebars.register_helper(
        "bl",
        Box::new(LiteralHelper {
            literal: "{".into(),
        }),
    );
    handlebars.register_helper(
        "br",
        Box::new(LiteralHelper {
            literal: "}".into(),
        }),
    );

    handlebars.register_helper("jsonpath", Box::new(JsonpathHelper {}));

    //number
    handlebars.register_helper("num", Box::new(crate::model::helper::number::NUM));
    handlebars.register_helper("num_add", Box::new(crate::model::helper::number::NUM_ADD));
    handlebars.register_helper("num_sub", Box::new(crate::model::helper::number::NUM_SUB));
    handlebars.register_helper("num_mul", Box::new(crate::model::helper::number::NUM_MUL));
    handlebars.register_helper("num_div", Box::new(crate::model::helper::number::NUM_DIV));

    //array
    handlebars.register_helper(
        "arr_contains",
        Box::new(crate::model::helper::array::contains),
    );
    handlebars.register_helper("arr_sub", Box::new(crate::model::helper::array::SUB));
    handlebars.register_helper("arr_len", Box::new(crate::model::helper::array::LEN));
    handlebars.register_helper("arr_get", Box::new(crate::model::helper::array::GET));

    //string
    handlebars.register_helper("str", Box::new(crate::model::helper::string::STR));
    handlebars.register_helper(
        "str_parse_json",
        Box::new(crate::model::helper::string::PARSE_JSON),
    );
    handlebars.register_helper(
        "str_contains",
        Box::new(crate::model::helper::string::contains),
    );
    handlebars.register_helper(
        "str_start_with",
        Box::new(crate::model::helper::string::start_with),
    );
    handlebars.register_helper(
        "str_end_with",
        Box::new(crate::model::helper::string::end_with),
    );
    handlebars.register_helper("str_sub", Box::new(crate::model::helper::string::SUB));
    handlebars.register_helper("str_len", Box::new(crate::model::helper::string::LEN));
    handlebars.register_helper("str_escape", Box::new(crate::model::helper::string::ESCAPE));
}

pub struct LiteralHelper {
    literal: String,
}

impl HelperDef for LiteralHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        _: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        Ok(Some(ScopedJson::Derived(Value::String(
            self.literal.to_string(),
        ))))
    }
}

pub struct JsonpathHelper {}

impl HelperDef for JsonpathHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let params = h.params();
        if params.len() != 2 {
            return Err(RenderError::new("Param invalid for helper \"jsonpath\""));
        }

        let value = params[0].value();

        let path = params[1]
            .value()
            .as_str()
            .ok_or(RenderError::new("Param invalid for helper \"jsonpath\""))?;

        let mut finder = JsonPathFinder::from_str("null", path).map_err(|e| {
            RenderError::new(format!("Param invalid for helper \"jsonpath\": {}", e))
        })?;
        finder.set_json(value.clone());

        let find = finder.find();
        Ok(Some(ScopedJson::Derived(find)))
    }
}
