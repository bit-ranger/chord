
use csv::{ReaderBuilder, Reader};
use common::error::Error;
use common::err;
use common::perr;
use common::value::{Json, Map};
use std::path::Path;
use std::fs::File;

pub fn load<R: std::io::Read>(reader: &mut Reader<R>, size_limit: usize) -> Result<Vec<Json>, Error> {
    let mut hashmap_vec = Vec::new();
    let mut curr_size = 0;
    for result in reader.deserialize() {

        let result = match result  {
            Err(e)  => return err!("csv", format!("{:?}", e)),
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

pub async fn from_reader<R: std::io::Read>(reader: R) -> Result<Reader<R>, Error>{
    Ok(ReaderBuilder::new().from_reader(reader))
}

pub async fn from_path<P: AsRef<Path>>(path: P) -> Result<Reader<File>, Error>{
    ReaderBuilder::new().from_path(path).map_err(|e|perr!("csv", e.to_string()))
}
