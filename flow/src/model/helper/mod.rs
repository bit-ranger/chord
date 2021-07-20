use crate::model::helper::boolean::{ALL_HELPER, ANY_HELPER, BOOL_HELPER};
use chord::value::Value;
use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};

mod array;
mod boolean;
mod json;
mod number;
mod string;

pub fn register(handlebars: &mut Handlebars) {
    //handlebars-3.5.4/src/registry.rs:118
    handlebars.register_helper("all", Box::new(ALL_HELPER));
    handlebars.register_helper("any", Box::new(ANY_HELPER));
    handlebars.register_helper("bool", Box::new(BOOL_HELPER));

    //literal
    handlebars.register_helper(
        "lbrace",
        Box::new(LiteralHelper {
            literal: "{".into(),
        }),
    );
    handlebars.register_helper(
        "rbrace",
        Box::new(LiteralHelper {
            literal: "}".into(),
        }),
    );

    //json
    handlebars.register_helper("json", Box::new(crate::model::helper::json::JSON_HELPER));

    //number
    handlebars.register_helper("num", Box::new(crate::model::helper::number::NUM_HELPER));

    //array
    handlebars.register_helper(
        "arr_contains",
        Box::new(crate::model::helper::array::contains),
    );

    //string
    handlebars.register_helper("str", Box::new(crate::model::helper::string::STR_HELPER));
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
    handlebars.register_helper(
        "str_substring",
        Box::new(crate::model::helper::string::SUBSTRING_HELPER),
    );
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
