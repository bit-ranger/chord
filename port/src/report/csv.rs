use std::fs::File;
use std::path::{Path, PathBuf};

use async_std::fs::rename;
use csv::Writer;

use chord_common::case::{CaseAssess, CaseState};
use chord_common::err;
use chord_common::error::Error;
use chord_common::flow::Flow;
use chord_common::point::PointState;
use chord_common::rerr;
use chord_common::task::{TaskAssess, TaskState};

pub struct Reporter {
    writer: Writer<File>,
    point_id_vec: Vec<String>,
    total_task_state: TaskState,
    report_dir: PathBuf,
    task_id: String,
}

impl Reporter {
    pub async fn new<P: AsRef<Path>, T: Into<String>>(report_dir: P, task_id: T, flow: &Flow) -> Result<Reporter, Error> {
        let report_dir = PathBuf::from(report_dir.as_ref());
        let task_id = task_id.into();
        let report_file = report_dir.join(format!("{}_result.csv", task_id));
        let mut report = Reporter {
            writer: from_path(report_file).await?,
            point_id_vec: flow.case_point_id_vec()?,
            total_task_state: TaskState::Ok(vec![]),
            report_dir,
            task_id,
        };
        prepare(&mut report.writer, &report.point_id_vec).await?;
        Ok(report)
    }

    pub async fn write(&mut self, task_assess: &dyn TaskAssess) -> Result<(), Error> {
        if self.task_id != task_assess.id() {
            return rerr!("400", "task_id mismatch");
        }

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

        report(&mut self.writer, task_assess, &self.point_id_vec).await?;

        Ok(())
    }

    pub async fn close(self) -> Result<(), Error> {
        let task_state_view = match self.total_task_state {
            TaskState::Ok(_) => "O",
            TaskState::Err(_) => "E",
            TaskState::Fail(_) => "F",
        };

        let report_file = self.report_dir.join(format!("{}_result.csv", self.task_id));
        let report_file_new = self.report_dir.join(format!("{}_result_{}.csv", self.task_id, task_state_view));
        rename(report_file, report_file_new).await?;
        Ok(())
    }
}


async fn from_path<P: AsRef<Path>>(path: P) -> Result<Writer<File>, Error> {
    csv::WriterBuilder::new().from_path(path).map_err(|e| err!("csv", e.to_string()))
}

async fn prepare<W: std::io::Write>(writer: &mut Writer<W>, pt_id_vec: &Vec<String>) -> Result<(), Error> {
    writer.write_record(create_head(pt_id_vec)).map_err(|e| err!("csv", e.to_string()))
}

fn create_head(pt_id_vec: &Vec<String>) -> Vec<String> {
    let mut vec: Vec<String> = vec![];
    vec.push(String::from("case_state"));
    vec.push(String::from("case_info"));
    vec.push(String::from("case_start"));
    vec.push(String::from("case_end"));

    let ph_vec: Vec<String> = pt_id_vec.iter()
        .flat_map(|pid| vec![format!("{}_state", pid), format!("{}_start", pid), format!("{}_end", pid)])
        .collect();
    vec.extend(ph_vec);
    vec.push(String::from("last_point_info"));
    vec
}


async fn report<W: std::io::Write>(writer: &mut Writer<W>, task_assess: &dyn TaskAssess, pt_id_vec: &Vec<String>) -> Result<(), Error> {
    let empty = &vec![];
    let ca_vec = match task_assess.state() {
        TaskState::Ok(ca_vec) => ca_vec,
        TaskState::Fail(ca_vec) => ca_vec,
        TaskState::Err(_) => empty
    };

    if ca_vec.len() == 0 {
        return Ok(());
    }

    let head = create_head(pt_id_vec);
    for sv in ca_vec.iter().map(|ca| to_value_vec(ca.as_ref(), head.len())) {
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
        _ => empty
    };

    if !pa_vec.is_empty() {
        let p_vec: Vec<String> = pa_vec.iter()
            .flat_map(|pa| match pa.state() {
                PointState::Ok(_) => {
                    vec![
                        String::from("O"),
                        pa.start().format("%T").to_string(),
                        pa.end().format("%T").to_string(),
                    ]
                }
                PointState::Err(_) => {
                    vec![
                        String::from("E"),
                        pa.start().format("%T").to_string(),
                        pa.end().format("%T").to_string(),
                    ]
                }
                PointState::Fail(_) => {
                    vec![
                        String::from("F"),
                        pa.start().format("%T").to_string(),
                        pa.end().format("%T").to_string()
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
            PointState::Fail(json) => {
                vec.push(json.to_string());
            }
            PointState::Err(e) => {
                vec.push(e.to_string());
            }
            _ => {
                vec.push(String::from(""));
            }
        }
    }

    vec
}



