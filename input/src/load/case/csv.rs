use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use csv::{Reader, ReaderBuilder};

use chord_core::flow::Flow;
use chord_core::input::{async_trait, Error, JobLoader, StageLoader, TaskLoader};
use chord_core::task::TaskId;
use chord_core::value::{Map, Value};
use std::sync::Arc;

pub struct CsvJobLoader {
    path: PathBuf,
}

impl CsvJobLoader {
    pub async fn new<P: AsRef<Path>>(_: Option<&Value>, path: P) -> Result<CsvJobLoader, Error> {
        Ok(CsvJobLoader {
            path: path.as_ref().to_path_buf(),
        })
    }
}

#[async_trait]
impl JobLoader for CsvJobLoader {
    async fn create(
        &self,
        task_id: Arc<dyn TaskId>,
        flow: Arc<Flow>,
    ) -> Result<Box<dyn TaskLoader>, Error> {
        let mut buf = self.path.clone();
        for p in task_id.task().split(".") {
            buf.push(p);
        }

        let loader = CsvTaskLoader::new(task_id.clone(), flow, buf).await?;
        Ok(Box::new(loader))
    }
}

pub struct CsvTaskLoader {
    task_id: Arc<dyn TaskId>,
    flow: Arc<Flow>,
    path: PathBuf,
}

impl CsvTaskLoader {
    async fn new(
        task_id: Arc<dyn TaskId>,
        flow: Arc<Flow>,
        path: PathBuf,
    ) -> Result<CsvTaskLoader, Error> {
        let loader = CsvTaskLoader {
            task_id,
            flow,
            path,
        };
        Ok(loader)
    }
}

#[async_trait]
impl TaskLoader for CsvTaskLoader {
    async fn create(&self, stage_id: &str) -> Result<Box<dyn StageLoader>, Error> {
        let case_name = self.flow.stage_case_name(stage_id);
        let path = self.path.join(format!("{}.csv", case_name));
        let loader = CsvStageLoader::new(path).await?;
        Ok(Box::new(loader))
    }
}

struct CsvStageLoader {
    row_num: usize,
    reader: Reader<File>,
}

impl CsvStageLoader {
    async fn new<P: AsRef<Path>>(path: P) -> Result<CsvStageLoader, Error> {
        let loader = CsvStageLoader {
            row_num: 1,
            reader: from_path(path.as_ref()).await?,
        };
        Ok(loader)
    }
}

#[async_trait]
impl StageLoader for CsvStageLoader {
    async fn load(&mut self, size: usize) -> Result<Vec<(String, Value)>, Error> {
        let dv = load(&mut self.reader, size).await?;
        let mut result: Vec<(String, Value)> = vec![];
        for d in dv {
            self.row_num = self.row_num + 1;
            result.push((self.row_num.to_string(), d));
        }
        Ok(result)
    }
}

async fn load<R: std::io::Read>(reader: &mut Reader<R>, size: usize) -> Result<Vec<Value>, Error> {
    let mut hashmap_vec = Vec::new();
    let mut curr_size = 0;
    for result in reader.deserialize() {
        let result: Map = result?;

        let mut record: Map = Map::new();
        //data fields must all be string
        for (k, v) in result {
            if v.is_string() {
                record.insert(k, v);
            } else {
                record.insert(k, Value::String(v.to_string()));
            }
        }

        hashmap_vec.push(Value::Object(record));

        curr_size += 1;
        if curr_size == size {
            break;
        }
    }
    Ok(hashmap_vec)
}

async fn from_path<P: AsRef<Path>>(path: P) -> Result<Reader<File>, Error> {
    Ok(ReaderBuilder::new().from_path(path)?)
}
