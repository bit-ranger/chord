use std::str::FromStr;

use async_std::sync::Arc;
use chrono::{DateTime, Utc};
use log::{error, info, trace, warn};
use surf::http::headers::{HeaderName, HeaderValue};
use surf::http::Method;
use surf::{Body, RequestBuilder, Response, Url};

use chord::case::{CaseAssess, CaseState};
use chord::err;
use chord::output::async_trait;
use chord::output::AssessReport;
use chord::step::{StepAssess, StepState};
use chord::task::{TaskAssess, TaskId, TaskState};
use chord::value::{json, to_string, Value};
use chord::value::{Deserialize, Serialize};
use chord::Error;

pub struct Reporter {
    es_url: String,
    es_index: String,
    task_id: Arc<dyn TaskId>,
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
        value: Value::Null,
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
        value: match ts {
            TaskState::Ok => Value::Null,
            TaskState::Fail => Value::Null,
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
        value: match sa.state() {
            StepState::Ok(result) | StepState::Fail(result) => result.clone(),
            StepState::Err(e) => json!({
                "code": e.code(),
                "message": e.message()
            }),
        },
    }
}

async fn data_send_all(es_url: &str, es_index: &str, data: Vec<Data>) -> Result<(), Error> {
    let path = format!("{}/{}/_doc/_bulk", es_url, es_index);
    let rb = RequestBuilder::new(Method::Put, Url::from_str(path.as_str())?);
    data_send_all_0(rb, data).await.map_err(|e| e.0)
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

    info!("index creating [{}]", index_name);
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
        let body: Value = res.body_json().await?;
        warn!(
            "data_send_0 failure: {}, {}",
            to_string(&data)?,
            to_string(&body)?
        )
    }
    Ok(())
}

async fn data_send_all_0(rb: RequestBuilder, data: Vec<Data>) -> Result<(), Rae> {
    let mut rb = rb.header(
        HeaderName::from_str("Content-Type").unwrap(),
        HeaderValue::from_str("application/json")?,
    );

    let mut body = String::new();
    for d in data {
        let act = ActionIndex {
            index: ActionMeta { _id: d.id.clone() },
        };
        body.push_str(to_string(&act)?.as_str());
        body.push_str("\n");
        body.push_str(to_string(&d)?.as_str());
        body.push_str("\n");
    }

    trace!("data_send_all_0: {}", body.escape_debug());
    rb = rb.body(Body::from_string(body));

    let mut res: Response = rb.send().await?;
    if !res.status().is_success() {
        let body: Value = res.body_json().await?;
        warn!("data_send_all_0 failure: {}", to_string(&body)?)
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

    let index = r#"
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
      "value": {
        "type": "object"
      }
    }
  }
}
"#;

    rb = rb.body(Body::from_string(index.into()));

    let mut res: Response = rb.send().await?;
    if !res.status().is_success() {
        let body: Value = res.body_json().await?;
        error!("index_create_0 failure: {}", to_string(&body)?)
    }
    Ok(())
}

struct Rae(chord::Error);

impl From<surf::Error> for Rae {
    fn from(err: surf::Error) -> Rae {
        Rae(err!("elasticsearch", format!("{}", err.status())))
    }
}

impl From<chord::value::Error> for Rae {
    fn from(err: chord::value::Error) -> Rae {
        Rae(err!(
            "serde_json",
            format!("{}:{}", err.line(), err.column())
        ))
    }
}

impl From<chord::Error> for Rae {
    fn from(err: Error) -> Self {
        Rae(err)
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ActionIndex {
    index: ActionMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ActionMeta {
    _id: String,
}
