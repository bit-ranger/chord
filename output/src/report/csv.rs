use async_std::fs::rename;
use async_std::path::{Path, PathBuf};
use async_std::stream::StreamExt;
use async_std::sync::Arc;
use chrono::{DateTime, Utc};
use csv::Writer;

use crate::report::Factory;
use async_std::fs::remove_file;
use async_std::fs::{create_dir_all, read_dir};
use chord::case::{CaseAssess, CaseState};
use chord::err;
use chord::flow::Flow;
use chord::output::async_trait;
use chord::output::Report;
use chord::step::StepState;
use chord::task::{TaskAssess, TaskId, TaskState};
use chord::value::to_string;
use chord::Error;

pub struct ReportFactory {
    dir: PathBuf,
}

#[async_trait]
impl Factory for ReportFactory {
    async fn create(&self, task_id: Arc<dyn TaskId>) -> Result<Box<dyn Report>, Error> {
        let factory = ReportFactory::create(self, task_id).await?;
        Ok(Box::new(factory))
    }
}

impl ReportFactory {
    pub async fn new<P: AsRef<Path>>(report_dir: P, name: String) -> Result<ReportFactory, Error> {
        let dir = report_dir.as_ref().join(name);

        if !dir.exists().await {
            create_dir_all(dir.as_path()).await?;
        }

        Ok(ReportFactory {
            dir: dir.to_path_buf(),
        })
    }

    pub async fn create(&self, task_id: Arc<dyn TaskId>) -> Result<Reporter, Error> {
        let report_dir = self.dir.join(task_id.exec_id());
        if report_dir.exists().await && report_dir.is_dir().await {
            let mut rd = read_dir(report_dir.clone()).await.unwrap();
            loop {
                let de = rd.next().await;
                if de.is_none() {
                    break;
                }
                let de = de.unwrap()?;
                if de.path().is_file().await {
                    remove_file(de.path()).await?;
                }
            }
        }
        Reporter::new(report_dir, task_id).await
    }
}

pub struct Reporter {
    writer: Writer<std::fs::File>,
    step_id_vec: Vec<String>,
    report_dir: PathBuf,
    task_id: Arc<dyn TaskId>,
}

#[async_trait]
impl Report for Reporter {
    async fn start(&mut self, _: DateTime<Utc>, flow: Arc<Flow>) -> Result<(), Error> {
        let step_id_vec: Vec<String> = flow
            .stage_id_vec()
            .iter()
            .flat_map(|s| flow.stage_step_id_vec(s))
            .map(|s| s.to_owned())
            .collect();
        self.step_id_vec = step_id_vec;
        prepare(&mut self.writer, &self.step_id_vec).await?;
        Ok(())
    }

    async fn report(
        &mut self,
        stage_id: &str,
        ca_vec: &Vec<Box<dyn CaseAssess>>,
    ) -> Result<(), Error> {
        report(&mut self.writer, stage_id, ca_vec, &self.step_id_vec).await
    }

    async fn end(&mut self, task_assess: &dyn TaskAssess) -> Result<(), Error> {
        let task_state_view = match task_assess.state() {
            TaskState::Ok => "O",
            TaskState::Err(_) => "E",
            TaskState::Fail => "F",
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
    ) -> Result<Reporter, Error> {
        let report_dir = PathBuf::from(report_dir.as_ref());
        if !report_dir.exists().await {
            create_dir_all(report_dir.as_path()).await?;
        }

        let report_file = report_dir.join(format!("{}_result.csv", task_id.task()));

        let report = Reporter {
            writer: from_path(report_file).await?,
            step_id_vec: vec![],
            report_dir,
            task_id,
        };
        Ok(report)
    }
}

async fn from_path<P: AsRef<Path>>(path: P) -> Result<Writer<std::fs::File>, Error> {
    csv::WriterBuilder::new()
        .from_path(path.as_ref().to_str().ok_or(err!("010", "invalid path"))?)
        .map_err(|e| err!("csv", e.to_string()))
}

async fn prepare<W: std::io::Write>(
    writer: &mut Writer<W>,
    sid_vec: &Vec<String>,
) -> Result<(), Error> {
    writer
        .write_record(create_head(sid_vec))
        .map_err(|e| err!("csv", e.to_string()))
}

fn create_head(sid_vec: &Vec<String>) -> Vec<String> {
    let mut vec: Vec<String> = vec![];
    vec.push(String::from("case_id"));
    vec.push(String::from("case_state"));
    vec.push(String::from("case_info"));
    vec.push(String::from("case_start"));
    vec.push(String::from("case_end"));

    let ph_vec: Vec<String> = sid_vec
        .iter()
        .flat_map(|sid| {
            vec![
                format!("{}_state", sid),
                format!("{}_start", sid),
                format!("{}_end", sid),
            ]
        })
        .collect();
    vec.extend(ph_vec);
    vec.push(String::from("last_step_info"));
    vec
}

async fn report<W: std::io::Write>(
    writer: &mut Writer<W>,
    stage_id: &str,
    ca_vec: &Vec<Box<dyn CaseAssess>>,
    sid_vec: &Vec<String>,
) -> Result<(), Error> {
    if ca_vec.len() == 0 {
        return Ok(());
    }

    for sv in ca_vec
        .iter()
        .map(|ca| to_value_vec(stage_id, ca.as_ref(), sid_vec))
    {
        writer.write_record(&sv)?
    }
    writer.flush()?;
    return Ok(());
}

fn to_value_vec(stage_id: &str, ca: &dyn CaseAssess, sid_vec: &Vec<String>) -> Vec<String> {
    let head_len = 5 + sid_vec.len() * 3 + 1;
    let value_vec: Vec<&str> = vec![""; head_len];
    let mut value_vec: Vec<String> = value_vec.into_iter().map(|v| v.to_owned()).collect();

    value_vec[0] = format!("{}@{}", ca.id().case(), stage_id);
    match ca.state() {
        CaseState::Ok(_) => {
            value_vec[1] = String::from("O");
            value_vec[2] = String::from("");
        }
        CaseState::Err(e) => {
            value_vec[1] = String::from("E");
            value_vec[2] = String::from(format!("{}", e));
        }
        CaseState::Fail(_) => {
            value_vec[1] = String::from("F");
            value_vec[2] = String::from("");
        }
    }
    value_vec[3] = ca.start().format("%T").to_string();
    value_vec[4] = ca.end().format("%T").to_string();

    let empty = &vec![];
    let pa_vec = match ca.state() {
        CaseState::Ok(pa_vec) => pa_vec,
        CaseState::Fail(pa_vec) => pa_vec,
        _ => empty,
    };

    if !pa_vec.is_empty() {
        for pa in pa_vec.iter() {
            let pv: Vec<String> = match pa.state() {
                StepState::Ok(_) => {
                    vec![
                        String::from("O"),
                        pa.start().format("%T").to_string(),
                        pa.end().format("%T").to_string(),
                    ]
                }
                StepState::Err(_) => {
                    vec![
                        String::from("E"),
                        pa.start().format("%T").to_string(),
                        pa.end().format("%T").to_string(),
                    ]
                }
                StepState::Fail(_) => {
                    vec![
                        String::from("F"),
                        pa.start().format("%T").to_string(),
                        pa.end().format("%T").to_string(),
                    ]
                }
            };

            let pai = sid_vec
                .iter()
                .position(|sid| sid == pa.id().step())
                .unwrap();
            let pos = 5 + pai * 3;

            for (pvi, pve) in pv.into_iter().enumerate() {
                value_vec[pos + pvi] = pve;
            }
        }
    }

    if let Some(last) = pa_vec.last() {
        match last.state() {
            StepState::Fail(scope) | StepState::Ok(scope) => {
                let json = scope.as_value();
                if json.is_string() {
                    value_vec[head_len - 1] =
                        json.as_str().map_or(json.to_string(), |j| j.to_owned());
                } else {
                    value_vec[head_len - 1] = to_string(json).unwrap_or_else(|j| j.to_string());
                }
            }
            StepState::Err(e) => {
                value_vec[head_len - 1] = e.to_string();
            }
        }
    }

    value_vec
}
