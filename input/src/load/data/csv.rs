use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use csv::{Reader, ReaderBuilder};

use chord::err;
use chord::input::CaseLoad;
use chord::input::{async_trait, CaseStore};
use chord::value::{to_string, Map, Value};
use chord::Error;

pub struct Store {
    path: PathBuf,
}

impl Store {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Store, Error> {
        Ok(Store {
            path: path.as_ref().to_path_buf(),
        })
    }
}

#[async_trait]
impl CaseStore for Store {
    async fn create(&self, name: &str) -> Result<Box<dyn CaseLoad>, Error> {
        let loader = Loader::new(self.path.join(format!("{}.csv", name))).await?;
        Ok(Box::new(loader))
    }
}

struct Loader {
    row_num: usize,
    reader: Reader<File>,
}

impl Loader {
    async fn new<P: AsRef<Path>>(path: P) -> Result<Loader, Error> {
        let loader = Loader {
            row_num: 1,
            reader: from_path(path.as_ref()).await?,
        };
        Ok(loader)
    }
}

#[async_trait]
impl CaseLoad for Loader {
    async fn load(&mut self, size: usize) -> Result<Vec<(String, Value)>, Error> {
        let dv = load(&mut self.reader, size).await?;
        let mut result: Vec<(String, Value)> = vec![];
        for d in dv {
            self.row_num = self.row_num + 1;
            result.push((self.row_num.to_string(), d));
        }
        Ok(result)
    }
}

async fn load<R: std::io::Read>(reader: &mut Reader<R>, size: usize) -> Result<Vec<Value>, Error> {
    let mut hashmap_vec = Vec::new();
    let mut curr_size = 0;
    for result in reader.deserialize() {
        let result: Map = match result {
            Err(e) => return Err(err!("csv", format!("{:?}", e))),
            Ok(r) => r,
        };

        let mut record: Map = Map::new();
        //data fields must all be string
        for (k, v) in result {
            if v.is_string() {
                record.insert(k, v);
            } else {
                record.insert(k, Value::String(to_string(&v)?));
            }
        }

        hashmap_vec.push(Value::Object(record));

        curr_size += 1;
        if curr_size == size {
            break;
        }
    }
    Ok(hashmap_vec)
}

async fn from_path<P: AsRef<Path>>(path: P) -> Result<Reader<File>, Error> {
    ReaderBuilder::new()
        .from_path(path)
        .map_err(|e| err!("csv", e.to_string()))
}
