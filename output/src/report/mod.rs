use std::borrow::Borrow;
use std::sync::Arc;

use async_trait::async_trait;

use chord_core::flow::Flow;
use chord_core::output::{Error, JobReporter, TaskReporter};
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

    #[error("invalid {0}: {1}")]
    ConfInvalid(String, String),

    #[error("conf lost entry `{0}`")]
    ConfLostEntry(String),
}

pub struct DefaultJobReporter {
    delegate: Box<dyn JobReporter>,
}

impl DefaultJobReporter {
    pub async fn new(
        conf: Option<&Value>,
        name: &str,
        exec_id: &str,
    ) -> Result<DefaultJobReporter, Error> {
        match conf {
            None => {
                return Err(Box::new(ConfLost));
            }
            Some(c) => {
                if !c.is_object() {
                    return Err(Box::new(ConfInvalid("conf".into(), c.to_string())));
                };
                let kind = c["kind"]
                    .as_str()
                    .ok_or(ConfLostEntry("report.kind".into()))?;

                match kind {
                    #[cfg(feature = "report_csv")]
                    "csv" => {
                        let v = c[kind].borrow();
                        let factory = csv::CsvJobReporter::new(
                            v["dir"]
                                .as_str()
                                .ok_or(ConfLostEntry("report.csv.dir".into()))?,
                            name.to_string(),
                            exec_id.to_string(),
                            v["with_bom"].as_bool().unwrap_or(true),
                        )
                        .await?;
                        return Ok(DefaultJobReporter {
                            delegate: Box::new(factory),
                        });
                    }
                    #[cfg(feature = "report_webhook")]
                    "webhook" => {
                        let v = c[kind].borrow();
                        let factory = webhook::WebhookJobReporter::new(
                            v["url"]
                                .as_str()
                                .ok_or(ConfLostEntry("report.webhook.url".into()))?
                                .to_string(),
                            name.to_string(),
                            exec_id.to_string(),
                        )
                        .await?;
                        return Ok(DefaultJobReporter {
                            delegate: Box::new(factory),
                        });
                    }
                    other => {
                        return Err(Box::new(ConfInvalid("kind".to_string(), other.to_string())))
                    }
                }
            }
        }
    }
}

#[async_trait]
impl JobReporter for DefaultJobReporter {
    async fn task(
        &self,
        task_id: Arc<dyn TaskId>,
        flow: Arc<Flow>,
    ) -> Result<Box<dyn TaskReporter>, Error> {
        self.delegate.task(task_id, flow).await
    }
}
