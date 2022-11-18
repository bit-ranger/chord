use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use actix_web::{App, get, HttpResponse, HttpServer, post, Responder, ResponseError};
use actix_web::body::BoxBody;
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::http::StatusCode;
use actix_web::web::Json;
use bean::component::HasComponent;
use bean::container;
use itertools::Itertools;
use tracing::Level;
use validator::{ValidationErrors, ValidationErrorsKind};

use chord_core::value::json;
use chord_core::value::Value;

use crate::app::conf::{Config, ConfigImpl};
use crate::ctl::job;
use crate::ctl::job::Val;

pub mod conf;

#[derive(thiserror::Error)]
pub enum Error {
    #[error("config error:\n{0}")]
    Config(chord_input::conf::Error),

    #[error("log error:\n{0}")]
    Logger(String),

    #[error("job error:\n{0}")]
    Job(job::Error),

    #[error("web error:\n{0}")]
    Web(std::io::Error),

    #[error("{0}")]
    Validation(String),
}

impl From<job::Error> for Error {
    fn from(e: job::Error) -> Self {
        if let job::Error::Validation(ve) = e {
            Error::Validation(
                validator_error_string_nested(&ve)
                    .into_iter()
                    .last()
                    .unwrap(),
            )
        } else {
            Error::Job(e)
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        if let Error::Validation(_) = &self {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let mut res = HttpResponse::new(self.status_code());
        res.headers_mut().insert(
            HeaderName::from_static("Content-Type"),
            HeaderValue::from_static("application/json"),
        );

        let buf = if let Error::Validation(_) = &self {
            json!({
               "code": StatusCode::BAD_REQUEST.as_u16(),
                "message": self.to_string()
            })
        } else {
            json!({
               "code": StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                "message": StatusCode::INTERNAL_SERVER_ERROR.to_string()
            })
        };

        res.set_body(BoxBody::new(buf.to_string()))
    }
}

fn validator_error_string_nested(e: &ValidationErrors) -> Vec<String> {
    return e
        .errors()
        .iter()
        .map(|(k, e)| match e {
            ValidationErrorsKind::Field(ev) => ev
                .iter()
                .map(|e| format!("[{}] {}", k, e.to_string()))
                .collect(),
            ValidationErrorsKind::Struct(f) => validator_error_string_nested(f.as_ref()),
            ValidationErrorsKind::List(m) => m
                .iter()
                .map(|(_i, e)| validator_error_string_nested(e.as_ref()))
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

container!(Web {ConfigImpl, job::CtlImpl});

pub async fn init(data: Value) -> Result<(), Error> {
    let config = Arc::new(ConfigImpl::new(data));

    let file_appender = tracing_appender::rolling::daily(config.log_dir(), "web.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_max_level(Level::WARN)
        .with_env_filter(config.log_level()
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .join(","))
        .with_writer(non_blocking)
        .try_init()
        .map_err(|e| Error::Logger(e.to_string()))?;

    let job_ctl = Arc::new(
        job::CtlImpl::new(config.clone())
            .await
            .map_err(|e| Error::Job(e))?,
    );

    Web::init()
        .put("conf", config.clone())
        .put("jobCtl", job_ctl.clone());

    HttpServer::new(|| App::new().service(root).service(job_exec))
        .bind((config.server_ip(), config.server_port() as u16))
        .map_err(|e| Error::Web(e))?
        .run()
        .await
        .unwrap();

    Ok(())
}

#[get("/")]
async fn root() -> impl Responder {
    "Hello, world!"
}

#[post("/job/exec")]
async fn job_exec(param: Json<job::Arg>) -> Result<Json<Val>, Error> {
    let job_ctl: Arc<job::CtlImpl> = Web::borrow().get("jobCtl").unwrap();
    let result = job::Ctl::exec(job_ctl.as_ref(), param.0).await?;
    Ok(Json(result))
}
