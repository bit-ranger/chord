use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surf::{Body, RequestBuilder, Response, Url};
use surf::http::headers::{HeaderName, HeaderValue};
use surf::http::Method;

use chord_common::{err, rerr};
use chord_common::case::{CaseAssess, CaseState};
use chord_common::error::Error;
use chord_common::point::{PointAssess, PointState};
use chord_common::task::{TaskAssess, TaskState, TaskId};
use chord_common::value::{Json, to_string};
use log::warn;
use async_std::sync::Arc;

pub struct Reporter {
    es_url: String,
    es_index: String,
    task_id: Arc<dyn TaskId>,
    start: DateTime<Utc>,
    total_task_state: TaskState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Data{
    id: String,
    id_in_layer: String,
    layer: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    elapse: usize,
    state: String,
    result: Json
}

impl Reporter {

    pub async fn new(
        es_url: String,
        es_index: String,
        task_id: Arc<dyn TaskId>) -> Result<Reporter, Error> {
        let task_data = ta_doc_init(task_id.as_ref());
        send_data(es_url.as_str(), es_index.as_str(), task_data).await?;

        Ok(Reporter {
            es_url,
            es_index,
            task_id,
            start: Utc::now(),
            total_task_state: TaskState::Ok(vec![]),
        })
    }

    pub async fn state(&mut self, state: TaskState)-> Result<(), Error> {
        self.total_task_state = state;
        Ok(())
    }

    pub async fn write(&mut self, task_assess: &dyn TaskAssess) -> Result<(), Error> {

        if let TaskState::Err(_) = self.total_task_state {
            return rerr!("500", "task is error");
        }

        match task_assess.state() {
            TaskState::Ok(_) => {}
            TaskState::Fail(_) => {
                self.total_task_state = TaskState::Fail(vec![]);
            }
            TaskState::Err(e) => {
                self.total_task_state = TaskState::Err(e.clone());
            }
        }

        match task_assess.state() {
            TaskState::Ok(ca_vec) | TaskState::Fail(ca_vec) => {
                let mut data_vec: Vec<Data> = vec![];
                for ca in ca_vec {
                    let ca_data = ca_doc(ca.as_ref());
                    data_vec.push(ca_data);
                    match ca.state() {
                        CaseState::Ok(pa_vec)  | CaseState::Fail(pa_vec)=> {
                            for pa in pa_vec {
                                let pa_data = pa_doc(pa.as_ref());
                                data_vec.push(pa_data);
                            }
                        },
                        CaseState::Err(_e) => {
                            // do nothing
                        }
                    }
                }
                send_data_all(self.es_url.as_str(), self.es_index.as_str(), data_vec).await?;
            }
            TaskState::Err(_e) => {
                // do nothing
            }
        }

        return Ok(());
    }

    pub async fn close(self) -> Result<(), Error> {
        let task_data = ta_doc(self.task_id.as_ref(), self.start, &self.total_task_state);
        send_data(self.es_url.as_str(), self.es_index.as_str(), task_data).await?;
        Ok(())
    }
}

fn ta_doc_init(task_id: &dyn TaskId) -> Data {
    let now = Utc::now();
    Data {
        id: task_id.to_string(),
        id_in_layer: task_id.task_id().to_owned(),
        layer: "task".to_owned(),
        start: now,
        end: now,
        elapse: 0,
        state: "R".to_owned(),
        result: Json::Null
    }
}

fn ta_doc(task_id: &dyn TaskId, start: DateTime<Utc>, ts: &TaskState) -> Data {
    Data {
        id: task_id.to_string(),
        id_in_layer: task_id.task_id().to_owned(),
        layer: "task".to_owned(),
        start,
        end: Utc::now(),
        elapse: (Utc::now() - start).num_microseconds().unwrap_or(-1) as usize,
        state: match ts {
            TaskState::Ok(_) => "O",
            TaskState::Fail(_) => "F",
            TaskState::Err(_) => "E"
        }.to_owned(),
        result: match ts {
            TaskState::Ok(_) => Json::Null,
            TaskState::Fail(_) => Json::Null,
            TaskState::Err(e) => Json::String(e.to_string())
        }
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
            CaseState::Err(_) => "E"
        }.to_owned(),
        result: match ca.state() {
            CaseState::Ok(_) => Json::Null,
            CaseState::Fail(_) => Json::Null,
            CaseState::Err(e) => Json::String(e.to_string())
        }
    }


}

fn pa_doc(pa: &dyn PointAssess) -> Data {
    Data {
        id: pa.id().to_string(),
        id_in_layer: pa.id().point_id().to_owned(),
        layer: "point".to_owned(),
        start: pa.start(),
        end: pa.end(),
        elapse: (pa.end() - pa.start()).num_microseconds().unwrap_or(-1) as usize,
        state: match pa.state() {
            PointState::Ok(_) => "O",
            PointState::Fail(_) => "F",
            PointState::Err(_) => "E"
        }.to_owned(),
        result: match pa.state() {
            PointState::Ok(result) => Json::String(to_string(result).unwrap_or("".to_owned())),
            PointState::Fail(result) => Json::String(to_string(result).unwrap_or("".to_owned())),
            PointState::Err(e) => Json::String(e.to_string())
        }
    }
}


async fn send_data_all(es_url: &str, es_index: &str, data: Vec<Data>) -> Result<(), Error>{
    for d in data{
        send_data(es_url, es_index, d).await?
    }
    Ok(())
}

async fn send_data(es_url: &str, es_index: &str, data: Data) -> Result<(), Error>{
    let path = format!("{}/{}/_doc/{}", es_url, es_index, data.id);
    let rb = RequestBuilder::new(Method::Put, Url::from_str(path.as_str())?);
    send(rb, data).await.map_err(|e| e.0)
}

async fn send(rb: RequestBuilder, data: Data)  -> Result<(), Rae>{
    let mut rb = rb.header(
        HeaderName::from_str("Content-Type").unwrap(),
        HeaderValue::from_str("application/json")?,
    );

    rb = rb.body(Body::from_json(&data)?);

    let mut res: Response = rb.send().await?;
    if !res.status().is_success(){
        let body: Json = res.body_json().await?;
        warn!("send failure: {}, {}", to_string(&data)?, to_string(&body)?)
    }
    Ok(())
}

struct Rae(chord_common::error::Error);

impl From<surf::Error> for Rae {
    fn from(err: surf::Error) -> Rae {
        Rae(err!("http", format!("{}", err.status())))
    }
}

impl From<serde_json::error::Error> for Rae {
    fn from(err: serde_json::error::Error) -> Rae {
        Rae(err!("serde_json", format!("{}:{}", err.line(), err.column())))
    }
}

impl From<chord_common::error::Error> for Rae {
    fn from(err: Error) -> Self {
        Rae(err)
    }
}

