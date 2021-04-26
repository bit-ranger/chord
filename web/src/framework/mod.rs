use std::path::Path;

use lazy_static::lazy_static;
use tide::{Request, Response};
use tide::http::StatusCode;
use tide::prelude::*;
use validator::{ValidationErrors, ValidationErrorsKind};

use chord_common::error::Error;
use chord_common::value::Json;

use crate::controller;

mod logger;

#[derive(Serialize, Deserialize)]
struct ErrorBody {
    code: String,
    message: String,
}

fn common_error_json(e: &Error) -> Json {
    json!(ErrorBody{
                code: e.code().into(),
                message: e.message().into()
            })
}

fn validator_error_json_nested(e: &ValidationErrors) -> Vec<String> {
    return e.errors()
        .iter()
        .map(|(k, e)|
            match e {
                ValidationErrorsKind::Field(ev) =>
                    ev.iter()
                        .map(|e| format!("[{}] {}", k, e.to_string()))
                        .collect(),
                ValidationErrorsKind::Struct(f) =>
                    validator_error_json_nested(f.as_ref()),
                ValidationErrorsKind::List(m) =>
                    m.iter()
                        .map(|(_i, e)|
                            validator_error_json_nested(e.as_ref()))
                        .fold(Vec::new(), |mut l, e| {
                            l.extend(e);
                            return l;
                        })
            }
        )
        .fold(Vec::new(), |mut l, e| {
            l.extend(e);
            return l;
        });
}

fn validator_error_json(e: &ValidationErrors) -> Json {
    json!(ErrorBody{
                code: "400".into(),
                message: validator_error_json_nested(e).into_iter().last().unwrap()
            })
}


#[macro_export]
macro_rules! json_handler {
    ($func:path, $ctl:expr) => {{
        |mut req: Request<()>| async move{
            let rb =  req.body_json().await?;
            if let Err(e) = validator::Validate::validate(&rb){
                return Ok(Response::builder(StatusCode::InternalServerError)
                    .body(validator_error_json(&e)))
            };
            let rst = $func($ctl, rb).await;
            match rst{
                Ok(r) => Ok(Response::builder(StatusCode::Ok)
                    .body(json!(r))),
                Err(e) => Ok(Response::builder(StatusCode::InternalServerError)
                    .body(common_error_json(&e)))
            }
        }
    }}
}



lazy_static! {
    static ref JOB_CTL:controller::job::Ctl = {
        controller::job::Ctl::new(String::from("/data/chord/job"), String::from("/data/chord/work"))
    };
}

pub async fn init() -> Result<(), Error>{
    let mut app = tide::new();

    let log_file_path = Path::new("/data/logs/chord/log.log");
    let _log_handler = logger::init(vec![], &log_file_path).await?;


    app.at("/job/exec").post(
        json_handler!(controller::job::Ctl::exec, &JOB_CTL)
    );

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
