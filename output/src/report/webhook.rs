use std::str::FromStr;

use async_std::sync::Arc;
use chrono::{DateTime, Utc};
use log::{info, trace, warn};
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Client, Method, RequestBuilder, Response, Url};

use chord_core::case::{CaseAssess, CaseState};
use chord_core::flow::Flow;
use chord_core::output::JobReporter;
use chord_core::output::{async_trait, Error};
use chord_core::output::{StageReporter, TaskReporter};
use chord_core::step::{StepAssess, StepState};
use chord_core::task::{StageAssess, TaskAssess, TaskId, TaskState};
use chord_core::value::{json, to_value, Value};
use chord_core::value::{Deserialize, Serialize};

pub struct WebhookJobReporter {
    url: String,
    index: String,
    client: Client,
}

#[async_trait]
impl JobReporter for WebhookJobReporter {
    async fn task(
        &self,
        task_id: Arc<dyn TaskId>,
        _: Arc<Flow>,
    ) -> Result<Box<dyn TaskReporter>, Error> {
        let reporter = WebhookTaskReporter::new(
            self.client.clone(),
            self.url.clone(),
            self.index.clone(),
            task_id,
        )
        .await?;
        Ok(Box::new(reporter))
    }
}

impl WebhookJobReporter {
    pub async fn new(
        url: String,
        job_name: String,
        _: String,
    ) -> Result<WebhookJobReporter, Error> {
        let client = Client::new();
        index_create(client.clone(), url.as_str(), job_name.as_str()).await?;
        Ok(WebhookJobReporter {
            url,
            index: job_name,
            client,
        })
    }
}

pub struct WebhookTaskReporter {
    url: String,
    index: String,
    task_id: Arc<dyn TaskId>,
    client: Client,
}

impl WebhookTaskReporter {
    async fn new(
        client: Client,
        es_url: String,
        es_index: String,
        task_id: Arc<dyn TaskId>,
    ) -> Result<WebhookTaskReporter, Error> {
        Ok(WebhookTaskReporter {
            client,
            url: es_url,
            index: es_index,
            task_id,
        })
    }
}

#[async_trait]
impl TaskReporter for WebhookTaskReporter {
    async fn stage(&self, _: &str) -> Result<Box<dyn StageReporter>, Error> {
        let reporter =
            WebhookStageReporter::new(self.client.clone(), self.url.clone(), self.index.clone())
                .await?;
        Ok(Box::new(reporter))
    }

    async fn start(&mut self, time: DateTime<Utc>) -> Result<(), Error> {
        let task_data = ta_doc_init(self.task_id.as_ref(), time);
        data_send(
            self.client.clone(),
            self.url.as_str(),
            self.index.as_str(),
            task_data,
        )
        .await?;
        Ok(())
    }

    async fn end(&mut self, task_assess: &dyn TaskAssess) -> Result<(), Error> {
        let task_data = ta_doc(
            self.task_id.as_ref(),
            task_assess.start(),
            task_assess.end(),
            task_assess.state(),
        );
        data_send(
            self.client.clone(),
            self.url.as_str(),
            self.index.as_str(),
            task_data,
        )
        .await?;
        Ok(())
    }
}

pub struct WebhookStageReporter {
    url: String,
    index: String,
    client: Client,
}

impl WebhookStageReporter {
    async fn new(
        client: Client,
        es_url: String,
        es_index: String,
    ) -> Result<WebhookStageReporter, Error> {
        Ok(WebhookStageReporter {
            client,
            url: es_url,
            index: es_index,
        })
    }
}

#[async_trait]
impl StageReporter for WebhookStageReporter {
    async fn start(&mut self, _: DateTime<Utc>) -> Result<(), Error> {
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
        data_send_batch(
            self.client.clone(),
            self.url.as_str(),
            self.index.as_str(),
            data_vec,
        )
        .await
    }

    async fn end(&mut self, _: &dyn StageAssess) -> Result<(), Error> {
        Ok(())
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
            TaskState::Err(e) => Value::String(e.to_string()),
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
            CaseState::Err(e) => Value::String(e.to_string()),
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
            StepState::Err(e) => Value::String(e.to_string()),
        },
    }
}

async fn data_send_batch(
    client: Client,
    url: &str,
    index: &str,
    data: Vec<Data>,
) -> Result<(), Error> {
    let path = format!("{}/chord/webhook", url);
    let rb = client.request(Method::POST, Url::from_str(path.as_str())?);

    let event = Event {
        kind: "data_send_batch".to_string(),
        object: json!({
            "index": index,
            "data": data
        }),
    };
    let data = to_value(&event)?;
    data_send_0(rb, data).await
}

async fn data_send(client: Client, url: &str, index: &str, data: Data) -> Result<(), Error> {
    let path = format!("{}/chord/webhook", url);
    let rb = client.request(Method::POST, Url::from_str(path.as_str())?);

    let event = Event {
        kind: "data_send".to_string(),
        object: json!({
            "index": index,
            "data": data
        }),
    };
    let data = to_value(&event)?;
    data_send_0(rb, data).await
}

async fn index_create(client: Client, url: &str, index_name: &str) -> Result<(), Error> {
    info!("index_create [{}]", index_name);
    let path = format!("{}/chord/webhook", url);
    let rb = client.request(Method::POST, Url::from_str(path.as_str())?);
    let event = Event {
        kind: "index_create".to_string(),
        object: Value::String(index_name.to_string()),
    };
    let data = to_value(&event)?;
    data_send_0(rb, data).await
}

async fn data_send_0(rb: RequestBuilder, data: Value) -> Result<(), Error> {
    let mut rb = rb.header(
        HeaderName::from_str("Content-Type").unwrap(),
        HeaderValue::from_str("application/json").unwrap(),
    );

    trace!("data_send: {}", &data);
    rb = rb.body(data.to_string());

    let res: Response = rb.send().await?;
    if !res.status().is_success() {
        let body = res.text().await?;
        warn!("data_send failure: {}, {}", data, body)
    }
    Ok(())
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
