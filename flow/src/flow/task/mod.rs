use std::error::Error as StdError;
use std::sync::Arc;

use futures::future::join_all;
use handlebars::RenderError;
use log::{error, info, trace, warn};
use tracing::{error_span, Instrument};

use chord_core::case::{CaseAsset, CaseId, CaseState};
use chord_core::collection::TailDropVec;
use chord_core::flow::Flow;
use chord_core::future::time::timeout;
use chord_core::input::{StageLoader, TaskLoader};
use chord_core::output::{StageReporter, TaskReporter};
use chord_core::output::Utc;
use chord_core::step::{StepAsset, StepState};
use chord_core::task::{StageAsset, StageId, StageState, TaskAsset, TaskId, TaskState};
use chord_core::value::{json, Map, Value};
use res::TaskAssetStruct;

use crate::CTX_ID;
use crate::flow::assign_by_render;
use crate::flow::case;
use crate::flow::case::arg::{CaseArgStruct, CaseIdStruct};
use crate::flow::step::{action_asset_to_value, StepRunner};
use crate::flow::step::arg::{ArgStruct, ChordStruct};
use crate::flow::task::arg::{StageIdStruct, TaskIdStruct};
use crate::flow::task::Error::*;
use crate::flow::task::res::StageAssetStruct;
use crate::model::app::{App, RenderContext};

pub mod arg;
pub mod res;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("`{0}` render:\n{1}")]
    Render(String, RenderError),

    #[error("pre:")]
    PreErr,

    #[error("pre step `{0}`")]
    PreFail(String),

    #[error("{0} `{1}` reporter error:\n{2}")]
    Reporter(String, String, Box<dyn StdError + Sync + Send>),

    #[error("{0} `{1}` loader error:\n{2}")]
    Loader(String, String, Box<dyn StdError + Sync + Send>),

    #[error("stage `{0}` case is empty")]
    CaseEmpty(String),

    #[error("step `{0}` create:\n{1}")]
    Step(String, Box<dyn StdError + Sync + Send>),

    #[error("{0}")]
    Unknown(String),
}

#[derive()]
pub struct TaskRunner {
    step_vec: Arc<TailDropVec<(String, StepRunner)>>,
    stage_round_no: usize,
    stage_id: Arc<String>,
    stage_state: StageState,

    pre_ctx: Option<Arc<Map>>,
    #[allow(dead_code)]
    pre_asset: Option<Box<dyn CaseAsset>>,
    #[allow(dead_code)]
    pre_step_vec: Option<Arc<TailDropVec<(String, StepRunner)>>>,

    task_state: TaskState,

    def_ctx: Option<Arc<Map>>,
    reporter: Box<dyn TaskReporter>,
    loader: Box<dyn TaskLoader>,
    chord: Arc<ChordStruct>,
    id: Arc<TaskIdStruct>,
    flow: Arc<Flow>,
    app: Arc<dyn App>,
}

impl TaskRunner {
    pub fn new(
        loader: Box<dyn TaskLoader>,
        reporter: Box<dyn TaskReporter>,
        app: Arc<dyn App>,
        flow: Arc<Flow>,
        id: Arc<TaskIdStruct>,
    ) -> TaskRunner {
        let runner = TaskRunner {
            step_vec: Arc::new(TailDropVec::from(vec![])),
            stage_id: Arc::new("0".into()),
            stage_round_no: 0,

            stage_state: StageState::Ok,

            pre_ctx: None,
            pre_asset: None,
            pre_step_vec: None,

            task_state: TaskState::Ok,

            def_ctx: None,
            reporter,
            loader,
            chord: Arc::new(ChordStruct::new(app.clone())),
            id,
            flow,
            app,
        };

        runner
    }

    pub fn id(&self) -> Arc<dyn TaskId> {
        self.id.clone()
    }

    pub async fn run(mut self) -> Box<dyn TaskAsset> {
        trace!("task run");
        let start = Utc::now();

        if let Err(e) = self.reporter.start(start).await {
            error!("task Err");
            return Box::new(TaskAssetStruct::new(
                self.id.clone(),
                start,
                Utc::now(),
                TaskState::Err(Box::new(Reporter(
                    "task".to_string(),
                    self.id.task().to_string(),
                    e,
                ))),
            ));
        }

        if let Some(def_raw) = self.flow.def() {
            let rc: Value = json!({
               "__meta__": self.flow.meta()
            });
            let rc = RenderContext::wraps(rc).unwrap();
            let rso = assign_by_render(self.app.get_handlebars(), &rc, def_raw, false);
            if let Err(e) = rso {
                error!("task Err");
                return Box::new(TaskAssetStruct::new(
                    self.id.clone(),
                    start,
                    Utc::now(),
                    TaskState::Err(Box::new(Render("def".to_string(), e))),
                ));
            } else {
                self.def_ctx = Some(Arc::new(rso.unwrap()));
            }
        }

        if let Some(pre_step_id_vec) = self.flow.pre_step_id_vec() {
            if !pre_step_id_vec.is_empty() {
                let pre_step_vec = step_vec_create(
                    self.app.as_ref(),
                    self.flow.as_ref(),
                    pre_step_id_vec.into_iter().map(|s| s.to_owned()).collect(),
                    Arc::new(StageIdStruct::new(
                        self.id.clone(),
                        "init".to_string(),
                        0.to_string(),
                    )),
                    self.chord.clone(),
                )
                    .await;
                if let Err(e) = pre_step_vec {
                    error!("task Err");
                    return Box::new(TaskAssetStruct::new(
                        self.id.clone(),
                        start,
                        Utc::now(),
                        TaskState::Err(Box::new(e)),
                    ));
                }

                let pre_step_vec = Arc::new(TailDropVec::from(pre_step_vec.unwrap()));
                let pre_arg = pre_arg(
                    self.flow.clone(),
                    self.id.clone(),
                    self.def_ctx.clone(),
                    pre_step_vec.clone(),
                )
                    .await;
                if let Err(e) = pre_arg {
                    error!("task Err");
                    return Box::new(TaskAssetStruct::new(
                        self.id,
                        start,
                        Utc::now(),
                        TaskState::Err(Box::new(e)),
                    ));
                }

                let pre_asset = case_run(self.app.as_ref(), pre_arg.unwrap()).await;

                match pre_asset.state() {
                    CaseState::Err(_) => {
                        error!("task Err");
                        return Box::new(TaskAssetStruct::new(
                            self.id,
                            start,
                            Utc::now(),
                            TaskState::Err(Box::new(PreErr)),
                        ));
                    }

                    CaseState::Fail(v) => {
                        error!("task Err");
                        return Box::new(TaskAssetStruct::new(
                            self.id.clone(),
                            start,
                            Utc::now(),
                            TaskState::Err(Box::new(PreFail(
                                v.last().unwrap().id().step().to_string(),
                            ))),
                        ));
                    }
                    CaseState::Ok(sa_vec) => {
                        let pre_ctx = pre_ctx_create(sa_vec.as_ref()).await;
                        self.pre_ctx = Some(Arc::new(pre_ctx));
                        self.pre_asset = Some(pre_asset);
                        self.pre_step_vec = Some(pre_step_vec);
                    }
                }
            }
        };

        let result = self.task_run().await;

        let task_asset = if let Err(e) = result {
            error!("task Err");
            TaskAssetStruct::new(
                self.id.clone(),
                start,
                Utc::now(),
                TaskState::Err(Box::new(e)),
            )
        } else {
            match self.task_state {
                TaskState::Ok => {
                    info!("task Ok");
                    TaskAssetStruct::new(self.id.clone(), start, Utc::now(), TaskState::Ok)
                }
                TaskState::Fail(c) => {
                    warn!("task Fail");
                    TaskAssetStruct::new(
                        self.id.clone(),
                        start,
                        Utc::now(),
                        TaskState::Fail(c.clone()),
                    )
                }
                TaskState::Err(e) => {
                    error!("task Err");
                    TaskAssetStruct::new(self.id.clone(), start, Utc::now(), TaskState::Err(e))
                }
            }
        };

        if let Err(e) = self.reporter.end(&task_asset).await {
            error!("task Err");
            return Box::new(TaskAssetStruct::new(
                self.id.clone(),
                start,
                Utc::now(),
                TaskState::Err(Box::new(Reporter(
                    "task".to_string(),
                    self.id.task().to_string(),
                    e,
                ))),
            ));
        }

        Box::new(task_asset)
    }

    async fn task_run(&mut self) -> Result<(), Error> {
        let stage_id_vec: Vec<String> = self
            .flow
            .stage_id_vec()
            .into_iter()
            .map(|s| s.to_owned())
            .collect();
        for state_id in stage_id_vec {
            trace!("stage run");
            self.stage_run(state_id.as_str())
                .await?;
            if let StageState::Fail(_) = self.stage_state {
                if "stage_fail" == self.flow.stage_break_on(state_id.as_str()) {
                    break;
                }
            }
        }
        Ok(())
    }

    async fn stage_run(&mut self, stage_id: &str) -> Result<(), Error> {
        self.stage_id = Arc::new(stage_id.to_string());
        self.stage_state = StageState::Ok;
        let step_id_vec: Vec<String> = self
            .flow
            .stage_step_id_vec(stage_id)
            .into_iter()
            .map(|s| s.to_owned())
            .collect();
        let action_vec = step_vec_create(
            self.app.as_ref(),
            self.flow.as_ref(),
            step_id_vec,
            Arc::new(StageIdStruct::new(
                self.id.clone(),
                stage_id.to_string(),
                "0".to_string())),
            self.chord.clone(),
        )
            .await?;
        self.step_vec = Arc::new(TailDropVec::from(action_vec));

        let duration = self.flow.stage_duration(stage_id);

        let srr = self.stage_run_round(stage_id);
        match timeout(duration, srr).await {
            Ok(r) => r?,
            Err(_) => (),
        }
        return Ok(());
    }

    async fn stage_run_round(&mut self, stage_id: &str) -> Result<(), Error> {
        let concurrency = self.flow.stage_concurrency(stage_id);
        let round_max = self.flow.stage_round(stage_id);
        let mut round_count = 1;
        loop {
            self.stage_round_no = round_count;
            let stage = Arc::new(StageIdStruct::new(
                self.id.clone(),
                stage_id.to_string(),
                round_count.to_string(),
            ));
            let id = format!("{}-{}", stage_id, round_count);
            self.stage_run_once(stage, concurrency)
                .instrument(error_span!("stage", id))
                .await?;
            if round_count >= round_max {
                break;
            }
            round_count += 1;
        }
        return Ok(());
    }

    async fn stage_run_once(&mut self, stage: Arc<StageIdStruct>, concurrency: usize) -> Result<(), Error> {
        trace!("stage run");
        let start = Utc::now();

        let mut loader = self
            .loader
            .stage(stage.stage())
            .await
            .map_err(|e| Loader("stage".to_string(), stage.to_string(), e))?;

        let mut reporter = self
            .reporter
            .stage(stage.stage())
            .await
            .map_err(|e| Reporter("stage".to_string(), stage.to_string(), e))?;

        reporter
            .start(Utc::now())
            .await
            .map_err(|e| Reporter("stage".to_string(), stage.to_string(), e))?;

        let result = self
            .stage_run_io(stage.clone(), loader.as_mut(), reporter.as_mut(), concurrency)
            .await;

        let stage_asset = if let Err(e) = result {
            error!("stage Err, {:?}", e);
            StageAssetStruct::new(
                stage.clone(),
                start,
                Utc::now(),
                StageState::Err(Box::new(e)),
            )
        } else {
            match &self.stage_state {
                StageState::Ok => {
                    info!("stage Ok");
                    StageAssetStruct::new(stage.clone(), start, Utc::now(), StageState::Ok)
                }
                StageState::Fail(c) => {
                    warn!("stage Fail");
                    StageAssetStruct::new(
                        stage.clone(),
                        start,
                        Utc::now(),
                        StageState::Fail(c.clone()),
                    )
                }
                StageState::Err(_) => unreachable!(),
            }
        };

        reporter
            .end(&stage_asset)
            .await
            .map_err(|e| Reporter("stage".to_string(), stage.to_string(), e))?;

        match stage_asset.state() {
            StageState::Ok => Ok(()),
            StageState::Fail(_) => Ok(()),
            StageState::Err(e) => Err(Unknown(e.to_string())),
        }
    }

    async fn stage_run_io(
        &mut self,
        stage: Arc<StageIdStruct>,
        loader: &mut dyn StageLoader,
        reporter: &mut dyn StageReporter,
        concurrency: usize,
    ) -> Result<(), Error> {
        let mut load_times = 0;
        loop {
            let case_data_vec: Vec<(String, Value)> = loader
                .load(concurrency)
                .await
                .map_err(|e| Loader("stage".to_string(), stage.to_string(), e))?;
            load_times = load_times + 1;
            if case_data_vec.len() == 0 {
                return if load_times == 1 {
                    Err(CaseEmpty(stage.to_string()))
                } else {
                    trace!("stage exhaust data");
                    Ok(())
                };
            }

            trace!(
                "stage load data, {}",
                case_data_vec.len()
            );

            let case_asset_vec = self.case_data_vec_run(stage.clone(), case_data_vec, concurrency).await;
            let first_fail = case_asset_vec.iter().find(|ca| !ca.state().is_ok());
            if first_fail.is_some() {
                let cause_case = first_fail.unwrap();
                let cause = match cause_case.state() {
                    CaseState::Err(_) => format!("case: {}", cause_case.id()),
                    CaseState::Fail(v) => {
                        let last_step_id = v
                            .last()
                            .map(|s| s.id().to_string())
                            .or_else(|| Some(String::new()))
                            .unwrap();
                        format!("step: {}", last_step_id)
                    }
                    CaseState::Ok(_) => String::new(),
                };
                self.stage_state = StageState::Fail(cause.clone());
                self.task_state = TaskState::Fail(cause.clone());
            }
            reporter
                .report(&case_asset_vec)
                .await
                .map_err(|e| Reporter("stage".to_string(), stage.to_string(), e))?;
        }
    }

    async fn case_data_vec_run(
        &mut self,
        stage: Arc<StageIdStruct>,
        case_vec: Vec<(String, Value)>,
        concurrency: usize,
    ) -> Vec<Box<dyn CaseAsset>> {
        let ca_vec = self.case_arg_vec(stage, case_vec);

        let mut case_asset_vec = Vec::<Box<dyn CaseAsset>>::new();
        let mut futures = vec![];
        for ca in ca_vec {
            let f = case_run_arc(self.app.clone(), ca);
            futures.push(f);
            if futures.len() >= concurrency {
                let case_asset = join_all(futures.split_off(0)).await;
                case_asset_vec.extend(case_asset);
            }
        }
        if !futures.is_empty() {
            let case_asset = join_all(futures).await;
            case_asset_vec.extend(case_asset);
        }

        case_asset_vec
    }

    fn case_arg_vec<'p>(&self, stage: Arc<StageIdStruct>, data: Vec<(String, Value)>) -> Vec<CaseArgStruct> {
        let vec = data
            .into_iter()
            .map(|(id, d)| {
                CaseArgStruct::new(
                    self.flow.clone(),
                    self.step_vec.clone(),
                    d,
                    self.pre_ctx.clone(),
                    self.def_ctx.clone(),
                    stage.clone(),
                    id,
                )
            })
            .collect();
        return vec;
    }
}

async fn pre_arg(
    flow: Arc<Flow>,
    task_id: Arc<TaskIdStruct>,
    def_ctx: Option<Arc<Map>>,
    pre_step_vec: Arc<TailDropVec<(String, StepRunner)>>,
) -> Result<CaseArgStruct, Error> {
    let stage = Arc::new(StageIdStruct::new(
        task_id.clone(),
        "init".to_string(),
        0.to_string(),
    ));
    Ok(CaseArgStruct::new(
        flow.clone(),
        pre_step_vec,
        Value::Null,
        None,
        def_ctx,
        stage,
        "pre".into(),
    ))
}

async fn pre_ctx_create(sa_vec: &Vec<Box<dyn StepAsset>>) -> Map {
    let mut pre_ctx = Map::new();
    pre_ctx.insert("step".to_owned(), Value::Object(Map::new()));
    for sa in sa_vec.iter() {
        if let StepState::Ok(av) = sa.state() {
            let mut am = Map::new();
            for a in av.iter() {
                am.insert(a.id().to_string(), action_asset_to_value(a.as_ref()));
            }
            pre_ctx["step"][sa.id().step()] = Value::Object(am);
        }
    }
    pre_ctx
}


async fn step_vec_create(
    app: &dyn App,
    flow: &Flow,
    step_id_vec: Vec<String>,
    stage: Arc<StageIdStruct>,
    chord: Arc<ChordStruct>,
) -> Result<Vec<(String, StepRunner)>, Error> {
    let mut step_vec = vec![];
    let fake_case_id = Arc::new(CaseIdStruct::new(
        stage,
        "0".to_string(),
    ));
    for sid in step_id_vec {
        let mut arg = ArgStruct::new(
            app,
            flow,
            RenderContext::wraps(Value::Object(Map::with_capacity(0)))
                .map_err(|e| Step(sid.clone(), Box::new(e)))?,
            fake_case_id.clone(),
            sid.clone(),
        );

        let pr = StepRunner::new(chord.clone(), &mut arg)
            .await
            .map_err(|e| Step(sid.clone(), Box::new(e)))?;
        step_vec.push((sid, pr));
    }
    Ok(step_vec)
}

async fn case_run(flow_ctx: &dyn App, case_arg: CaseArgStruct) -> Box<dyn CaseAsset> {
    let cas = case::run(flow_ctx, case_arg)
        .await;
    Box::new(cas)
}


async fn case_run_arc(flow_ctx: Arc<dyn App>, case_arg: CaseArgStruct) -> Box<dyn CaseAsset> {
    let id = format!("{}", case_arg.id().case());
    CTX_ID
        .scope(
            case_arg.id().to_string(),
            case_run(flow_ctx.as_ref(), case_arg),
        )
        .instrument(error_span!("case", id))
        .await
}
