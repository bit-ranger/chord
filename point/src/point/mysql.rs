use common::point::PointArg;
use common::value::{Json, Map};
use crate::model::{PointValue, PointError};
use sqlx::{MySqlConnection, Row, Column};
use sqlx::Connection;
use sqlx::mysql::MySqlRow;
use crate::perr;

pub async fn run(point_arg: &dyn PointArg) -> PointValue {
    let url = point_arg.config()["url"].as_str().ok_or(perr!("010", "missing url"))?;
    let sql = point_arg.config()["sql"].as_str().ok_or(perr!("010", "missing sql"))?;
    let mut conn = MySqlConnection::connect(url).await?;
    let result:Vec<Json> = sqlx::query(sql)
        .map(|row: MySqlRow| to_json(row))
        .fetch_all(&mut conn).await?;
    return Ok(Json::Array(result))
}

impl From<sqlx::Error> for PointError {
    fn from(err: sqlx::Error) -> PointError {
        PointError::new("sqlx", format!("{:?}", err).as_str())
    }
}

fn to_json(row: MySqlRow) -> Json{
    let mut map = Map::with_capacity(row.len());
    for c in row.columns() {
        let k = c.name();
        let v:sqlx::types::Json<Json> = row.get(k);
        map.insert(String::from(k), v.0);
    }

    return Json::Object(map);
}