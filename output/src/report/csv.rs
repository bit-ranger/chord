use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use csv::Writer;

use chord_core::action::Asset;
use chord_core::case::{CaseAsset, CaseState};
use chord_core::flow::Flow;
use chord_core::future::fs::{create_dir_all, DirEntry, metadata, read_dir, remove_file, rename};
use chord_core::future::path::exists;
use chord_core::output::{async_trait, TaskReporter};
use chord_core::output::Error;
use chord_core::output::JobReporter;
use chord_core::output::StageReporter;
use chord_core::step::{ActionState, StepState};
use chord_core::task::{StageAsset, TaskAsset, TaskId, TaskState};
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
        if !exists(report_dir.as_path()).await {
            create_dir_all(report_dir.as_path()).await?;
        } else {
            let mut rd = read_dir(report_dir.clone()).await?;
            loop {
                let rf: Option<DirEntry> = rd.next_entry().await?;
                if rf.is_none() {
                    break;
                }
                let rf = rf.unwrap();
                let meta = metadata(rf.path()).await;
                if meta.is_ok() && meta.unwrap().is_file() {
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

    async fn end(&mut self, task_asset: &dyn TaskAsset) -> Result<(), Error> {
        let task_state_view = match task_asset.state() {
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
        _flow: Arc<Flow>,
        with_bom: bool,
    ) -> Result<CsvStageReporter, Error> {
        let dir = PathBuf::from(dir.as_ref());
        let report_file = dir.join(format!("{}.{}.csv", task_id.task(), stage_id));
        let mut writer: Writer<std::fs::File> =
            from_path(report_file, with_bom, false).await?;

        let head = vec![
            "task", "stage", "case", "step", "action", "frame", "layer", "start", "end", "state", "value", "explain",
        ]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
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

    async fn report(&mut self, ca_vec: &Vec<Box<dyn CaseAsset>>) -> Result<(), Error> {
        if ca_vec.is_empty() {
            return Ok(());
        }
        return Ok(report(&mut self.writer, ca_vec, &self.head).await?);
    }

    async fn end(&mut self, _: &dyn StageAsset) -> Result<(), Error> {
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


async fn report<W: Write>(
    writer: &mut Writer<W>,
    ca_vec: &Vec<Box<dyn CaseAsset>>,
    header: &Vec<String>,
) -> Result<(), Error> {
    if ca_vec.len() == 0 {
        return Ok(());
    }

    for sv in ca_vec.iter().map(|ca| to_value_vec(ca.as_ref(), header)) {
        for v in sv {
            writer.write_record(&v)?
        }
    }
    writer.flush()?;
    return Ok(());
}

fn to_value_vec(ca: &dyn CaseAsset, _header: &Vec<String>) -> Vec<Vec<String>> {
    let mut result_vec: Vec<Vec<String>> = Vec::new();
    match ca.state() {
        CaseState::Ok(sa_vec)
        | CaseState::Fail(sa_vec) => {
            for sa in sa_vec.iter() {
                match sa.state() {
                    StepState::Ok(aa_vec)
                    | StepState::Fail(aa_vec) => {
                        for aa in aa_vec.iter() {
                            match aa.state() {
                                ActionState::Ok(a) => {
                                    match a {
                                        Asset::Value(v) => {
                                            let aar = vec![
                                                sa.id().case_id().task_id().task().to_string(),
                                                sa.id().case_id().stage_id().to_string(),
                                                sa.id().case_id().case().to_string(),
                                                sa.id().step().to_string(),
                                                aa.id().to_string(),
                                                "".to_string(),
                                                "action".to_string(),
                                                aa.start().format("%T").to_string(),
                                                aa.end().format("%T").to_string(),
                                                "O".to_string(),
                                                to_csv_string(v),
                                                to_csv_string(aa.explain()),
                                            ];
                                            result_vec.push(aar);
                                        }
                                        Asset::Data(d) => {
                                            let aar = vec![
                                                sa.id().case_id().task_id().task().to_string(),
                                                sa.id().case_id().stage_id().to_string(),
                                                sa.id().case_id().case().to_string(),
                                                sa.id().step().to_string(),
                                                aa.id().to_string(),
                                                "".to_string(),
                                                "action".to_string(),
                                                aa.start().format("%T").to_string(),
                                                aa.end().format("%T").to_string(),
                                                "O".to_string(),
                                                to_csv_string(&d.to_value()),
                                                to_csv_string(aa.explain()),
                                            ];
                                            result_vec.push(aar);
                                        }
                                        Asset::Frames(fv) => {
                                            for f in fv {
                                                let aar = vec![
                                                    sa.id().case_id().task_id().task().to_string(),
                                                    sa.id().case_id().stage_id().to_string(),
                                                    sa.id().case_id().case().to_string(),
                                                    sa.id().step().to_string(),
                                                    aa.id().to_string(),
                                                    f.id().to_string(),
                                                    "frame".to_string(),
                                                    f.start().format("%T").to_string(),
                                                    f.end().format("%T").to_string(),
                                                    "O".to_string(),
                                                    to_csv_string(&f.to_value()),
                                                    "".to_string(),
                                                ];
                                                result_vec.push(aar);
                                            }

                                            let aar = vec![
                                                sa.id().case_id().task_id().task().to_string(),
                                                sa.id().case_id().stage_id().to_string(),
                                                sa.id().case_id().case().to_string(),
                                                sa.id().step().to_string(),
                                                aa.id().to_string(),
                                                "".to_string(),
                                                "action".to_string(),
                                                aa.start().format("%T").to_string(),
                                                aa.end().format("%T").to_string(),
                                                "O".to_string(),
                                                "".to_string(),
                                                to_csv_string(aa.explain()),
                                            ];
                                            result_vec.push(aar);
                                        }
                                    }
                                }
                                ActionState::Err(e) => {
                                    let aar = vec![
                                        sa.id().case_id().task_id().task().to_string(),
                                        sa.id().case_id().stage_id().to_string(),
                                        sa.id().case_id().case().to_string(),
                                        sa.id().step().to_string(),
                                        aa.id().to_string(),
                                        "".to_string(),
                                        "action".to_string(),
                                        aa.start().format("%T").to_string(),
                                        aa.end().format("%T").to_string(),
                                        "E".to_string(),
                                        e.to_string(),
                                        to_csv_string(aa.explain()),
                                    ];
                                    result_vec.push(aar);
                                }
                            }
                        }
                    }
                };

                let sas = match sa.state() {
                    StepState::Ok(_) => "O".to_string(),
                    StepState::Fail(_) => "F".to_string(),
                };
                let sar = vec![
                    sa.id().case_id().task_id().task().to_string(),
                    sa.id().case_id().stage_id().to_string(),
                    sa.id().case_id().case().to_string(),
                    sa.id().step().to_string(),
                    "".to_string(),
                    "".to_string(),
                    "step".to_string(),
                    sa.start().format("%T").to_string(),
                    sa.end().format("%T").to_string(),
                    sas,
                    "".to_string(),
                    "".to_string(),
                ];
                result_vec.push(sar);
            }
        }

        CaseState::Err(_) => {}
    };
    let cas = match ca.state() {
        CaseState::Ok(_) => "O".to_string(),
        CaseState::Err(_) => "E".to_string(),
        CaseState::Fail(_) => "F".to_string(),
    };

    let car = vec![
        ca.id().task_id().to_string(),
        ca.id().stage_id().to_string(),
        ca.id().case().to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "case".to_string(),
        ca.start().format("%T").to_string(),
        ca.end().format("%T").to_string(),
        cas,
        "".to_string(),
        "".to_string(),
    ];
    result_vec.push(car);

    result_vec
}

fn to_csv_string(explain: &Value) -> String {
    if explain.is_string() {
        return explain.as_str().unwrap().to_string();
    } else {
        to_string_pretty(&explain).unwrap_or("".to_string())
    }
}
