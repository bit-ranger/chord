use std::str::FromStr;

use async_std::sync::Arc;
use chrono::{DateTime, Utc};
use log::{trace, warn};
use serde::{Deserialize, Serialize};
use surf::http::headers::{HeaderName, HeaderValue};
use surf::http::Method;
use surf::{Body, RequestBuilder, Response, Url};

use chord_common::case::{CaseAssess, CaseState};
use chord_common::err;
use chord_common::error::Error;
use chord_common::output::async_trait;
use chord_common::output::AssessReport;
use chord_common::step::{StepAssess, StepState};
use chord_common::task::{TaskAssess, TaskId, TaskState};
use chord_common::value::{to_string, Json};

pub struct Reporter {
    es_url: String,
    es_index: String,
    task_id: Arc<dyn TaskId>,
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
    result: Json,
}

#[async_trait]
impl AssessReport for Reporter {
    async fn start(&mut self, time: DateTime<Utc>) -> Result<(), Error> {
        let task_data = ta_doc_init(self.task_id.as_ref(), time);
        data_send(self.es_url.as_str(), self.es_index.as_str(), task_data).await?;
        Ok(())
    }

    async fn report(&mut self, _: &str, ca_vec: &Vec<Box<dyn CaseAssess>>) -> Result<(), Error> {
        let mut data_vec: Vec<Data> = vec![];
        for ca in ca_vec {
            let ca_data = ca_doc(ca.as_ref());
            data_vec.push(ca_data);
            match ca.state() {
                CaseState::Ok(pa_vec) | CaseState::Fail(pa_vec) => {
                    for pa in pa_vec {
                        let pa_data = sa_doc(pa.as_ref());
                        data_vec.push(pa_data);
                    }
                }
                CaseState::Err(_e) => {
                    // do nothing
                }
            }
        }
        data_send_all(self.es_url.as_str(), self.es_index.as_str(), data_vec).await
    }

    async fn end(&mut self, task_assess: &dyn TaskAssess) -> Result<(), Error> {
        let task_data = ta_doc(
            self.task_id.as_ref(),
            task_assess.start(),
            task_assess.end(),
            task_assess.state(),
        );
        data_send(self.es_url.as_str(), self.es_index.as_str(), task_data).await?;
        Ok(())
    }
}

impl Reporter {
    pub async fn new(
        es_url: String,
        es_index: String,
        task_id: Arc<dyn TaskId>,
    ) -> Result<Reporter, Error> {
        Ok(Reporter {
            es_url,
            es_index,
            task_id,
        })
    }

    pub async fn close(self) -> Result<(), Error> {
        Ok(())
    }
}

fn ta_doc_init(task_id: &dyn TaskId, time: DateTime<Utc>) -> Data {
    Data {
        id: task_id.to_string(),
        id_in_layer: task_id.task_id().to_owned(),
        layer: "task".to_owned(),
        start: time,
        end: time,
        elapse: 0,
        state: "R".to_owned(),
        result: Json::Null,
    }
}

fn ta_doc(task_id: &dyn TaskId, start: DateTime<Utc>, end: DateTime<Utc>, ts: &TaskState) -> Data {
    Data {
        id: task_id.to_string(),
        id_in_layer: task_id.task_id().to_owned(),
        layer: "task".to_owned(),
        start,
        end,
        elapse: (Utc::now() - start).num_microseconds().unwrap_or(-1) as usize,
        state: match ts {
            TaskState::Ok => "O",
            TaskState::Fail => "F",
            TaskState::Err(_) => "E",
        }
        .to_owned(),
        result: match ts {
            TaskState::Ok => Json::Null,
            TaskState::Fail => Json::Null,
            TaskState::Err(e) => Json::String(e.to_string()),
        },
    }
}

fn ca_doc(ca: &dyn CaseAssess) -> Data {
    Data {
        id: ca.id().to_string(),
        id_in_layer: ca.id().case_id().to_string(),
        layer: "case".to_owned(),
        start: ca.start(),
        end: ca.end(),
        elapse: (ca.end() - ca.start()).num_microseconds().unwrap_or(-1) as usize,
        state: match ca.state() {
            CaseState::Ok(_) => "O",
            CaseState::Fail(_) => "F",
            CaseState::Err(_) => "E",
        }
        .to_owned(),
        result: match ca.state() {
            CaseState::Ok(_) => Json::Null,
            CaseState::Fail(_) => Json::Null,
            CaseState::Err(e) => Json::String(e.to_string()),
        },
    }
}

fn sa_doc(sa: &dyn StepAssess) -> Data {
    Data {
        id: sa.id().to_string(),
        id_in_layer: sa.id().step_id().to_owned(),
        layer: "step".to_owned(),
        start: sa.start(),
        end: sa.end(),
        elapse: (sa.end() - sa.start()).num_microseconds().unwrap_or(-1) as usize,
        state: match sa.state() {
            StepState::Ok(_) => "O",
            StepState::Fail(_) => "F",
            StepState::Err(_) => "E",
        }
        .to_owned(),
        result: match sa.state() {
            StepState::Ok(result) => Json::String(to_string(result).unwrap_or("".to_owned())),
            StepState::Fail(result) => Json::String(to_string(result).unwrap_or("".to_owned())),
            StepState::Err(e) => Json::String(e.to_string()),
        },
    }
}

async fn data_send_all(es_url: &str, es_index: &str, data: Vec<Data>) -> Result<(), Error> {
    for d in data {
        data_send(es_url, es_index, d).await?
    }
    Ok(())
}

async fn data_send(es_url: &str, es_index: &str, data: Data) -> Result<(), Error> {
    let path = format!("{}/{}/_doc/{}", es_url, es_index, data.id);
    let rb = RequestBuilder::new(Method::Put, Url::from_str(path.as_str())?);
    data_send_0(rb, data).await.map_err(|e| e.0)
}

pub async fn index_create(es_url: &str, index_name: &str) -> Result<(), Error> {
    let path = format!("{}/{}", es_url, index_name);
    let rb = RequestBuilder::new(Method::Get, Url::from_str(path.as_str())?);
    let res = empty_send_0(rb).await.map_err(|e| e.0)?;
    if res.status().is_success() {
        trace!("index [{}] exist, ignore", index_name);
        return Ok(());
    }

    let rb = RequestBuilder::new(Method::Put, Url::from_str(path.as_str())?);
    index_create_0(rb).await.map_err(|e| e.0)
}

async fn data_send_0(rb: RequestBuilder, data: Data) -> Result<(), Rae> {
    let mut rb = rb.header(
        HeaderName::from_str("Content-Type").unwrap(),
        HeaderValue::from_str("application/json")?,
    );

    rb = rb.body(Body::from_json(&data)?);

    let mut res: Response = rb.send().await?;
    if !res.status().is_success() {
        let body: Json = res.body_json().await?;
        warn!(
            "data_send_0 failure: {}, {}",
            to_string(&data)?,
            to_string(&body)?
        )
    }
    Ok(())
}

async fn empty_send_0(rb: RequestBuilder) -> Result<Response, Rae> {
    let rb = rb.header(
        HeaderName::from_str("Content-Type").unwrap(),
        HeaderValue::from_str("application/json")?,
    );

    let res: Response = rb.send().await?;

    Ok(res)
}

async fn index_create_0(rb: RequestBuilder) -> Result<(), Rae> {
    let mut rb = rb.header(
        HeaderName::from_str("Content-Type").unwrap(),
        HeaderValue::from_str("application/json")?,
    );

    let index: Json = r#"
{
  "settings": {
    "index": {
      "analysis.analyzer.default.type": "ik_max_word"
    }
  },
  "mappings": {
    "properties": {
      "id": {
        "type": "text"
      },
      "id_in_layer": {
        "type": "keyword"
      },
      "layer": {
        "type": "keyword"
      },
      "start": {
        "type": "date"
      },
      "end": {
        "type": "date"
      },
      "elapse": {
        "type": "long"
      },
      "state": {
        "type": "keyword"
      },
      "result": {
        "type": "text",
        "analyzer": "ik_max_word",
        "search_analyzer": "ik_max_word"
      }
    }
  }
}
"#
    .into();

    rb = rb.body(Body::from_json(&index)?);

    let mut res: Response = rb.send().await?;
    if !res.status().is_success() {
        let body: Json = res.body_json().await?;
        warn!("index_create_0 failure: {}", to_string(&body)?)
    }
    Ok(())
}

struct Rae(chord_common::error::Error);

impl From<surf::Error> for Rae {
    fn from(err: surf::Error) -> Rae {
        Rae(err!("elasticsearch", format!("{}", err.status())))
    }
}

impl From<serde_json::error::Error> for Rae {
    fn from(err: serde_json::error::Error) -> Rae {
        Rae(err!(
            "serde_json",
            format!("{}:{}", err.line(), err.column())
        ))
    }
}

impl From<chord_common::error::Error> for Rae {
    fn from(err: Error) -> Self {
        Rae(err)
    }
}
