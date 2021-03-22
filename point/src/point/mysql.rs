use common::point::PointArg;
use common::value::{Json, Map, Number};
use crate::model::{PointValue, PointError};
use sqlx::{MySqlConnection, Row, Column, ValueRef, TypeInfo};
use sqlx::Connection;
use sqlx::mysql::MySqlRow;
use crate::perr;
use log::warn;
use sqlx::decode::Decode;
use sqlx::types::chrono::{DateTime, Utc};

pub async fn run(point_arg: &dyn PointArg) -> PointValue {
    let url = point_arg.config_rendered(vec!["url"]).ok_or(perr!("010", "missing url"))?;
    let sql = point_arg.config_rendered(vec!["sql"]).ok_or(perr!("010", "missing sql"))?;
    let mut conn = MySqlConnection::connect(url.as_str()).await?;
    let vec:Vec<Json> = sqlx::query(sql.as_str())
        .map(|row: MySqlRow| to_json(row))
        .fetch_all(&mut conn).await?;

    let rows = Json::Array(vec);
    println!("mysql rows:\n{}", rows);
    return Ok(rows)
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
        let v= row.try_get_raw(k).unwrap();
        if v.is_null(){
            map.insert(String::from(k), Json::Null);
            continue;
        }

        let type_info = v.type_info();
        let type_name = type_info.name();
        match type_name {
            "BOOLEAN"
            => {
                map.insert(String::from(k), Json::Bool(bool::decode(v).unwrap()));
            },
            "TINYINT UNSIGNED"
            | "SMALLINT UNSIGNED"
            | "INT UNSIGNED"
            | "MEDIUMINT UNSIGNED"
            | "BIGINT UNSIGNED"
             => {
                 map.insert(String::from(k), Json::Number(Number::from(u64::decode(v).unwrap())));
             },
             "TINYINT"
            | "SMALLINT"
            | "INT"
            | "MEDIUMINT"
            | "BIGINT"
            => {
                map.insert(String::from(k), Json::Number(Number::from(i64::decode(v).unwrap())));
            },
            "FLOAT"
            => {
                map.insert(String::from(k), Json::Number(Number::from_f64(f64::decode(v).unwrap()).unwrap()));
            },
             "DOUBLE"
            | "DECIMAL"
            => {
                map.insert(String::from(k), Json::Number(Number::from_f64(f64::decode(v).unwrap()).unwrap()));
            },
             "DATE"
            | "TIMESTAMP"
            | "TIME"
            | "DATETIME"
            => {
                 let time = DateTime::< Utc > ::decode(v).unwrap();
                 let tt = time.format("%Y-%m-%d %T").to_string();
                 map.insert(String::from(k), Json::String(tt));
            },
             "ENUM"
            | "CHAR"
            | "VARCHAR"
            => {
                map.insert(String::from(k), Json::String(String::decode(v).unwrap()));
            },
            _ => {
                warn!("column type not supported! {}", k);
                map.insert(String::from(k), Json::Null);
            }
        }
    }

    return Json::Object(map);

}
