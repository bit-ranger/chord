use std::collections::HashMap;

use pyo3::conversion::ToPyObject;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyInt, PyLong, PyString};

use chord::action::prelude::*;
use chord::value::{Map, Number};

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
            let obj = to_py_obj(v, py);
            locals.set_item(k, obj)?;
        }
        let value = py.eval(code, None, Some(&locals))?;
        if value.is_instance::<PyLong>()? {
            return Ok(Value::Number(Number::from(value.extract::<i64>().unwrap())));
        } else if value.is_instance::<PyInt>()? {
            return Ok(Value::Number(Number::from(value.extract::<i32>().unwrap())));
        } else if value.is_instance::<PyBool>()? {
            return Ok(Value::Bool(value.extract::<bool>().unwrap()));
        } else if value.is_instance::<PyString>()? {
            return Ok(Value::String(value.extract::<String>().unwrap()));
        }

        Ok(Value::Null)
    }
}

fn to_py_obj(vars: Value, py: pyo3::prelude::Python) -> PyObject {
    match vars {
        Value::String(v) => v.to_object(py),
        Value::Number(v) => {
            if v.is_f64() {
                v.as_f64().to_object(py)
            } else if v.is_u64() {
                v.as_u64().to_object(py)
            } else if v.is_i64() {
                v.as_i64().to_object(py)
            } else {
                py.None()
            }
        }
        Value::Bool(v) => v.to_object(py),
        Value::Array(vec) => {
            let r: Vec<PyObject> = vec.into_iter().map(|e| to_py_obj(e, py)).collect();
            r.to_object(py)
        }
        Value::Object(m) => {
            let mut dict: HashMap<String, PyObject> = HashMap::new();
            for (k0, v0) in m {
                let v: PyObject = to_py_obj(v0, py);
                dict.insert(k0, v);
            }
            dict.to_object(py)
        }
        _ => py.None(),
    }
}
