use log::trace;
use rbatis::crud::CRUD;
use rbatis::plugin::page::{Page, PageRequest};
use rbatis::rbatis::Rbatis;

use chord_core::action::prelude::*;

use crate::err;

pub struct DatabaseCreator {}

impl DatabaseCreator {
    pub async fn new(_: Option<Value>) -> Result<DatabaseCreator, Error> {
        Ok(DatabaseCreator {})
    }
}

#[async_trait]
impl Creator for DatabaseCreator {
    async fn create(&self, chord: &dyn Chord, arg: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        let args_init = arg.args_init();
        if let Some(args_init) = args_init {
            let url = &args_init["url"];
            if url.is_string() {
                let url = chord.render(arg.context(), url)?;
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
    async fn execute(
        &self,
        _chord: &dyn Chord,
        arg: &mut dyn Arg,
    ) -> Result<Box<dyn Scope>, Error> {
        run(&self, arg).await
    }
}

async fn create_rb(url: &str) -> Result<Rbatis, Error> {
    let rb = Rbatis::new();
    rb.link(url).await?;
    Ok(rb)
}

async fn run(obj: &Database, arg: &dyn Arg) -> Result<Box<dyn Scope>, Error> {
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

async fn run0(arg: &dyn Arg, rb: &Rbatis) -> Result<Box<dyn Scope>, Error> {
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
