use std::fs::File;
use std::path::{Path, PathBuf};

use async_std::fs::rename;
use csv::Writer;

use async_std::sync::Arc;
use chord::case::{CaseAssess, CaseState};
use chord::err;
use chord::flow::Flow;
use chord::output::async_trait;
use chord::output::AssessReport;
use chord::step::StepState;
use chord::task::{TaskAssess, TaskId, TaskState};
use chord::Error;
use chrono::{DateTime, Utc};

pub struct Reporter {
    writer: Writer<File>,
    step_id_vec: Vec<String>,
    report_dir: PathBuf,
    task_id: Arc<dyn TaskId>,
}

#[async_trait]
impl AssessReport for Reporter {
    async fn start(&mut self, _: DateTime<Utc>) -> Result<(), Error> {
        prepare(&mut self.writer, &self.step_id_vec).await?;
        Ok(())
    }

    async fn report(&mut self, _: &str, ca_vec: &Vec<Box<dyn CaseAssess>>) -> Result<(), Error> {
        report(&mut self.writer, ca_vec, &self.step_id_vec).await
    }

    async fn end(&mut self, task_assess: &dyn TaskAssess) -> Result<(), Error> {
        let task_state_view = match task_assess.state() {
            TaskState::Ok => "O",
            TaskState::Err(_) => "E",
            TaskState::Fail => "F",
        };

        let report_file = self
            .report_dir
            .join(format!("{}_result.csv", self.task_id.task_id()));
        let report_file_new = self.report_dir.join(format!(
            "{}_result_{}.csv",
            self.task_id.task_id(),
            task_state_view
        ));
        rename(report_file, report_file_new).await?;
        Ok(())
    }
}

impl Reporter {
    pub async fn new<P: AsRef<Path>>(
        report_dir: P,
        flow: &Flow,
        task_id: Arc<dyn TaskId>,
    ) -> Result<Reporter, Error> {
        let report_dir = PathBuf::from(report_dir.as_ref());
        let report_file = report_dir.join(format!("{}_result.csv", task_id.task_id()));
        let report = Reporter {
            writer: from_path(report_file).await?,
            step_id_vec: flow.case_step_id_vec(),
            report_dir,
            task_id,
        };
        Ok(report)
    }
}

async fn from_path<P: AsRef<Path>>(path: P) -> Result<Writer<File>, Error> {
    csv::WriterBuilder::new()
        .from_path(path)
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
    ca_vec: &Vec<Box<dyn CaseAssess>>,
    sid_vec: &Vec<String>,
) -> Result<(), Error> {
    if ca_vec.len() == 0 {
        return Ok(());
    }

    let head = create_head(sid_vec);
    for sv in ca_vec
        .iter()
        .map(|ca| to_value_vec(ca.as_ref(), head.len()))
    {
        writer.write_record(&sv)?
    }
    writer.flush()?;
    return Ok(());
}

fn to_value_vec(ca: &dyn CaseAssess, head_len: usize) -> Vec<String> {
    let mut vec = vec![];

    match ca.state() {
        CaseState::Ok(_) => {
            vec.push(String::from("O"));
            vec.push(String::from(""));
        }
        CaseState::Err(e) => {
            vec.push(String::from("E"));
            vec.push(String::from(format!("{}", e)));
        }
        CaseState::Fail(_) => {
            vec.push(String::from("F"));
            vec.push(String::from(""));
        }
    }
    vec.push(ca.start().format("%T").to_string());
    vec.push(ca.end().format("%T").to_string());

    let empty = &vec![];
    let pa_vec = match ca.state() {
        CaseState::Ok(pa_vec) => pa_vec,
        CaseState::Fail(pa_vec) => pa_vec,
        _ => empty,
    };

    if !pa_vec.is_empty() {
        let p_vec: Vec<String> = pa_vec
            .iter()
            .flat_map(|pa| match pa.state() {
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
            })
            .collect();
        vec.extend(p_vec);
    }

    if vec.len() < head_len - 1 {
        for _i in 0..head_len - 1 - vec.len() {
            vec.push(String::from(""));
        }
    }

    if pa_vec.is_empty() {
        vec.push(String::from(""));
    } else {
        match pa_vec.last().unwrap().state() {
            StepState::Fail(json) => {
                vec.push(json.to_string());
            }
            StepState::Err(e) => {
                vec.push(e.to_string());
            }
            _ => {
                vec.push(String::from(""));
            }
        }
    }

    vec
}
