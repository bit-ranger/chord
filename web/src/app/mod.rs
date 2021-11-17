use tide::http::StatusCode;
use tide::prelude::*;
use tide::{Request, Response};
use validator::{ValidationErrors, ValidationErrorsKind};

use chord::value::Value;

use crate::app::conf::{Config, ConfigImpl};
use crate::ctl::job;
use async_std::sync::Arc;
use bean::component::HasComponent;
use bean::container;
use chord_input::load;

pub mod conf;
mod logger;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("config error:\n{0}")]
    Config(load::conf::Error),

    #[error("log error:\n{0}")]
    Logger(logger::Error),

    #[error("job error:\n{0}")]
    Job(job::Error),

    #[error("web error:\n{0}")]
    Web(std::io::Error),
}

#[derive(Serialize, Deserialize)]
struct ErrorBody {
    code: String,
    message: String,
}

fn common_error_json(e: &Error) -> Value {
    json!(ErrorBody {
        code: match e {
            Error::Config(_) => "Config".to_string(),
            Error::Logger(_) => "Logger".to_string(),
            Error::Job(_) => "Job".to_string(),
            Error::Web(_) => "Web".to_string(),
        },
        message: e.to_string()
    })
}

fn validator_error_json_nested(e: &ValidationErrors) -> Vec<String> {
    return e
        .errors()
        .iter()
        .map(|(k, e)| match e {
            ValidationErrorsKind::Field(ev) => ev
                .iter()
                .map(|e| format!("[{}] {}", k, e.to_string()))
                .collect(),
            ValidationErrorsKind::Struct(f) => validator_error_json_nested(f.as_ref()),
            ValidationErrorsKind::List(m) => m
                .iter()
                .map(|(_i, e)| validator_error_json_nested(e.as_ref()))
                .fold(Vec::new(), |mut l, e| {
                    l.extend(e);
                    return l;
                }),
        })
        .fold(Vec::new(), |mut l, e| {
            l.extend(e);
            return l;
        });
}

fn validator_error_json(e: &ValidationErrors) -> Value {
    json!(ErrorBody {
        code: "400".into(),
        message: validator_error_json_nested(e).into_iter().last().unwrap()
    })
}

#[macro_export]
macro_rules! json_handler {
    ($closure:tt) => {{
        |mut req: Request<()>| async move {
            let rb = req.body_json().await?;
            if let Err(e) = validator::Validate::validate(&rb) {
                return Ok(Response::builder(StatusCode::BadRequest).body(validator_error_json(&e)));
            };
            let rst = $closure(rb).await;
            match rst {
                Ok(r) => Ok(Response::builder(StatusCode::Ok).body(json!(r))),
                Err(e) => {
                    Ok(Response::builder(StatusCode::InternalServerError)
                        .body(common_error_json(&e)))
                }
            }
        }
    }};
}

container!(Web {ConfigImpl, job::CtlImpl});

pub async fn init(data: Value) -> Result<(), Error> {
    let config = Arc::new(ConfigImpl::new(data));

    let log_file_path = config.log_dir().join("web.log");
    let _log_handler = logger::init(config.log_level(), &log_file_path)
        .await
        .map_err(|e| Error::Logger(e))?;

    let job_ctl = Arc::new(
        job::CtlImpl::new(config.clone())
            .await
            .map_err(|e| Error::Job(e))?,
    );

    Web::init()
        .put("default", config.clone())
        .put("default", job_ctl.clone());

    let mut app = tide::new();

    app.at("/job/exec").post(json_handler!(
        (|rb: job::Req| async {
            let job_ctl: Arc<job::CtlImpl> = Web::borrow().get("default").unwrap();
            job::Ctl::exec(job_ctl.as_ref(), rb)
                .await
                .map_err(|e| Error::Job(e))
        })
    ));

    app.at("/").get(|_| async { Ok("Hello, world!") });

    app.listen(format!("{}:{}", config.server_ip(), config.server_port()))
        .await
        .map_err(|e| Error::Web(e))?;
    Ok(())
}
