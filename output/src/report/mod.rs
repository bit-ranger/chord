use std::borrow::Borrow;
use std::sync::Arc;

use async_trait::async_trait;

use chord_core::flow::Flow;
use chord_core::output::{Error, Report};
use chord_core::task::TaskId;
use chord_core::value::Value;
use ReportError::*;

#[cfg(feature = "report_csv")]
mod csv;
#[cfg(feature = "report_webhook")]
mod webhook;

#[derive(thiserror::Error, Debug)]
enum ReportError {
    #[error("conf lost")]
    ConfLost,

    #[error("conf invalid")]
    ConfInvalid,

    #[error("conf lost entry `{0}`")]
    ConfLostEntry(String),
}

#[async_trait]
pub trait Factory: Sync + Send {
    async fn create(
        &self,
        task_id: Arc<dyn TaskId>,
        flow: Arc<Flow>,
    ) -> Result<Box<dyn Report>, Error>;
}

pub struct ReportFactory {
    delegate: Box<dyn Factory>,
}

impl ReportFactory {
    pub async fn new(
        conf: Option<&Value>,
        name: &str,
        exec_id: &str,
    ) -> Result<ReportFactory, Error> {
        match conf {
            None => {
                return Err(Box::new(ConfLost));
            }
            Some(c) => {
                if !c.is_object() {
                    return Err(Box::new(ConfInvalid));
                };
                let kind = c["kind"]
                    .as_str()
                    .ok_or(ConfLostEntry("report.kind".into()))?;

                match kind {
                    #[cfg(feature = "report_csv")]
                    "csv" => {
                        let v = c[kind].borrow();
                        let factory = csv::ReportFactory::new(
                            v["dir"]
                                .as_str()
                                .ok_or(ConfLostEntry("report.csv.dir".into()))?,
                            name.to_string(),
                            exec_id.to_string(),
                            v["with_bom"].as_bool().unwrap_or(true),
                        )
                        .await?;
                        return Ok(ReportFactory {
                            delegate: Box::new(factory),
                        });
                    }
                    #[cfg(feature = "report_webhook")]
                    "webhook" => {
                        let v = c[kind].borrow();
                        let factory = webhook::ReportFactory::new(
                            v["url"]
                                .as_str()
                                .ok_or(ConfLostEntry("report.webhook.url".into()))?
                                .to_string(),
                            name.to_string(),
                            exec_id.to_string(),
                        )
                        .await?;
                        return Ok(ReportFactory {
                            delegate: Box::new(factory),
                        });
                    }
                    _ => return Err(Box::new(ConfInvalid)),
                }
            }
        }
    }
}

#[async_trait]
impl Factory for ReportFactory {
    async fn create(
        &self,
        task_id: Arc<dyn TaskId>,
        flow: Arc<Flow>,
    ) -> Result<Box<dyn Report>, Error> {
        self.delegate.create(task_id, flow).await
    }
}
