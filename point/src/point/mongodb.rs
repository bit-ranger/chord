use mongodb::{Client, options::ClientOptions};
use mongodb::bson::{Document, to_document};

use chord_common::{err, rerr};
use chord_common::point::{PointArg, PointValue, PointRunner, async_trait};
use chord_common::value::{Json, from_str};
use chord_common::error::Error;

struct Mongodb {}

#[async_trait]
impl PointRunner for Mongodb {

    async fn run(&self, arg: &dyn PointArg) -> PointValue {
        run(arg).await
    }
}

pub async fn create(_: &Json) -> Result<Box<dyn PointRunner>, Error>{
    Ok(Box::new(Mongodb {}))
}


async fn run(pt_arg: &dyn PointArg) -> PointValue {
    let url = pt_arg.config_rendered(vec!["url"]).ok_or(err!("010", "missing url"))?;
    let database = pt_arg.config_rendered(vec!["database"]).ok_or(err!("010", "missing database"))?;
    let collection = pt_arg.config_rendered(vec!["collection"]).ok_or(err!("010", "missing collection"))?;
    let operation = pt_arg.config_rendered(vec!["operation"]).ok_or(err!("010", "missing operation"))?;
    let arg = pt_arg.config_rendered(vec!["arg"]).ok_or(err!("010", "missing arg"))?;

    // Parse a connection string into an options struct.
    let client_options = ClientOptions::parse(url.as_str()).await?;
    // Get a handle to the deployment.
    let client = Client::with_options(client_options)?;
    let db = client.database(database.as_str());
    let collection = db.collection::<Document>(collection.as_str());

    match operation.as_str() {
        "insert_many" => {
            let arg_json:Json = from_str(arg.as_str())?;
            match arg_json {
                Json::Array(arr) => {
                    let doc_vec: Vec<Document> = arr
                        .iter()
                        .map(|v| to_document(v).unwrap())
                        .collect();
                    collection.insert_many(doc_vec, None).await?;
                    return Ok(Json::Null);
                },
                _ => rerr!("010", "illegal arg")
            }
        },
        _ => rerr!("010", "illegal operation")
    }
}