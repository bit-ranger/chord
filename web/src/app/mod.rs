use std::path::Path;

use tide::{Request, Response};
use tide::http::StatusCode;
use tide::prelude::*;
use validator::{ValidationErrors, ValidationErrorsKind};

use chord_common::error::Error;
use chord_common::value::Json;

use shaku::{module, Component, Interface, HasComponent};

use crate::ctl;
use crate::app::conf::{Config, ConfigImpl, ConfigImplParameters};

mod logger;
pub mod conf;

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
                return Ok(Response::builder(StatusCode::BadRequest)
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

module! {
    BeanPool {
        components = [ConfigImpl],
        providers = []
    }
}


pub async fn init(data: Json) -> Result<(), Error>{
    let bean_pool = BeanPool::builder()
        .with_component_override_fn::<dyn Config>(Box::new(|c| Box::new(ConfigImpl::new(data))))
        .build();

    let config: &dyn Config = bean_pool.resolve_ref();
    let mut app = tide::new();

    let log_file_path = Path::new(config.log_path());
    let _log_handler = logger::init(config.log_level(), &log_file_path).await?;

    // ctl::job::CtlImpl::create_singleton().await?;
    app.at("/job/exec").post(
        json_handler!(ctl::job::CtlImpl::exec, ctl::job::CtlImpl::get_singleton().await)
    );

    app.listen(format!("{}:{}", config.server_ip(), config.server_port())).await?;
    Ok(())
}
