use chord_common::err;
use chord_common::error::Error;
use chord_common::point::{async_trait, RunArg, PointRunner, PointValue, CreateArg};
use chord_common::value::Json;
use libloading::Library;

struct Dynlib {
    lib: Library,
}

#[async_trait]
impl PointRunner for Dynlib {
    async fn run(&self, arg: &dyn RunArg) -> PointValue {
        let args_raw = arg.config()["args"]
            .as_array()
            .ok_or(err!("010", "missing args"))?;

        let mut args_rendered = vec![];

        for ar in args_raw {
            let x = arg.render(ar.as_str().ok_or(err!("010", "arg must be string"))?)?;
            args_rendered.push(x);
        }

        let args_dynlib: Vec<&str> = args_rendered.iter().map(|a| a.as_str()).collect();

        let dynlib_run: libloading::Symbol<fn(Vec<&str>) -> PointValue> =
            unsafe { self.lib.get(b"run")? };
        dynlib_run(args_dynlib)
    }
}

pub async fn create(_: Option<&Json>, arg: &dyn CreateArg) -> Result<Box<dyn PointRunner>, Error> {
    let path = arg.config()["path"]
        .as_str()
        .ok_or(err!("010", "missing path"))?;

    let lib = unsafe { libloading::Library::new(path)? };

    Ok(Box::new(Dynlib { lib }))
}