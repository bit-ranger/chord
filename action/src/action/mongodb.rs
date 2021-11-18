use mongodb::bson::{to_document, Document};
use mongodb::{options::ClientOptions, Client};

use chord::action::prelude::*;
use chord::value::from_str;

use crate::err;

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
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        run(arg).await
    }
}

async fn run(arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
    let args = arg.args()?;
    let url = args["url"].as_str().ok_or(err!("100", "missing url"))?;
    let database = args["database"]
        .as_str()
        .ok_or(err!("101", "missing database"))?;
    let collection = args["collection"]
        .as_str()
        .ok_or(err!("102", "missing collection"))?;
    let op = args["operation"]
        .as_str()
        .ok_or(err!("103", "missing operation"))?;
    let op_arg = args["arg"].as_str().ok_or(err!("104", "missing arg"))?;

    // Parse a connection string into an options struct.
    let client_options = ClientOptions::parse(url).await?;
    // Get a handle to the deployment.
    let client = Client::with_options(client_options)?;
    let db = client.database(database);
    let collection = db.collection::<Document>(collection);

    match op {
        "insert_many" => {
            let arg_json: Value = from_str(op_arg)?;
            match arg_json {
                Value::Array(arr) => {
                    let doc_vec: Vec<Document> =
                        arr.iter().map(|v| to_document(v).unwrap()).collect();
                    collection.insert_many(doc_vec, None).await?;
                    return Ok(Box::new(Value::Null));
                }
                _ => Err(err!("105", "illegal arg")),
            }
        }
        _ => Err(err!("106", "illegal operation")),
    }
}
