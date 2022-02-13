use std::path::Path;

use hocon::{Hocon, HoconLoader};

use chord_core::future::fs::metadata;
use chord_core::value::{Map, Number, Value};

pub type Error = hocon::Error;

pub async fn load<P: AsRef<Path>>(dir_path: P, name: &str) -> Result<Value, Error> {
    let file_path = dir_path.as_ref().join(format!("{}.conf", name));
    let loader = HoconLoader::new();
    let hocon = loader.strict().load_file(file_path)?.hocon()?;
    convert(hocon)
}

pub async fn exists<P: AsRef<Path>>(dir_path: P, name: &str) -> bool {
    let file_path = dir_path.as_ref().join(format!("{}.conf", name));
    metadata(file_path).await.is_ok()
}

fn convert(hocon: Hocon) -> Result<Value, Error> {
    let hv = match hocon {
        Hocon::Null => Value::Null,
        Hocon::Real(v) => {
            Value::Number(Number::from_f64(v).ok_or_else(|| Error::Deserialization {
                message: format!("{:?}", hocon),
            })?)
        }
        Hocon::Integer(v) => Value::Number(Number::from(v)),
        Hocon::String(v) => Value::String(v),
        Hocon::Boolean(v) => Value::Bool(v),
        Hocon::Array(vec) => {
            let mut v: Vec<Value> = Vec::with_capacity(vec.len());
            for i in vec {
                let hv = convert(i)?;
                v.push(hv)
            }
            Value::Array(v)
        }
        Hocon::Hash(hash) => {
            let mut m = Map::new();
            for (k, v) in hash {
                let hv = convert(v)?;
                m.insert(k, hv);
            }
            Value::Object(m)
        }
        Hocon::BadValue(_) => Err(Error::Deserialization {
            message: format!("{:?}", hocon),
        })?,
    };
    Ok(hv)
}
