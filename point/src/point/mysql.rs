use chord_common::value::{Json, Map, Number};
use chord_common::point::{PointArg, PointValue, PointRunner, async_trait};
use chord_common::{err};
use log::{debug};
use rbatis::rbatis::Rbatis;
use rbatis::plugin::page::{Page, PageRequest};
use chord_common::error::Error;

struct Mysql {}

#[async_trait]
impl PointRunner for Mysql {

    async fn run(&self, arg: &dyn PointArg) -> PointValue {
        run(arg).await
    }
}

pub async fn create(_: &dyn PointArg) -> Result<Box<dyn PointRunner>, Error>{
    Ok(Box::new(Mysql {}))
}


async fn run(arg: &dyn PointArg) -> PointValue {
    let url = arg.config()["url"].as_str()
        .map(|s|arg.render(s))
        .ok_or(err!("010", "missing url"))??;
    let sql = arg.config()["sql"].as_str()
        .map(|s|arg.render(s))
        .ok_or(err!("010", "missing sql"))??;
    let rb = Rbatis::new();
    rb.link(url.as_str()).await?;

    if sql.trim_start().to_uppercase().starts_with("SELECT "){
        let pr = PageRequest::new(1, 20);
        let args = vec![];
        let page:Page<Json> = rb.fetch_page("", sql.as_str(), &args, &pr).await?;
        let mut map = Map::new();
        map.insert(String::from("total"), Json::Number(Number::from(page.total)));
        map.insert(String::from("pages"), Json::Number(Number::from(page.pages)));
        map.insert(String::from("page_no"), Json::Number(Number::from(page.page_no)));
        map.insert(String::from("page_size"), Json::Number(Number::from(page.page_size)));
        map.insert(String::from("records"), Json::Array(page.records));
        let page = Json::Object(map);
        debug!("mysql select:\n{}", page);
        return Ok(page)
    } else {
        let exec = rb.exec("", sql.as_str()).await?;
        let mut map = Map::new();
        map.insert(String::from("rows_affected"), Json::Number(Number::from(exec.rows_affected)));
        match exec.last_insert_id {
            Some(id) => {
                map.insert(String::from("last_insert_id"), Json::Number(Number::from(id)));
            },
            None => {}
        }
        let exec = Json::Object(map);
        debug!("mysql exec:\n{}", exec);
        return Ok(exec)
    }


}

// impl From<rbatis::Error> for PointError {
//     fn from(err: rbatis::Error) -> PointError {
//         PointError::new("rbatis", format!("{:?}", err))
//     }
// }
