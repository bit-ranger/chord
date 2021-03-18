use std::str::FromStr;

use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};
use serde_json::Value;

use common::value::{Json, Number};

pub static NUM_HELPER: NumHelper = NumHelper { };
pub static BOOL_HELPER: BoolHelper = BoolHelper { };

#[derive(Clone, Copy)]
pub struct NumHelper;

impl HelperDef for NumHelper {

    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let param = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"num\""))?;

        let param = param.value();

        match param {
            Value::String(x) => {
                let n = Number::from_str(x.trim()).unwrap();
                Ok(Some(ScopedJson::Derived(Json::Number(n))))
            },
            Value::Number(n) => Ok(Some(ScopedJson::Derived(Json::Number(n.clone())))),
            _ => Err(RenderError::new("\"num\" can not convert "))
        }

    }
}


#[derive(Clone, Copy)]
pub struct BoolHelper;

impl HelperDef for BoolHelper {

    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let param = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"num\""))?;

        let param = param.value();

        match param {
            Value::String(x) => {
                let n = bool::from_str(x.trim()).unwrap();
                Ok(Some(ScopedJson::Derived(Json::Bool(n))))
            },
            Value::Bool(n) => Ok(Some(ScopedJson::Derived(Json::Bool(n.clone())))),
            _ => Err(RenderError::new("\"bool\" can not convert "))
        }

    }
}