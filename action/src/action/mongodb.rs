use mongodb::bson::{to_document, Document};
use mongodb::{options::ClientOptions, Client};

use chord::action::prelude::*;
use chord::value::from_str;
use chord::{err, rerr};

pub struct MongodbFactory {}

impl MongodbFactory {
    pub async fn new(_: Option<Value>) -> Result<MongodbFactory, Error> {
        Ok(MongodbFactory {})
    }
}

#[async_trait]
impl Factory for MongodbFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Mongodb {}))
    }
}

struct Mongodb {}

#[async_trait]
impl Action for Mongodb {
    async fn run(&self, arg: &dyn RunArg) -> ActionValue {
        run(arg).await
    }
}

async fn run(arg: &dyn RunArg) -> ActionValue {
    let url = arg.config()["url"]
        .as_str()
        .map(|s| arg.render_str(s))
        .ok_or(err!("010", "missing url"))??;
    let database = arg.config()["database"]
        .as_str()
        .map(|s| arg.render_str(s))
        .ok_or(err!("010", "missing database"))??;
    let collection = arg.config()["collection"]
        .as_str()
        .map(|s| arg.render_str(s))
        .ok_or(err!("010", "missing collection"))??;
    let op = arg.config()["operation"]
        .as_str()
        .map(|s| arg.render_str(s))
        .ok_or(err!("010", "missing operation"))??;
    let op_arg = arg.config()["arg"]
        .as_str()
        .map(|s| arg.render_str(s))
        .ok_or(err!("010", "missing arg"))??;

    // Parse a connection string into an options struct.
    let client_options = ClientOptions::parse(url.as_str()).await?;
    // Get a handle to the deployment.
    let client = Client::with_options(client_options)?;
    let db = client.database(database.as_str());
    let collection = db.collection::<Document>(collection.as_str());

    match op.as_str() {
        "insert_many" => {
            let arg_json: Value = from_str(op_arg.as_str())?;
            match arg_json {
                Value::Array(arr) => {
                    let doc_vec: Vec<Document> =
                        arr.iter().map(|v| to_document(v).unwrap()).collect();
                    collection.insert_many(doc_vec, None).await?;
                    return Ok(Value::Null);
                }
                _ => rerr!("010", "illegal arg"),
            }
        }
        _ => rerr!("010", "illegal operation"),
    }
}
