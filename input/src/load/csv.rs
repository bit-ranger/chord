use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use csv::{Reader, ReaderBuilder};
use log::trace;

use chord_core::flow::Flow;
use chord_core::input::{async_trait, Error, JobLoader, StageLoader, TaskLoader};
use chord_core::task::TaskId;
use chord_core::value::{Map, Value};

static LOAD_STRATEGY_DEFAULT: &str = "actual";

pub struct CsvJobLoader {
    path: PathBuf,
    strategy: String,
}

impl CsvJobLoader {
    pub async fn new<P: AsRef<Path>>(conf: Option<&Value>, path: P) -> Result<CsvJobLoader, Error> {
        let ls = conf
            .map(|c| {
                c["csv"]["strategy"]
                    .as_str()
                    .unwrap_or(LOAD_STRATEGY_DEFAULT)
            })
            .unwrap_or(LOAD_STRATEGY_DEFAULT);

        Ok(CsvJobLoader {
            path: path.as_ref().to_path_buf(),
            strategy: ls.to_string(),
        })
    }
}

#[async_trait]
impl JobLoader for CsvJobLoader {
    async fn task(
        &self,
        task_id: Arc<dyn TaskId>,
        flow: Arc<Flow>,
    ) -> Result<Box<dyn TaskLoader>, Error> {
        let mut buf = self.path.clone();
        for p in task_id.task().split(".") {
            buf.push(p);
        }

        let loader = CsvTaskLoader::new(flow, buf, self.strategy.clone()).await?;
        Ok(Box::new(loader))
    }
}

pub struct CsvTaskLoader {
    flow: Arc<Flow>,
    path: PathBuf,
    strategy: String,
}

impl CsvTaskLoader {
    async fn new(flow: Arc<Flow>, path: PathBuf, strategy: String) -> Result<CsvTaskLoader, Error> {
        let loader = CsvTaskLoader {
            flow,
            path,
            strategy,
        };
        Ok(loader)
    }
}

#[async_trait]
impl TaskLoader for CsvTaskLoader {
    async fn stage(&self, stage_id: &str) -> Result<Box<dyn StageLoader>, Error> {
        let path = self.path.join(format!("{}.csv", stage_id));

        let strategy = self.flow.stage_loader(stage_id)["strategy"]
            .as_str()
            .unwrap_or(self.strategy.as_str());
        let loader = CsvStageLoader::new(path, strategy.to_string()).await?;
        Ok(Box::new(loader))
    }
}

struct CsvStageLoader {
    row_num: usize,
    reader: Reader<File>,
    strategy: String,
}

impl CsvStageLoader {
    async fn new<P: AsRef<Path>>(path: P, strategy: String) -> Result<CsvStageLoader, Error> {
        trace!("new CsvStageLoader {}", path.as_ref().to_str().unwrap());
        let loader = CsvStageLoader {
            row_num: 1,
            reader: from_path(path.as_ref()).await?,
            strategy,
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

        if (!result.is_empty()) && result.len() < size {
            match self.strategy.as_str() {
                "fix_size_repeat_last_page" => {
                    let last_page = result.clone();
                    for i in 0..size - result.len() {
                        let offset = i % last_page.len();
                        let fake_row = (
                            (self.row_num + 1 + i).to_string(),
                            last_page[offset].1.clone(),
                        );
                        result.push(fake_row);
                    }
                }
                _ => {}
            }
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
