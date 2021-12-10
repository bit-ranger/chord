use std::io::Write;

use async_std::fs::create_dir_all;
use async_std::fs::read_dir;
use async_std::fs::remove_file;
use async_std::fs::rename;
use async_std::path::{Path, PathBuf};
use async_std::sync::Arc;
use chrono::{DateTime, Utc};
use csv::Writer;
use futures::StreamExt;

use chord_core::case::{CaseAssess, CaseState};
use chord_core::flow::Flow;
use chord_core::output::async_trait;
use chord_core::output::Error;
use chord_core::output::Report;
use chord_core::step::StepState;
use chord_core::task::{TaskAssess, TaskId, TaskState};
use chord_core::value::{to_string_pretty, Value};

use crate::report::Factory;

pub struct ReportFactory {
    dir: PathBuf,
    with_bom: bool,
}

#[async_trait]
impl Factory for ReportFactory {
    async fn create(&self, task_id: Arc<dyn TaskId>) -> Result<Box<dyn Report>, Error> {
        let factory = ReportFactory::create(self, task_id).await?;
        Ok(Box::new(factory))
    }
}

impl ReportFactory {
    pub async fn new<P: AsRef<Path>>(
        report_dir: P,
        name: String,
        exec_id: String,
        with_bom: bool,
    ) -> Result<ReportFactory, Error> {
        let report_dir = report_dir.as_ref().join(name).join(exec_id);
        if !report_dir.exists().await {
            create_dir_all(report_dir.as_path()).await?;
        } else {
            let mut rd = read_dir(report_dir.clone()).await?;
            loop {
                let rf = rd.next().await;
                if rf.is_none() {
                    break;
                }
                let rf = rf.unwrap()?;
                if rf.path().is_file().await {
                    remove_file(rf.path()).await?;
                }
            }
        }

        Ok(ReportFactory {
            dir: report_dir.clone(),
            with_bom,
        })
    }

    pub async fn create(&self, task_id: Arc<dyn TaskId>) -> Result<Reporter, Error> {
        Reporter::new(self.dir.clone(), task_id, self.with_bom).await
    }
}

pub struct Reporter {
    writer: Writer<std::fs::File>,
    report_dir: PathBuf,
    task_id: Arc<dyn TaskId>,
}

#[async_trait]
impl Report for Reporter {
    async fn start(&mut self, _: DateTime<Utc>, _: Arc<Flow>) -> Result<(), Error> {
        prepare(&mut self.writer).await?;
        Ok(())
    }

    async fn report(&mut self, ca_vec: &Vec<Box<dyn CaseAssess>>) -> Result<(), Error> {
        report(&mut self.writer, ca_vec).await
    }

    async fn end(&mut self, task_assess: &dyn TaskAssess) -> Result<(), Error> {
        let task_state_view = match task_assess.state() {
            TaskState::Ok => "O",
            TaskState::Err(_) => "E",
            TaskState::Fail(_) => "F",
        };

        let report_file = self
            .report_dir
            .join(format!("{}_result.csv", self.task_id.task()));
        let report_file_new = self.report_dir.join(format!(
            "{}_result_{}.csv",
            self.task_id.task(),
            task_state_view
        ));
        rename(report_file, report_file_new).await?;
        Ok(())
    }
}

impl Reporter {
    pub async fn new<P: AsRef<Path>>(
        report_dir: P,
        task_id: Arc<dyn TaskId>,
        with_bom: bool,
    ) -> Result<Reporter, Error> {
        let report_dir = PathBuf::from(report_dir.as_ref());

        let report_file = report_dir.join(format!("{}_result.csv", task_id.task()));

        let report = Reporter {
            writer: from_path(report_file, with_bom).await?,
            report_dir,
            task_id,
        };
        Ok(report)
    }
}

async fn from_path<P: AsRef<Path>>(
    path: P,
    with_bom: bool,
) -> Result<Writer<std::fs::File>, Error> {
    let mut file = std::fs::File::create(path.as_ref().to_str().unwrap())?;
    if with_bom {
        file.write_all("\u{feff}".as_bytes())?;
    }
    Ok(csv::WriterBuilder::new().from_writer(file))
}

async fn prepare<W: std::io::Write>(writer: &mut Writer<W>) -> Result<(), Error> {
    writer.write_record(create_head())?;
    Ok(())
}

fn create_head() -> Vec<String> {
    let mut vec: Vec<String> = vec![];
    vec.push(String::from("id"));
    vec.push(String::from("layer"));
    vec.push(String::from("state"));
    vec.push(String::from("value"));
    vec.push(String::from("explain"));
    vec.push(String::from("start"));
    vec.push(String::from("end"));
    vec
}

async fn report<W: std::io::Write>(
    writer: &mut Writer<W>,
    ca_vec: &Vec<Box<dyn CaseAssess>>,
) -> Result<(), Error> {
    if ca_vec.len() == 0 {
        return Ok(());
    }

    for sv in ca_vec.iter().map(|ca| to_value_vec(ca.as_ref())) {
        for v in sv {
            writer.write_record(&v)?
        }
    }
    writer.flush()?;
    return Ok(());
}

fn to_value_vec(ca: &dyn CaseAssess) -> Vec<Vec<String>> {
    let mut value_vec = Vec::new();
    let empty = &vec![];
    let pa_vec = match ca.state() {
        CaseState::Ok(pa_vec) => pa_vec,
        CaseState::Fail(pa_vec) => pa_vec,
        _ => empty,
    };

    if !pa_vec.is_empty() {
        for pa in pa_vec.iter() {
            let mut step_value = Vec::with_capacity(6);
            step_value.push(pa.id().to_string());
            step_value.push("step".to_string());
            match pa.state() {
                StepState::Ok(v) => {
                    step_value.push(String::from("O"));
                    step_value.push(to_csv_string(v.as_value()));
                    step_value.push(to_csv_string(pa.explain()));
                }
                StepState::Err(e) => {
                    step_value.push(String::from("E"));
                    step_value.push(String::from(format!("{}", e)));
                    step_value.push(to_csv_string(pa.explain()));
                }
                StepState::Fail(v) => {
                    step_value.push(String::from("F"));
                    step_value.push(to_csv_string(v.as_value()));
                    step_value.push(to_csv_string(pa.explain()));
                }
            }
            step_value.push(pa.start().format("%T").to_string());
            step_value.push(pa.end().format("%T").to_string());
            value_vec.push(step_value);
        }
    }

    let mut case_value = Vec::with_capacity(6);
    case_value.push(ca.id().to_string());
    case_value.push("case".to_string());
    match ca.state() {
        CaseState::Ok(_) => {
            case_value.push(String::from("O"));
            case_value.push(String::from(""));
            case_value.push(to_csv_string(ca.data()));
        }
        CaseState::Err(e) => {
            case_value.push(String::from("E"));
            case_value.push(String::from(format!("{}", e)));
            case_value.push(to_csv_string(ca.data()));
        }
        CaseState::Fail(_) => {
            case_value.push(String::from("F"));
            case_value.push(String::from(""));
            case_value.push(to_csv_string(ca.data()));
        }
    }
    case_value.push(ca.start().format("%T").to_string());
    case_value.push(ca.end().format("%T").to_string());

    value_vec.push(case_value);
    value_vec
}

fn to_csv_string(explain: &Value) -> String {
    if explain.is_string() {
        return explain.as_str().unwrap().to_string();
    } else {
        to_string_pretty(&explain).unwrap_or("".to_string())
    }
}
