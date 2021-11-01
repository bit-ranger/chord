use std::path::Path;

use chord::err;
use chord::value::{Map, Number, Value};
use chord::Error;
use hocon_linked::{Hocon, HoconLoader};

pub async fn load<P: AsRef<Path>>(path: P) -> Result<Value, Error> {
    let loader = HoconLoader::new();
    let hocon = loader.strict().load_file(path)?.hocon()?;
    let deserialized: Result<Value, Error> = convert(hocon);
    return match deserialized {
        Err(e) => return Err(err!("hocon", format!("{:?}", e))),
        Ok(r) => Ok(r),
    };
}

fn convert(hocon: Hocon) -> Result<Value, Error> {
    let hv = match hocon {
        Hocon::Null => Value::Null,
        Hocon::Real(v) => Value::Number(
            Number::from_f64(v).ok_or_else(|| err!("hocon", format!("invalid number {}", v)))?,
        ),
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
        Hocon::BadValue(e) => Err(err!("hocon", format!("bad value {}", e)))?,
    };
    Ok(hv)
}
