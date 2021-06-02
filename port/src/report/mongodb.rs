use chrono::Utc;
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
use chord_common::task::{TaskAssess, TaskState, TaskId};
use mongodb::options::UpdateOptions;


pub struct Reporter {
    collection: Arc<Collection>,
    task_id: Arc<dyn TaskId>,
    total_task_state: TaskState,
}

impl Reporter {
    pub async fn new<T, E>(collection: Arc<Collection>,
                           task_id: Arc<dyn TaskId>) -> Result<Reporter, Error>{
        collection.update_one(
            doc! {
                "exec_id": task_id.exec_id()
            },
            doc! {
                "$set": {
                    "exec_id": task_id.exec_id(),
                    "exec_time": Utc::now(),
                    "task_assess": []
                    }
            },
            Some(UpdateOptions::builder().upsert(true).build())
        ).await?;

        Ok(Reporter {
            collection,
            task_id,
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

        let task_doc = self.collection.find_one(doc! { "exec_id": self.task_id.exec_id(), "task_assess.id": task_assess.id()}, None).await?;
        if let None = task_doc {
            self.collection.update_one(
                doc! { "exec_id": self.task_id.exec_id()},
                doc! { "$push": {
                                    "task_assess":
                                    ta_doc_init(task_assess)
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
                    doc! {
                                "$push": {
                                    "task_assess.$.case_assess": {
                                        "$each": ca_vec.iter().map(|ca| ca_doc(ca.as_ref())).collect_vec()
                                    }
                                },
                                "$set": {
                                    "task_assess.$.end": task_assess.end()
                                }
                            },
                    None,
                ).await?;
            }
            TaskState::Err(e) => {
                self.collection.update_one(
                    doc! { "exec_id": self.exec_id.as_str(), "task_assess.id": task_assess.id()},
                    doc! {
                                "$set": {
                                    "task_assess.$.state": "E",
                                    "task_assess.$.end": task_assess.end(),
                                    "task_assess.$.error": e.to_string()
                                }
                            },
                    None,
                ).await?;
            }
        }

        return Ok(());
    }

    pub async fn close(self) -> Result<(), Error> {
        let state = match self.total_task_state {
            TaskState::Ok(_) => "O",
            TaskState::Fail(_) => "F",
            TaskState::Err(_) => "E"
        };

        self.collection.update_one(
            doc! { "exec_id": self.exec_id.as_str(), "task_assess.id": self.task_id},
            doc! {"$set": {"task_assess.$.state": state}},
            None,
        ).await?;
        Ok(())
    }
}

fn ta_doc_init(ta: &dyn TaskAssess) -> Document {
    match ta.state() {
        TaskState::Ok(ca_vec) | TaskState::Fail(ca_vec) => {
            doc! {
                "id": ta.id().task_id(),
                "start": ta.start(),
                "end": ta.end(),
                "state": "R",
                "case_assess": ca_vec.iter().map(|ca| ca_doc(ca.as_ref())).collect_vec()
            }
        },
        TaskState::Err(e) => {
            doc! {
                "id": ta.id().task_id(),
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
                "id": ca.id().case_id() as u64,
                "start": ca.start(),
                "end": ca.end(),
                "state": "O",
                "point_assess": pa_vec.iter().map(|pa|pa_doc(pa.as_ref())).collect_vec()
            }
        }
        CaseState::Fail(pa_vec) => {
            doc! {
                "id": ca.id().case_id() as u64,
                "start": ca.start(),
                "end": ca.end(),
                "state": "F",
                "point_assess": pa_vec.iter().map(|pa|pa_doc(pa.as_ref())).collect_vec()
            }
        }
        CaseState::Err(e) => {
            doc! {
                "id": ca.id().case_id() as u64,
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
            "id": pa.id().point_id(),
            "start": pa.start(),
            "end": pa.end(),
            "state": match pa.state(){
               PointState::Ok(_) => "O",
               PointState::Fail(_) => "F",
               PointState::Err(_) => "E",
            }
        }
}


