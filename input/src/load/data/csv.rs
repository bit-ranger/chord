use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use csv::{Reader, ReaderBuilder};

use chord::err;
use chord::input::async_trait;
use chord::input::CaseLoad;
use chord::rerr;
use chord::value::{to_string, Map, Value};
use chord::Error;

pub struct Loader {
    path: PathBuf,
    reader: Reader<File>,
}

impl Loader {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Loader, Error> {
        let loader = Loader {
            path: path.as_ref().to_path_buf(),
            reader: from_path(path.as_ref()).await?,
        };
        Ok(loader)
    }

    pub async fn close(self) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait]
impl CaseLoad for Loader {
    async fn load(&mut self, size: usize) -> Result<Vec<Value>, Error> {
        load(&mut self.reader, size).await
    }

    async fn reset(&mut self) -> Result<(), Error> {
        self.reader = from_path(&self.path).await?;
        Ok(())
    }
}

async fn load<R: std::io::Read>(reader: &mut Reader<R>, size: usize) -> Result<Vec<Value>, Error> {
    let mut hashmap_vec = Vec::new();
    let mut curr_size = 0;
    for result in reader.deserialize() {
        let result: Map = match result {
            Err(e) => return rerr!("csv", format!("{:?}", e)),
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
