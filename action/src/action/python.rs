use std::collections::HashMap;

use pyo3::conversion::ToPyObject;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyLong, PyString, PyTuple};

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
        let py_any = py
            .eval(code, None, Some(&locals))
            .map_err(|e| err!("python", e.to_string()))?;
        Ok(to_value(py_any))
    }
}

fn to_value(py_any: &PyAny) -> Value {
    if py_any.is_none() {
        return Value::Null;
    } else if py_any.is_instance::<PyLong>().unwrap_or(false) {
        if let Ok(v) = py_any.extract::<i64>() {
            return Value::Number(Number::from(v));
        }
    } else if py_any.is_instance::<PyInt>().unwrap_or(false) {
        if let Ok(v) = py_any.extract::<i32>() {
            return Value::Number(Number::from(v));
        }
    } else if py_any.is_instance::<PyFloat>().unwrap_or(false) {
        if let Ok(v) = py_any.extract::<f64>() {
            if let Some(v) = Number::from_f64(v) {
                return Value::Number(v);
            }
        }
        return Value::Null;
    } else if py_any.is_instance::<PyBool>().unwrap_or(false) {
        if let Ok(v) = py_any.extract::<bool>() {
            return Value::Bool(v);
        }
    } else if py_any.is_instance::<PyString>().unwrap_or(false) {
        if let Ok(v) = py_any.extract::<String>() {
            return Value::String(v);
        }
    } else if py_any.is_instance::<PyList>().unwrap_or(false) {
        if let Ok(iter) = py_any.iter() {
            let vec: Vec<Value> = iter
                .map(|item| {
                    if let Ok(item) = item {
                        to_value(item)
                    } else {
                        Value::Null
                    }
                })
                .collect();
            return Value::Array(vec);
        }
    } else if py_any.is_instance::<PyDict>().unwrap_or(false) {
        let mut obj = Map::new();
        if let Ok(dict) = <PyDict as PyTryFrom>::try_from(py_any) {
            for item in dict.items().iter() {
                if item.is_instance::<PyTuple>().unwrap_or(false) {
                    if let Ok(k) = item.get_item(0) {
                        if let Ok(v) = item.get_item(1) {
                            if k.is_instance::<PyString>().unwrap_or(false) {
                                let k = to_value(k);
                                let v = to_value(v);
                                if let Value::String(k) = k {
                                    obj.insert(k, v);
                                }
                            }
                        }
                    }
                }
            }
            return Value::Object(obj);
        }
    }
    Value::Null
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
