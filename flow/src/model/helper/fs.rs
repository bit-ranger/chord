use std::fs::canonicalize;
use std::fs::read_to_string;
use std::path::PathBuf;

use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};

use chord_core::value::Value;

pub static READ: ReadHelper = ReadHelper {};
pub static PATH: PathHelper = PathHelper {};

#[derive(Clone, Copy)]
pub struct ReadHelper;

impl HelperDef for ReadHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let param = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"fs_read\""))?;

        let param = param.value();

        match param {
            Value::String(x) => {
                let task_dir = ctx.data()["__meta__"]["task_dir"]
                    .as_str()
                    .ok_or_else(|| RenderError::new("Param invalid for helper \"fs_read\""))?;
                let mut file_path = PathBuf::from(task_dir);
                file_path.push(x);
                file_path = canonicalize(file_path.as_path())
                    .map_err(|_| RenderError::new("Param invalid for helper \"fs_read\""))?;
                let file_string = read_to_string(file_path).map_err(|e| {
                    RenderError::new(format!("Failed for helper \"fs_read\", cause {}", e))
                })?;
                Ok(Some(ScopedJson::Derived(Value::String(file_string))))
            }
            _ => Err(RenderError::new("Param invalid for helper \"fs_read\"")),
        }
    }
}

#[derive(Clone, Copy)]
pub struct PathHelper;

impl HelperDef for PathHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let param = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"fs_path\""))?;

        let param = param.value();

        match param {
            Value::String(x) => {
                let task_dir = ctx.data()["__meta__"]["task_dir"]
                    .as_str()
                    .ok_or_else(|| RenderError::new("Param invalid for helper \"fs_path\""))?;
                let mut file_path = PathBuf::from(task_dir);
                file_path.push(x);
                file_path = canonicalize(file_path.as_path())
                    .map_err(|_| RenderError::new("Param invalid for helper \"fs_path\""))?;
                let path_string = file_path
                    .to_str()
                    .ok_or_else(|| RenderError::new("Failed for helper \"fs_path\""))?;
                Ok(Some(ScopedJson::Derived(Value::String(
                    path_string.to_string(),
                ))))
            }
            _ => Err(RenderError::new("Param invalid for helper \"fs_path\"")),
        }
    }
}
