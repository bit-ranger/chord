use log::trace;
use rbatis::plugin::page::{Page, PageRequest};
use rbatis::rbatis::Rbatis;

use crate::action::CommonScope;
use chord::action::prelude::*;
use chord::value::{Map, Number};

pub struct DatabaseFactory {}

impl DatabaseFactory {
    pub async fn new(_: Option<Value>) -> Result<DatabaseFactory, Error> {
        Ok(DatabaseFactory {})
    }
}

#[async_trait]
impl Factory for DatabaseFactory {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        if let Some(url) = arg.args_raw()["url"].as_str() {
            if arg.is_static(url) {
                let url = arg.render_str(url, None)?;
                let rb = create_rb(url.as_str()).await?;
                return Ok(Box::new(Database { rb: Some(rb) }));
            }
        }

        return Ok(Box::new(Database { rb: None }));
    }
}

struct Database {
    rb: Option<Rbatis>,
}

#[async_trait]
impl Action for Database {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        run(&self, arg).await
    }
}

async fn create_rb(url: &str) -> Result<Rbatis, Error> {
    let rb = Rbatis::new();
    rb.link(url).await?;
    Ok(rb)
}

async fn run(obj: &Database, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
    let args = arg.args(None)?;
    return match obj.rb.as_ref() {
        Some(r) => run0(arg, r).await,
        None => {
            let url = args["url"].as_str().ok_or(err!("100", "missing url"))?;
            let rb = Rbatis::new();
            rb.link(url).await?;
            run0(arg, &rb).await
        }
    };
}

async fn run0(arg: &dyn RunArg, rb: &Rbatis) -> Result<Box<dyn Scope>, Error> {
    let args = arg.args(None)?;
    let sql = args["sql"].as_str().ok_or(err!("101", "missing sql"))?;

    if sql.trim_start().to_uppercase().starts_with("SELECT ") {
        let pr = PageRequest::new(1, 20);
        let sql_args = vec![];
        let page: Page<Value> = rb.fetch_page("", sql, &sql_args, &pr).await?;
        let mut map = Map::new();
        map.insert(
            String::from("total"),
            Value::Number(Number::from(page.total)),
        );
        map.insert(
            String::from("pages"),
            Value::Number(Number::from(page.pages)),
        );
        map.insert(
            String::from("page_no"),
            Value::Number(Number::from(page.page_no)),
        );
        map.insert(
            String::from("page_size"),
            Value::Number(Number::from(page.page_size)),
        );
        map.insert(String::from("records"), Value::Array(page.records));
        let value = Value::Object(map);
        trace!("select: {} >>> {}", arg.id(), value);
        return Ok(Box::new(CommonScope { args, value }));
    } else {
        let exec = rb.exec("", sql).await?;
        let mut map = Map::new();
        map.insert(
            String::from("rows_affected"),
            Value::Number(Number::from(exec.rows_affected)),
        );
        match exec.last_insert_id {
            Some(id) => {
                map.insert(
                    String::from("last_insert_id"),
                    Value::Number(Number::from(id)),
                );
            }
            None => {}
        }
        let value = Value::Object(map);
        trace!("exec: {} >>> {}", arg.id(), value);
        return Ok(Box::new(CommonScope { args, value }));
    }
}
