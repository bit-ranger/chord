use chord::value::Value;
use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};

use crate::model::helper::bool::{ALL, ANY, BOOL};

mod arr;
mod bool;
mod fs;
mod json;
mod num;
mod obj;
mod str;

pub fn register(handlebars: &mut Handlebars) {
    //handlebars-3.5.4/src/registry.rs:118
    handlebars.register_helper("ref", Box::new(RefHelper {}));

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

    //json
    handlebars.register_helper("json", Box::new(crate::model::helper::json::JSON));
    handlebars.register_helper("json_path", Box::new(crate::model::helper::json::PATH));

    //object
    handlebars.register_helper("obj", Box::new(crate::model::helper::obj::OBJ));

    // bool
    handlebars.register_helper("bool", Box::new(BOOL));
    handlebars.register_helper("all", Box::new(ALL));
    handlebars.register_helper("any", Box::new(ANY));

    //number
    handlebars.register_helper("num", Box::new(crate::model::helper::num::NUM));
    handlebars.register_helper("num_add", Box::new(crate::model::helper::num::ADD));
    handlebars.register_helper("num_sub", Box::new(crate::model::helper::num::SUB));
    handlebars.register_helper("num_mul", Box::new(crate::model::helper::num::MUL));
    handlebars.register_helper("num_div", Box::new(crate::model::helper::num::DIV));

    //array
    handlebars.register_helper("arr", Box::new(crate::model::helper::arr::ARR));
    handlebars.register_helper(
        "arr_contains",
        Box::new(crate::model::helper::arr::CONTAINS),
    );
    handlebars.register_helper("arr_sub", Box::new(crate::model::helper::arr::SUB));
    handlebars.register_helper("arr_len", Box::new(crate::model::helper::arr::LEN));
    handlebars.register_helper("arr_get", Box::new(crate::model::helper::arr::GET));

    //string
    handlebars.register_helper("str", Box::new(crate::model::helper::str::STR));
    handlebars.register_helper("str_sub", Box::new(crate::model::helper::str::SUB));
    handlebars.register_helper("str_len", Box::new(crate::model::helper::str::LEN));
    handlebars.register_helper("str_escape", Box::new(crate::model::helper::str::ESCAPE));
    handlebars.register_helper(
        "str_contains",
        Box::new(crate::model::helper::str::CONTAINS),
    );
    handlebars.register_helper(
        "str_start_with",
        Box::new(crate::model::helper::str::START_WITH),
    );
    handlebars.register_helper(
        "str_end_with",
        Box::new(crate::model::helper::str::END_WITH),
    );

    //fs
    handlebars.register_helper("fs_read", Box::new(crate::model::helper::fs::READ));
    handlebars.register_helper("fs_path", Box::new(crate::model::helper::fs::PATH));
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

#[derive(Clone, Copy)]
pub struct RefHelper {}

impl HelperDef for RefHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let param = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"ref\""))?;

        Ok(Some(ScopedJson::Derived(param.value().clone())))
    }
}
