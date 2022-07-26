use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};

use chord_core::value::Value;

mod arr;
mod bool;
mod fs;
mod json;
mod num;
mod obj;
mod str;

pub fn register(handlebars: &mut Handlebars) {
    //handlebars-3.5.4/src/registry.rs:118
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
    handlebars.register_helper("json", Box::new(json::JSON));
    handlebars.register_helper("json_path", Box::new(json::PATH));

    //object
    handlebars.register_helper("obj", Box::new(obj::OBJ));
    handlebars.register_helper("obj_contains_key", Box::new(obj::OBJ_CONTAINS_KEY));

    // bool
    handlebars.register_helper("bool", Box::new(bool::BOOL));
    handlebars.register_helper("all", Box::new(bool::ALL));
    handlebars.register_helper("any", Box::new(bool::ANY));

    //number
    handlebars.register_helper("num", Box::new(num::NUM));
    handlebars.register_helper("num_add", Box::new(num::ADD));
    handlebars.register_helper("num_sub", Box::new(num::SUB));
    handlebars.register_helper("num_mul", Box::new(num::MUL));
    handlebars.register_helper("num_div", Box::new(num::DIV));

    //array
    handlebars.register_helper("arr", Box::new(arr::ARR));
    handlebars.register_helper("arr_contains", Box::new(arr::CONTAINS));
    handlebars.register_helper("arr_sub", Box::new(arr::SUB));
    handlebars.register_helper("arr_len", Box::new(arr::LEN));
    handlebars.register_helper("arr_get", Box::new(arr::GET));

    //string
    handlebars.register_helper("str", Box::new(str::STR));
    handlebars.register_helper("str_sub", Box::new(str::SUB));
    handlebars.register_helper("str_len", Box::new(str::LEN));
    handlebars.register_helper("str_escape", Box::new(str::ESCAPE));
    handlebars.register_helper("str_contains", Box::new(str::CONTAINS));
    handlebars.register_helper("str_start_with", Box::new(str::START_WITH));
    handlebars.register_helper("str_end_with", Box::new(str::END_WITH));

    //fs
    handlebars.register_helper("fs_read", Box::new(fs::READ));
    handlebars.register_helper("fs_path", Box::new(fs::PATH));
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
    ) -> Result<ScopedJson<'reg, 'rc>, RenderError> {
        Ok(ScopedJson::Derived(Value::String(self.literal.to_string())))
    }
}
