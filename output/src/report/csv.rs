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
use log::info;

use chord_core::case::{CaseAssess, CaseState};
use chord_core::flow::Flow;
use chord_core::output::Error;
use chord_core::output::JobReporter;
use chord_core::output::StageReporter;
use chord_core::output::{async_trait, TaskReporter};
use chord_core::step::StepState;
use chord_core::task::{StageAssess, TaskAssess, TaskId, TaskState};
use chord_core::value::{to_string_pretty, Value};

pub struct CsvJobReporter {
    dir: PathBuf,
    with_bom: bool,
}

#[async_trait]
impl JobReporter for CsvJobReporter {
    async fn task(
        &self,
        task_id: Arc<dyn TaskId>,
        flow: Arc<Flow>,
    ) -> Result<Box<dyn TaskReporter>, Error> {
        let reporter = CsvTaskReporter::new(self.dir.clone(), task_id, flow, self.with_bom).await?;
        Ok(Box::new(reporter))
    }
}

impl CsvJobReporter {
    pub async fn new<P: AsRef<Path>>(
        report_dir: P,
        name: String,
        exec_id: String,
        with_bom: bool,
    ) -> Result<CsvJobReporter, Error> {
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

        Ok(CsvJobReporter {
            dir: report_dir.clone(),
            with_bom,
        })
    }
}

pub struct CsvTaskReporter {
    dir: PathBuf,
    task_id: Arc<dyn TaskId>,
    with_bom: bool,
    flow: Arc<Flow>,
}

impl CsvTaskReporter {
    pub async fn new<P: AsRef<Path>>(
        dir: P,
        task_id: Arc<dyn TaskId>,
        flow: Arc<Flow>,
        with_bom: bool,
    ) -> Result<CsvTaskReporter, Error> {
        let dir = PathBuf::from(dir.as_ref());
        let task_state_file = dir.join(format!("R.{}.csv", task_id.task()));
        from_path(task_state_file, with_bom, false).await?;
        let report = CsvTaskReporter {
            dir,
            task_id,
            with_bom,
            flow,
        };
        Ok(report)
    }
}

#[async_trait]
impl TaskReporter for CsvTaskReporter {
    async fn stage(&self, stage_id: &str) -> Result<Box<dyn StageReporter>, Error> {
        let reporter = CsvStageReporter::new(
            self.dir.clone(),
            self.task_id.clone(),
            stage_id,
            self.flow.clone(),
            self.with_bom,
        )
        .await?;
        Ok(Box::new(reporter))
    }

    async fn start(&mut self, _: DateTime<Utc>) -> Result<(), Error> {
        Ok(())
    }

    async fn end(&mut self, task_assess: &dyn TaskAssess) -> Result<(), Error> {
        let task_state_view = match task_assess.state() {
            TaskState::Ok => "O",
            TaskState::Err(_) => "E",
            TaskState::Fail(_) => "F",
        };

        let report_file = self.dir.join(format!("R.{}.csv", self.task_id.task()));
        let report_file_new =
            self.dir
                .join(format!("{}.{}.csv", task_state_view, self.task_id.task()));
        rename(report_file, report_file_new).await?;
        Ok(())
    }
}

pub struct CsvStageReporter {
    writer: Writer<std::fs::File>,
    head: Vec<String>,
}

impl CsvStageReporter {
    pub async fn new<P: AsRef<Path>>(
        dir: P,
        task_id: Arc<dyn TaskId>,
        stage_id: &str,
        flow: Arc<Flow>,
        with_bom: bool,
    ) -> Result<CsvStageReporter, Error> {
        let fixed_step = {
            let mut v = true;
            for step_id in flow.stage_step_id_vec(stage_id) {
                if let Some(then) = flow.step_then(step_id) {
                    for th in then {
                        if th.goto().is_some() {
                            info!("step goto detected {}, result will be flexible", step_id);
                            v = false;
                            break;
                        }
                    }
                }
            }
            v
        };

        let dir = PathBuf::from(dir.as_ref());
        let report_file = dir.join(format!("{}.{}.csv", task_id.task(), stage_id));
        let mut writer: Writer<std::fs::File> =
            from_path(report_file, with_bom, !fixed_step).await?;

        let head = if fixed_step {
            create_head(flow.stage_step_id_vec(stage_id))
        } else {
            create_head(vec![])
        };
        writer.write_record(&head)?;

        let report = CsvStageReporter { writer, head };
        Ok(report)
    }
}

#[async_trait]
impl StageReporter for CsvStageReporter {
    async fn start(&mut self, _: DateTime<Utc>) -> Result<(), Error> {
        Ok(())
    }

    async fn report(&mut self, ca_vec: &Vec<Box<dyn CaseAssess>>) -> Result<(), Error> {
        if ca_vec.is_empty() {
            return Ok(());
        }
        return Ok(report(&mut self.writer, ca_vec, &self.head).await?);
    }

    async fn end(&mut self, _: &dyn StageAssess) -> Result<(), Error> {
        self.writer.flush()?;
        Ok(())
    }
}

async fn from_path<P: AsRef<Path>>(
    path: P,
    with_bom: bool,
    flexible: bool,
) -> Result<Writer<std::fs::File>, Error> {
    let mut file = std::fs::File::create(path.as_ref().to_str().unwrap())?;
    if with_bom {
        file.write_all("\u{feff}".as_bytes())?;
    }
    Ok(csv::WriterBuilder::new()
        .flexible(flexible)
        .from_writer(file))
}

fn create_head(step_id_vec: Vec<&str>) -> Vec<String> {
    let mut vec: Vec<String> = vec![];
    vec.push(String::from("id"));
    vec.push(String::from("case_state"));
    vec.push(String::from("case_value"));
    vec.push(String::from("case_data"));
    vec.push(String::from("case_start"));
    vec.push(String::from("case_end"));

    for step_id in step_id_vec {
        vec.push(format!("{}_state", step_id));
        vec.push(format!("{}_value", step_id));
        vec.push(format!("{}_explain", step_id));
        vec.push(format!("{}_start", step_id));
        vec.push(format!("{}_end", step_id));
    }

    vec
}

async fn report<W: std::io::Write>(
    writer: &mut Writer<W>,
    ca_vec: &Vec<Box<dyn CaseAssess>>,
    header: &Vec<String>,
) -> Result<(), Error> {
    if ca_vec.len() == 0 {
        return Ok(());
    }

    for sv in ca_vec.iter().map(|ca| to_value_vec(ca.as_ref(), header)) {
        writer.write_record(&sv)?
    }
    writer.flush()?;
    return Ok(());
}

fn to_value_vec(ca: &dyn CaseAssess, header: &Vec<String>) -> Vec<String> {
    let mut value_vec: Vec<String> = Vec::new();
    let empty = &vec![];
    let pa_vec = match ca.state() {
        CaseState::Ok(pa_vec) => pa_vec,
        CaseState::Fail(pa_vec) => pa_vec,
        _ => empty,
    };

    let mut case_value = Vec::with_capacity(6);
    case_value.push(ca.id().to_string());
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
    value_vec.append(&mut case_value);

    if !pa_vec.is_empty() {
        for pa in pa_vec.iter() {
            let mut step_value = Vec::with_capacity(6);
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
            value_vec.append(&mut step_value);
        }
    }

    if header.len() > value_vec.len() {
        let vacancy = header.len() - value_vec.len();
        for _ in 0..vacancy {
            value_vec.push(String::new());
        }
    }

    value_vec
}

fn to_csv_string(explain: &Value) -> String {
    if explain.is_string() {
        return explain.as_str().unwrap().to_string();
    } else {
        to_string_pretty(&explain).unwrap_or("".to_string())
    }
}
