#[cfg(feature = "report_csv")]
mod csv;
#[cfg(feature = "report_elasticsearch")]
mod elasticsearch;

use async_std::sync::Arc;
use async_trait::async_trait;
use chord::err;
use chord::output::Report;
use chord::task::TaskId;
use chord::value::Value;
use chord::Error;

#[async_trait]
pub trait Factory: Sync + Send {
    async fn create(&self, task_id: Arc<dyn TaskId>) -> Result<Box<dyn Report>, Error>;
}

pub struct ReportFactory {
    delegate: Box<dyn Factory>,
}

impl ReportFactory {
    pub async fn new(conf: Option<&Value>, name: &str) -> Result<ReportFactory, Error> {
        match conf {
            None => {
                return Err(err!("report", "missing conf"));
            }
            Some(c) => {
                if !c.is_object() {
                    return Err(err!("report", "invalid conf"));
                };
                let c = c.as_object().unwrap();
                for (k, v) in c {
                    match k.as_str() {
                        #[cfg(feature = "report_csv")]
                        "csv" => {
                            let factory = csv::ReportFactory::new(
                                v["dir"]
                                    .as_str()
                                    .ok_or(err!("report", "missing report.csv.dir"))?,
                            )
                            .await?;
                            return Ok(ReportFactory {
                                delegate: Box::new(factory),
                            });
                        }
                        #[cfg(feature = "report_elasticsearch")]
                        "elasticsearch" => {
                            let factory = elasticsearch::ReportFactory::new(
                                v["url"]
                                    .as_str()
                                    .ok_or(err!("report", "missing report.elasticsearch.url"))?
                                    .to_string(),
                                name.to_string(),
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
        return Err(err!("report", "missing conf"));
    }
}

#[async_trait]
impl Factory for ReportFactory {
    async fn create(&self, task_id: Arc<dyn TaskId>) -> Result<Box<dyn Report>, Error> {
        self.delegate.create(task_id).await
    }
}
