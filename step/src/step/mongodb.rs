use mongodb::bson::{to_document, Document};
use mongodb::{options::ClientOptions, Client};

use chord_common::error::Error;
use chord_common::step::{async_trait, RunArg, StepRunner, StepValue, CreateArg};
use chord_common::value::{from_str, Json};
use chord_common::{err, rerr};

struct Mongodb {}

#[async_trait]
impl StepRunner for Mongodb {
    async fn run(&self, arg: &dyn RunArg) -> StepValue {
        run(arg).await
    }
}

pub async fn create(_: Option<&Json>, _: &dyn CreateArg) -> Result<Box<dyn StepRunner>, Error> {
    Ok(Box::new(Mongodb {}))
}

async fn run(arg: &dyn RunArg) -> StepValue {
    let url = arg.config()["url"]
        .as_str()
        .map(|s| arg.render(s))
        .ok_or(err!("010", "missing url"))??;
    let database = arg.config()["database"]
        .as_str()
        .map(|s| arg.render(s))
        .ok_or(err!("010", "missing database"))??;
    let collection = arg.config()["collection"]
        .as_str()
        .map(|s| arg.render(s))
        .ok_or(err!("010", "missing collection"))??;
    let op = arg.config()["operation"]
        .as_str()
        .map(|s| arg.render(s))
        .ok_or(err!("010", "missing operation"))??;
    let op_arg = arg.config()["arg"]
        .as_str()
        .map(|s| arg.render(s))
        .ok_or(err!("010", "missing arg"))??;

    // Parse a connection string into an options struct.
    let client_options = ClientOptions::parse(url.as_str()).await?;
    // Get a handle to the deployment.
    let client = Client::with_options(client_options)?;
    let db = client.database(database.as_str());
    let collection = db.collection::<Document>(collection.as_str());

    match op.as_str() {
        "insert_many" => {
            let arg_json: Json = from_str(op_arg.as_str())?;
            match arg_json {
                Json::Array(arr) => {
                    let doc_vec: Vec<Document> =
                        arr.iter().map(|v| to_document(v).unwrap()).collect();
                    collection.insert_many(doc_vec, None).await?;
                    return Ok(Json::Null);
                }
                _ => rerr!("010", "illegal arg"),
            }
        }
        _ => rerr!("010", "illegal operation"),
    }
}
