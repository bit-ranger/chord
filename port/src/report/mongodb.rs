use chord_common::case::{CaseState, CaseAssess};
use chord_common::error::Error;
use chord_common::point::{PointState, PointAssess};
use crate::model::PortError;
use std::path::Path;
use chord_common::flow::Flow;
use std::fs::File;
use chord_common::task::{TaskAssess, TaskState};
use chord_common::err;
use async_std::sync::Arc;
use mongodb::{Collection, Client};
use mongodb::options::ClientOptions;
use mongodb::bson::{Document, to_document};
use mongodb::bson::doc;

pub struct Writer {
    collection: Collection,
    exe_id: String,
}

impl Writer {
    pub async fn new(client_options: ClientOptions,
                     flow: &Flow,
                     job_name: &str,
                     exe_id: &str) -> Result<Writer, Error> {
        // Get a handle to the deployment.
        let client = Client::with_options(client_options)?;
        let db = client.database(job_name);
        let collection = db.collection::<Document>(collection.as_str());
        collection.insert_one(job_toc(exe_id), None).await?;
        Ok(Writer {
            collection,
            exe_id: exe_id.to_owned(),
        })
    }

    pub async fn write(&self, task_assess: &dyn TaskAssess) -> Result<(), Error> {
        let task_doc = self.collection.find_one(doc! { "exe_id": self.exe_id, "task_assess.$.id": task_assess.id()}, None).await?;
        if let None = task_doc {
            self.collection.insert_one(ta_doc(task_assess), None).await?;
            return Ok(());
        }

        match ta.state() {
            TaskState::Ok(ca_vec) | TaskState::Fail(ca_vec) => {
                self.collection.update_one(
                    doc! { "exe_id": self.exe_id, "task_assess.$.id": task_assess.id()},
                    doc! { "$push": {
                                    format!("task_assess.$.{}.case_assess", task_assess.id()):
                                    ca_vec.iter().map(ca_doc).collect_vec()
                                }
                            },
                    None,
                ).await?;
            }
            TaskState::Fail(_) => ()
        }

        return Ok(());
    }

    pub async fn close(&self) -> Result<(), Error> {
        //todo 计算task state
        //todo 计算job state

    }
}


fn job_toc(exe_id: &str) -> Document {
    doc! {
        "exe_id": exe_id,
        "task_assess": vec![]
    }
}

fn ta_doc(ta: &dyn TaskAssess) -> Document {
    match ta.state() {
        TaskState::Ok(ca_vec) => {
            doc! {
                "id": ca.id(),
                "start": ca.start(),
                "end": ca.end(),
                "state": "O",
                "case_assess": ca_vec.iter().map(ca_doc).collect_vec()
            }
        }
        TaskState::Fail(ca_vec) => {
            doc! {
                "id": ca.id(),
                "start": ca.start(),
                "end": ca.end(),
                "state": "F",
                "case_assess": ca_vec.iter().map(ca_doc).collect_vec()
            }
        }
        TaskState::Err(e) => {
            doc! {
                "id": ca.id(),
                "start": ca.start(),
                "end": ca.end(),
                "state": "E",
                "error": e.to_string()
            }
        }
    }
}

fn ca_doc(ca: &dyn CaseAssess) -> Document {
    match ca.state() {
        CaseState::Ok(pa_vec) => {
            doc! {
                "id": ca.id(),
                "start": ca.start(),
                "end": ca.end(),
                "state": "O",
                "point_assess": pa_vec.iter().map(pa_doc).collect_vec()
            }
        }
        CaseState::Fail(pa_vec) => {
            doc! {
                "id": ca.id(),
                "start": ca.start(),
                "end": ca.end(),
                "state": "F",
                "point_assess": pa_vec.iter().map(pa_doc).collect_vec()
            }
        }
        CaseState::Err(e) => {
            doc! {
                "id": ca.id(),
                "start": ca.start(),
                "end": ca.end(),
                "state": "E",
                "error": e.to_string()
            }
        }
    }
}

fn pa_doc(pa: &dyn PointAssess) -> Document {
    doc! {
            "id": pa.id(),
            "start": pa.start(),
            "end": pa.end(),
            "state": match pa.state(){
               PointState::Ok(_) => "O",
               PointState::Fail(_) => "F",
               PointState::Err(_) => "E",
            }
        }
}



