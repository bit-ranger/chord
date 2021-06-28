use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyDict};

use chord::action::prelude::*;
use chord::value::{from_str, Map};

pub struct PythonFactory {}

impl PythonFactory {
    pub async fn new(_: Option<Value>) -> Result<PythonFactory, Error> {
        Ok(PythonFactory {})
    }
}

#[async_trait]
impl Factory for PythonFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Python {}))
    }
}

struct Python {}

#[async_trait]
impl Action for Python {
    async fn run(&self, arg: &dyn RunArg) -> ActionValue {
        pyo3::prelude::Python::with_gil(|py| {
            let code = arg.args()["code"]
                .as_str()
                .ok_or(err!("python", "missing code"))?;

            let vars = arg
                .render_value(&arg.args()["vars"])?
                .as_object()
                .map(|m| m.to_owned())
                .unwrap_or(Map::new());

            self.eval(py, code, vars)
        })
    }
}

impl Python {
    fn eval(&self, py: pyo3::prelude::Python, code: &str, vars: Map) -> ActionValue {
        let locals = PyDict::new(py);
        for (k, v) in vars {
            collect(locals, k.as_str(), v)?
        }
        let value: String = py.eval(code, None, Some(&locals))?.extract()?;
        Ok(from_str(value.as_str())?)
    }
}

fn collect(dict: &PyDict, k: &str, vars: Value) -> Result<(), Error> {
    match vars {
        Value::String(v) => dict.set_item(k, v)?,
        Value::Number(v) => {
            if v.is_f64() {
                dict.set_item(k, v.as_f64())?
            } else if v.is_u64() {
                dict.set_item(k, v.as_u64())?
            } else if v.is_i64() {
                dict.set_item(k, v.as_i64())?
            }
        }
        // Value::Array(vec) => {
        //     for v in vec {
        //         collect(dict, v)?
        //     }
        // }
        Value::Object(m) => {
            let dict0 = dict.copy()?;
            dict0.clear();
            for (k0, v0) in m {
                collect(dict0, k0.as_str(), v0)?
            }
            dict.set_item(k, dict0)?
        }
        _ => {}
    }

    Ok(())
}
