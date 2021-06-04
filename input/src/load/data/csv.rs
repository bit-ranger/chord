use chord_common::err;
use chord_common::error::Error;
use chord_common::input::async_trait;
use chord_common::input::DataLoad;
use chord_common::rerr;
use chord_common::value::{Json, Map};
use csv::{Reader, ReaderBuilder};
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

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
impl DataLoad for Loader {
    async fn load(&mut self, size: usize) -> Result<Vec<Json>, Error> {
        load(&mut self.reader, size).await
    }

    async fn reset(&mut self) -> Result<(), Error> {
        self.reader = from_path(&self.path).await?;
        Ok(())
    }
}

async fn load<R: std::io::Read>(reader: &mut Reader<R>, size: usize) -> Result<Vec<Json>, Error> {
    let mut hashmap_vec = Vec::new();
    let mut curr_size = 0;
    for result in reader.deserialize() {
        let result = match result {
            Err(e) => return rerr!("csv", format!("{:?}", e)),
            Ok(r) => r,
        };

        let record: Map = result;

        hashmap_vec.push(Json::Object(record));

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
