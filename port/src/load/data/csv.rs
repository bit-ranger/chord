use std::collections::BTreeMap;
use csv::ReaderBuilder;
use common::error::Error;
use common::err;

pub fn load<R: std::io::Read>(reader: R) -> Result<Vec<BTreeMap<String, String>>, Error> {
    let mut rdr = ReaderBuilder::new().from_reader(reader);
    let mut hashmap_vec = Vec::new();
    for result in rdr.deserialize() {
        let result = match result  {
            Err(e)  => return err!("csv", format!("{:?}", e).as_str()),
            Ok(r) => r
        };

        let record: BTreeMap<String, String> = result;

        hashmap_vec.push(record);
    }
    Ok(hashmap_vec)
}
