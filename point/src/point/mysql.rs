use chord_common::err;
use chord_common::error::Error;
use chord_common::point::{async_trait, PointArg, PointRunner, PointValue};
use chord_common::value::{Json, Map, Number};
use log::debug;
use rbatis::plugin::page::{Page, PageRequest};
use rbatis::rbatis::Rbatis;

struct Mysql {
    rb: Option<Rbatis>,
}

#[async_trait]
impl PointRunner for Mysql {
    async fn run(&self, arg: &dyn PointArg) -> PointValue {
        run(&self, arg).await
    }
}

pub async fn create(arg: &dyn PointArg) -> Result<Box<dyn PointRunner>, Error> {
    let url = arg.config()["url"]
        .as_str()
        .ok_or(err!("010", "missing url"))?;

    if !arg.is_shared(url) {
        return Ok(Box::new(Mysql { rb: None }));
    }

    let url = arg.render(url)?;
    let rb = create_rb(url.as_str()).await?;
    return Ok(Box::new(Mysql { rb: Some(rb) }));
}

async fn create_rb(url: &str) -> Result<Rbatis, Error> {
    let rb = Rbatis::new();
    rb.link(url).await?;
    Ok(rb)
}

async fn run(obj: &Mysql, arg: &dyn PointArg) -> PointValue {
    return match obj.rb.as_ref() {
        Some(r) => run0(arg, r).await,
        None => {
            let url = arg.config()["url"]
                .as_str()
                .map(|s| arg.render(s))
                .ok_or(err!("010", "missing url"))??;
            let rb = Rbatis::new();
            rb.link(url.as_str()).await?;
            run0(arg, &rb).await
        }
    };
}

async fn run0(arg: &dyn PointArg, rb: &Rbatis) -> PointValue {
    let sql = arg.config()["sql"]
        .as_str()
        .map(|s| arg.render(s))
        .ok_or(err!("010", "missing sql"))??;

    if sql.trim_start().to_uppercase().starts_with("SELECT ") {
        let pr = PageRequest::new(1, 20);
        let args = vec![];
        let page: Page<Json> = rb.fetch_page("", sql.as_str(), &args, &pr).await?;
        let mut map = Map::new();
        map.insert(
            String::from("total"),
            Json::Number(Number::from(page.total)),
        );
        map.insert(
            String::from("pages"),
            Json::Number(Number::from(page.pages)),
        );
        map.insert(
            String::from("page_no"),
            Json::Number(Number::from(page.page_no)),
        );
        map.insert(
            String::from("page_size"),
            Json::Number(Number::from(page.page_size)),
        );
        map.insert(String::from("records"), Json::Array(page.records));
        let page = Json::Object(map);
        debug!("mysql select:\n{}", page);
        return Ok(page);
    } else {
        let exec = rb.exec("", sql.as_str()).await?;
        let mut map = Map::new();
        map.insert(
            String::from("rows_affected"),
            Json::Number(Number::from(exec.rows_affected)),
        );
        match exec.last_insert_id {
            Some(id) => {
                map.insert(
                    String::from("last_insert_id"),
                    Json::Number(Number::from(id)),
                );
            }
            None => {}
        }
        let exec = Json::Object(map);
        debug!("mysql exec:\n{}", exec);
        return Ok(exec);
    }
}
