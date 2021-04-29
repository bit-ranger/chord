use chord_common::case::{CaseState, CaseAssess};
use chord_common::error::Error;
use chord_common::point::{PointState, PointAssess};
use chord_common::task::{TaskAssess, TaskState};
use mongodb::{Collection, Client};
pub use mongodb::options::ClientOptions;
use mongodb::bson::{Document};
use mongodb::bson::doc;
use itertools::Itertools;

pub struct Writer {
    collection: Collection,
    exec_id: String,
}

impl Writer {
    pub async fn new(client_options: ClientOptions,
                     job_name: &str,
                     exec_id: &str) -> Result<Writer, Error> {
        // Get a handle to the deployment.
        let db_name = client_options.credential
            .as_ref()
            .map(|c|
                c.source.as_ref().map(|s| s.clone()).unwrap_or("chord".to_owned()))
            .unwrap();
        let client = Client::with_options(client_options)?;
        let db = client.database(db_name.as_str());
        let collection = db.collection::<Document>(job_name);
        collection.insert_one(job_toc(exec_id), None).await?;
        Ok(Writer {
            collection,
            exec_id: exec_id.to_owned(),
        })
    }

    pub async fn write(&self, task_assess: &dyn TaskAssess) -> Result<(), Error> {
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

    pub async fn close(&self) -> Result<(), Error> {
        //todo 计算task state
        //todo 计算job state
        Ok(())
    }
}


fn job_toc(exec_id: &str) -> Document {
    doc! {
        "exec_id": exec_id,
        "task_assess": []
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



