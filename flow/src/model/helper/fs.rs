use chord::value::Value;
use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};
use std::fs::read_to_string;
use std::path::PathBuf;

pub static FILE: FileHelper = FileHelper {};

#[derive(Clone, Copy)]
pub struct FileHelper;

impl HelperDef for FileHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let param = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"file\""))?;

        let param = param.value();

        match param {
            Value::String(x) => {
                let task_dir = ctx.data()["def"]["__meta__"]["task_dir"]
                    .as_str()
                    .ok_or_else(|| RenderError::new("Param not found for helper \"file\""))?;
                let mut file_path = PathBuf::from(task_dir);
                file_path.push(x);
                let file_string = read_to_string(file_path).map_err(|e| {
                    RenderError::new(format!("Failed for helper \"file\", cause {}", e))
                })?;
                Ok(Some(ScopedJson::Derived(Value::String(file_string))))
            }
            _ => Err(RenderError::new("Param invalid for helper \"file\"")),
        }
    }
}
