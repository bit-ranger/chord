use common::point::PointArg;
use common::value::{Json, Map, Number};
use crate::model::{PointValue, PointError};
use crate::perr;
use log::{warn, debug};
use rbatis::rbatis::Rbatis;
use rbatis::plugin::page::{Page, PageRequest};

pub async fn run(pt_arg: &dyn PointArg) -> PointValue {
    let url = pt_arg.config_rendered(vec!["url"]).ok_or(perr!("010", "missing url"))?;
    let sql = pt_arg.config_rendered(vec!["sql"]).ok_or(perr!("011", "missing sql"))?;
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

impl From<rbatis::Error> for PointError {
    fn from(err: rbatis::Error) -> PointError {
        PointError::new("rbatis", format!("{:?}", err).as_str())
    }
}
