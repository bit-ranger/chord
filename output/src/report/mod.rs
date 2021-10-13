#[cfg(feature = "report_csv")]
mod csv;
#[cfg(feature = "report_elasticsearch")]
mod elasticsearch;
#[cfg(feature = "report_webhook")]
mod webhook;

use async_std::sync::Arc;
use async_trait::async_trait;
use chord::err;
use chord::output::Report;
use chord::task::TaskId;
use chord::value::Value;
use chord::Error;
use std::borrow::Borrow;

#[async_trait]
pub trait Factory: Sync + Send {
    async fn create(&self, task_id: Arc<dyn TaskId>) -> Result<Box<dyn Report>, Error>;
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
                return Err(err!("report", "missing conf"));
            }
            Some(c) => {
                if !c.is_object() {
                    return Err(err!("report", "invalid conf"));
                };
                let kind = c["kind"]
                    .as_str()
                    .ok_or(err!("report", "missing report.kind"))?;

                match kind {
                    #[cfg(feature = "report_csv")]
                    "csv" => {
                        let v = c[kind].borrow();
                        let factory = csv::ReportFactory::new(
                            v["dir"]
                                .as_str()
                                .ok_or(err!("report", "missing report.csv.dir"))?,
                            name.to_string(),
                            exec_id.to_string(),
                            v["with_bom"].as_bool().unwrap_or(true),
                        )
                        .await?;
                        return Ok(ReportFactory {
                            delegate: Box::new(factory),
                        });
                    }
                    #[cfg(feature = "report_elasticsearch")]
                    "elasticsearch" => {
                        let v = c[kind].borrow();
                        let factory = elasticsearch::ReportFactory::new(
                            v["url"]
                                .as_str()
                                .ok_or(err!("report", "missing report.elasticsearch.url"))?
                                .to_string(),
                            name.to_string(),
                            exec_id.to_string(),
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
                                .ok_or(err!("report", "missing report.webhook.url"))?
                                .to_string(),
                            name.to_string(),
                            exec_id.to_string(),
                        )
                        .await?;
                        return Ok(ReportFactory {
                            delegate: Box::new(factory),
                        });
                    }
                    _ => return Err(err!("report", "invalid conf")),
                }
            }
        }
    }
}

#[async_trait]
impl Factory for ReportFactory {
    async fn create(&self, task_id: Arc<dyn TaskId>) -> Result<Box<dyn Report>, Error> {
        self.delegate.create(task_id).await
    }
}
