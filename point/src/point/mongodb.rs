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

pub async fn create(_: &dyn PointArg) -> Result<Box<dyn PointRunner>, Error>{
    Ok(Box::new(Mongodb {}))
}


async fn run(arg: &dyn PointArg) -> PointValue {
    let url = arg.config()["url"].as_str()
        .map(|s|arg.render(s))
        .ok_or(err!("010", "missing url"))??;
    let database = arg.config()["database"].as_str()
        .map(|s|arg.render(s))
        .ok_or(err!("010", "missing database"))??;
    let collection = arg.config()["collection"].as_str()
        .map(|s|arg.render(s))
        .ok_or(err!("010", "missing collection"))??;
    let op = arg.config()["operation"].as_str()
        .map(|s|arg.render(s))
        .ok_or(err!("010", "missing operation"))??;
    let op_arg = arg.config()["arg"].as_str()
        .map(|s|arg.render(s))
        .ok_or(err!("010", "missing arg"))??;

    // Parse a connection string into an options struct.
    let client_options = ClientOptions::parse(url.as_str()).await?;
    // Get a handle to the deployment.
    let client = Client::with_options(client_options)?;
    let db = client.database(database.as_str());
    let collection = db.collection::<Document>(collection.as_str());

    match op.as_str() {
        "insert_many" => {
            let arg_json:Json = from_str(op_arg.as_str())?;
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