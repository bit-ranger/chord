
use csv::{ReaderBuilder, Reader};
use chord_common::error::Error;
use chord_common::rerr;
use chord_common::err;
use chord_common::value::{Json, Map};
use std::path::Path;
use std::fs::File;

pub struct Loader{
    reader: Reader<File>,
    size_limit: usize
}

impl Loader {

    pub async fn new<P: AsRef<Path>>(path: P, size_limit: usize) -> Result<Loader, Error>{
        let loader = Loader {
            reader: from_path(path).await?,
            size_limit
        };
        Ok(loader)
    }

    pub async fn load(&mut self) -> Result<Vec<Json>, Error>{
        load(&mut self.reader, self.size_limit).await
    }

    pub async fn close(self) -> Result<(),Error>{
        Ok(())
    }
}

async fn load<R: std::io::Read>(reader: &mut Reader<R>, size_limit: usize) -> Result<Vec<Json>, Error> {
    let mut hashmap_vec = Vec::new();
    let mut curr_size = 0;
    for result in reader.deserialize() {

        let result = match result  {
            Err(e)  => return rerr!("csv", format!("{:?}", e)),
            Ok(r) => r
        };

        let record: Map = result;

        hashmap_vec.push(Json::Object(record));

        curr_size += 1;
        if curr_size == size_limit{
            break;
        }
    }
    Ok(hashmap_vec)
}


async fn from_path<P: AsRef<Path>>(path: P) -> Result<Reader<File>, Error>{
    ReaderBuilder::new().from_path(path).map_err(|e| err!("csv", e.to_string()))
}
