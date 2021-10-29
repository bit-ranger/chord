use std::str::FromStr;

use async_std::sync::Arc;
use chrono::{DateTime, Utc};
use log::{info, trace, warn};
use surf::http::headers::{HeaderName, HeaderValue};
use surf::http::Method;
use surf::{Body, RequestBuilder, Response, Url};

use crate::report::Factory;
use chord::case::{CaseAssess, CaseState};
use chord::err;
use chord::flow::Flow;
use chord::output::async_trait;
use chord::output::Report;
use chord::step::{StepAssess, StepState};
use chord::task::{TaskAssess, TaskId, TaskState};
use chord::value::{json, to_value, Value};
use chord::value::{Deserialize, Serialize};
use chord::Error;

pub struct ReportFactory {
    url: String,
    index: String,
}

#[async_trait]
impl Factory for ReportFactory {
    async fn create(&self, task_id: Arc<dyn TaskId>) -> Result<Box<dyn Report>, Error> {
        let reporter = ReportFactory::create(self, task_id).await?;
        Ok(Box::new(reporter))
    }
}

impl ReportFactory {
    pub async fn new(url: String, job_name: String, _: String) -> Result<ReportFactory, Error> {
        index_create(url.as_str(), job_name.as_str()).await?;
        Ok(ReportFactory {
            url,
            index: job_name,
        })
    }

    pub async fn create(&self, task_id: Arc<dyn TaskId>) -> Result<Reporter, Error> {
        Reporter::new(self.url.clone(), self.index.clone(), task_id).await
    }
}

pub struct Reporter {
    url: String,
    index: String,
    task_id: Arc<dyn TaskId>,
}

#[async_trait]
impl Report for Reporter {
    async fn start(&mut self, time: DateTime<Utc>, _: Arc<Flow>) -> Result<(), Error> {
        let task_data = ta_doc_init(self.task_id.as_ref(), time);
        data_send(self.url.as_str(), self.index.as_str(), task_data).await?;
        Ok(())
    }

    async fn report(&mut self, ca_vec: &Vec<Box<dyn CaseAssess>>) -> Result<(), Error> {
        let mut data_vec: Vec<Data> = vec![];
        for ca in ca_vec {
            let ca_data = ca_doc(ca.as_ref());
            data_vec.push(ca_data);
            match ca.state() {
                CaseState::Ok(pa_vec) | CaseState::Fail(pa_vec) => {
                    for pa in pa_vec.iter() {
                        let pa_data = sa_doc(pa.as_ref());
                        data_vec.push(pa_data);
                    }
                }
                CaseState::Err(_e) => {
                    // do nothing
                }
            }
        }
        data_send_batch(self.url.as_str(), self.index.as_str(), data_vec).await
    }

    async fn end(&mut self, task_assess: &dyn TaskAssess) -> Result<(), Error> {
        let task_data = ta_doc(
            self.task_id.as_ref(),
            task_assess.start(),
            task_assess.end(),
            task_assess.state(),
        );
        data_send(self.url.as_str(), self.index.as_str(), task_data).await?;
        Ok(())
    }
}

impl Reporter {
    async fn new(
        es_url: String,
        es_index: String,
        task_id: Arc<dyn TaskId>,
    ) -> Result<Reporter, Error> {
        Ok(Reporter {
            url: es_url,
            index: es_index,
            task_id,
        })
    }
}

fn ta_doc_init(task_id: &dyn TaskId, time: DateTime<Utc>) -> Data {
    Data {
        id: task_id.to_string(),
        id_in_layer: task_id.task().to_owned(),
        layer: "task".to_owned(),
        start: time,
        end: time,
        elapse: 0,
        state: "R".to_owned(),
        value: Value::Null,
    }
}

fn ta_doc(task_id: &dyn TaskId, start: DateTime<Utc>, end: DateTime<Utc>, ts: &TaskState) -> Data {
    Data {
        id: task_id.to_string(),
        id_in_layer: task_id.task().to_owned(),
        layer: "task".to_owned(),
        start,
        end,
        elapse: (end - start).num_milliseconds() as usize,
        state: match ts {
            TaskState::Ok => "O",
            TaskState::Fail(_) => "F",
            TaskState::Err(_) => "E",
        }
        .to_owned(),
        value: match ts {
            TaskState::Ok => Value::Null,
            TaskState::Fail(_) => Value::Null,
            TaskState::Err(e) => json!({
                "code": e.code(),
                "message": e.message()
            }),
        },
    }
}

fn ca_doc(ca: &dyn CaseAssess) -> Data {
    Data {
        id: ca.id().to_string(),
        id_in_layer: ca.id().case().to_string(),
        layer: "case".to_owned(),
        start: ca.start(),
        end: ca.end(),
        elapse: (ca.end() - ca.start()).num_milliseconds() as usize,
        state: match ca.state() {
            CaseState::Ok(_) => "O",
            CaseState::Fail(_) => "F",
            CaseState::Err(_) => "E",
        }
        .to_owned(),
        value: match ca.state() {
            CaseState::Ok(_) => Value::Null,
            CaseState::Fail(_) => Value::Null,
            CaseState::Err(e) => json!({
                "code": e.code(),
                "message": e.message()
            }),
        },
    }
}

fn sa_doc(sa: &dyn StepAssess) -> Data {
    Data {
        id: sa.id().to_string(),
        id_in_layer: sa.id().step().to_owned(),
        layer: "step".to_owned(),
        start: sa.start(),
        end: sa.end(),
        elapse: (sa.end() - sa.start()).num_milliseconds() as usize,
        state: match sa.state() {
            StepState::Ok(_) => "O",
            StepState::Fail(_) => "F",
            StepState::Err(_) => "E",
        }
        .to_owned(),
        value: match sa.state() {
            StepState::Ok(scope) | StepState::Fail(scope) => scope.as_value().clone(),
            StepState::Err(e) => json!({
                "code": e.code(),
                "message": e.message()
            }),
        },
    }
}

async fn data_send_batch(url: &str, index: &str, data: Vec<Data>) -> Result<(), Error> {
    let path = format!("{}/chord/webhook", url);
    let rb = RequestBuilder::new(Method::Post, Url::from_str(path.as_str())?);

    let event = Event {
        kind: "data_send_batch".to_string(),
        object: json!({
            "index": index,
            "data": data
        }),
    };
    let data = to_value(&event)?;
    data_send_0(rb, data).await.map_err(|e| e.0)
}

async fn data_send(url: &str, index: &str, data: Data) -> Result<(), Error> {
    let path = format!("{}/chord/webhook", url);
    let rb = RequestBuilder::new(Method::Post, Url::from_str(path.as_str())?);

    let event = Event {
        kind: "data_send".to_string(),
        object: json!({
            "index": index,
            "data": data
        }),
    };
    let data = to_value(&event)?;
    data_send_0(rb, data).await.map_err(|e| e.0)
}

async fn index_create(url: &str, index_name: &str) -> Result<(), Error> {
    info!("index_create [{}]", index_name);
    let path = format!("{}/chord/webhook", url);
    let rb = RequestBuilder::new(Method::Post, Url::from_str(path.as_str())?);
    let event = Event {
        kind: "index_create".to_string(),
        object: Value::String(index_name.to_string()),
    };
    let data = to_value(&event)?;
    data_send_0(rb, data).await.map_err(|e| e.0)
}

async fn data_send_0(rb: RequestBuilder, data: Value) -> Result<(), ElasticError> {
    let mut rb = rb.header(
        HeaderName::from_str("Content-Type").unwrap(),
        HeaderValue::from_str("application/json")?,
    );

    trace!("data_send: {}", &data);
    rb = rb.body(Body::from_json(&data)?);

    let mut res: Response = rb.send().await?;
    if !res.status().is_success() {
        let body = res.body_string().await?;
        warn!("data_send failure: {}, {}", data, body)
    }
    Ok(())
}

struct ElasticError(chord::Error);

impl From<surf::Error> for ElasticError {
    fn from(err: surf::Error) -> ElasticError {
        ElasticError(err!("webhook", format!("{}", err.status())))
    }
}

impl From<chord::value::Error> for ElasticError {
    fn from(err: chord::value::Error) -> ElasticError {
        ElasticError(err!(
            "serde_json",
            format!("{}:{}", err.line(), err.column())
        ))
    }
}

impl From<chord::Error> for ElasticError {
    fn from(err: Error) -> Self {
        ElasticError(err)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Event {
    kind: String,
    object: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Data {
    id: String,
    id_in_layer: String,
    layer: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    elapse: usize,
    state: String,
    value: Value,
}
