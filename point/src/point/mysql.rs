use common::point::PointArg;
use common::value::Json;
use crate::model::PointValue;
use async_std::task::sleep;
use std::time::Duration;
use sqlx::{MySqlConnection};
use sqlx::Connection;
use sqlx::mysql::MySqlRow;

pub async fn run(point_arg: &dyn PointArg) -> PointValue {
    let url = point_arg.config()["url"].as_str()?;
    let sql = point_arg.config()["sql"].as_str()?;
    let conn = MySqlConnection::connect(url).await?;
    let result:Vec<Json> = sqlx::query(sql)
        .map(|row: MySqlRow| row.into())
        .fetch_all(&conn).await?;
    return Ok(Json::Array(result))
}