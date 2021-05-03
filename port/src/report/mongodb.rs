use async_std::sync::Arc;
use itertools::Itertools;
pub use mongodb::bson::doc;
pub use mongodb::bson::Document;
pub use mongodb::Client;
pub use mongodb::Collection;
pub use mongodb::Database;
pub use mongodb::options::ClientOptions;

use chord_common::case::{CaseAssess, CaseState};
use chord_common::error::Error;
use chord_common::point::{PointAssess, PointState};
use chord_common::rerr;
use chord_common::task::{TaskAssess, TaskState};

pub struct Reporter {
    collection: Arc<Collection>,
    exec_id: String,
    task_id: String,
}

impl Reporter {
    pub async fn new<T, E>(collection: Arc<Collection>,
                           task_id: T,
                           exec_id: E) -> Result<Reporter, Error>
        where T: Into<String>, E: Into<String> {
        Ok(Reporter {
            collection,
            exec_id: exec_id.into(),
            task_id: task_id.into(),
        })
    }

    pub async fn write(&mut self, task_assess: &dyn TaskAssess) -> Result<(), Error> {
        if self.task_id != task_assess.id() {
            return rerr!("400", "task_id mismatch");
        }
        let task_doc = self.collection.find_one(doc! { "exec_id": self.exec_id.as_str(), "task_assess.id": task_assess.id()}, None).await?;
        if let None = task_doc {
            self.collection.update_one(
                doc! { "exec_id": self.exec_id.as_str()},
                doc! { "$push": {
                                    "task_assess":
                                    ta_doc(task_assess)
                                }
                            },
                None,
            ).await?;
            return Ok(());
        }

        match task_assess.state() {
            TaskState::Ok(ca_vec) | TaskState::Fail(ca_vec) => {
                self.collection.update_one(
                    doc! { "exec_id": self.exec_id.as_str(), "task_assess.id": task_assess.id()},
                    doc! { "$push": {
                                    "task_assess.$.case_assess":
                                    {
                                        "$each": ca_vec.iter().map(|ca| ca_doc(ca.as_ref())).collect_vec()
                                    }
                                }
                            },
                    None,
                ).await?;
            }
            TaskState::Err(_) => ()
        }

        return Ok(());
    }

    pub async fn close(self) -> Result<(), Error> {
        //todo 计算task state
        //todo 计算job state
        Ok(())
    }
}

fn ta_doc(ta: &dyn TaskAssess) -> Document {
    match ta.state() {
        TaskState::Ok(ca_vec) => {
            doc! {
                "id": ta.id(),
                "start": ta.start(),
                "end": ta.end(),
                "state": "O",
                "case_assess": ca_vec.iter().map(|ca| ca_doc(ca.as_ref())).collect_vec()
            }
        }
        TaskState::Fail(ca_vec) => {
            doc! {
                "id": ta.id(),
                "start": ta.start(),
                "end": ta.end(),
                "state": "F",
                "case_assess": ca_vec.iter().map(|ca| ca_doc(ca.as_ref())).collect_vec()
            }
        }
        TaskState::Err(e) => {
            doc! {
                "id": ta.id(),
                "start": ta.start(),
                "end": ta.end(),
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
                "id": ca.id() as u64,
                "start": ca.start(),
                "end": ca.end(),
                "state": "O",
                "point_assess": pa_vec.iter().map(|pa|pa_doc(pa.as_ref())).collect_vec()
            }
        }
        CaseState::Fail(pa_vec) => {
            doc! {
                "id": ca.id() as u64,
                "start": ca.start(),
                "end": ca.end(),
                "state": "F",
                "point_assess": pa_vec.iter().map(|pa|pa_doc(pa.as_ref())).collect_vec()
            }
        }
        CaseState::Err(e) => {
            doc! {
                "id": ca.id() as u64,
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



