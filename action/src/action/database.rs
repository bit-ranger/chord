use log::trace;
use rbatis::crud::CRUD;
use rbatis::executor::Executor;
use rbatis::plugin::page::{Page, PageRequest};
use rbatis::rbatis::Rbatis;

use chord_core::action::prelude::*;

use crate::err;

pub struct DatabaseFactory {}

impl DatabaseFactory {
    pub async fn new(_: Option<Value>) -> Result<DatabaseFactory, Error> {
        Ok(DatabaseFactory {})
    }
}

#[async_trait]
impl Factory for DatabaseFactory {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let args_raw = arg.args_raw();
        if let Some(url) = args_raw["url"].as_str() {
            if arg.is_static(url) {
                let url = arg.render_str(url)?;
                let url = url.as_str().ok_or(err!("100", "invalid url"))?;
                let rb = create_rb(url).await?;
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
    async fn run(&self, arg: &mut dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        run(&self, arg).await
    }
}

async fn create_rb(url: &str) -> Result<Rbatis, Error> {
    let rb = Rbatis::new();
    rb.link(url).await?;
    Ok(rb)
}

async fn run(obj: &Database, arg: &mut dyn RunArg) -> Result<Box<dyn Scope>, Error> {
    let args = arg.args()?;
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

async fn run0(arg: &mut dyn RunArg, rb: &Rbatis) -> Result<Box<dyn Scope>, Error> {
    let args = arg.args()?;
    let sql = args["sql"].as_str().ok_or(err!("101", "missing sql"))?;
    let sql = sql.trim();
    if sql.to_uppercase().starts_with("SELECT ") {
        let mut sql = sql;
        if sql.ends_with(";") {
            sql = &sql[0..sql.len() - 1];
        }

        let page_no = args["page_no"].as_u64().unwrap_or(1);
        let page_size = args["page_size"].as_u64().unwrap_or(100);

        let pr = PageRequest::new(page_no, page_size);
        let page: Page<Value> = rb.fetch_page(sql, vec![], &pr).await?;
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
        let page = Value::Object(map);
        trace!("select: {} >>> {}", arg.id(), page);
        return Ok(Box::new(page));
    } else {
        let exec = rb.exec(sql, vec![]).await?;
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
        let exec = Value::Object(map);
        trace!("exec: {} >>> {}", arg.id(), exec);
        return Ok(Box::new(exec));
    }
}
