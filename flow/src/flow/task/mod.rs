use std::error::Error as StdError;
use std::sync::Arc;

use futures::future::join_all;
use handlebars::RenderError;
use log::{error, info, trace, warn};

use chord_core::case::{CaseAsset, CaseState};
use chord_core::collection::TailDropVec;
use chord_core::flow::Flow;
use chord_core::future::task::{JoinError, JoinHandle, spawn};
use chord_core::future::time::timeout;
use chord_core::input::{StageLoader, TaskLoader};
use chord_core::output::{StageReporter, TaskReporter};
use chord_core::output::Utc;
use chord_core::step::{ActionState, StepAsset, StepState};
use chord_core::task::{StageAssess, StageState, TaskAsset, TaskId, TaskState};
use chord_core::value::{json, Map, Value};
use res::TaskAssessStruct;

use crate::CTX_ID;
use crate::flow::assign_by_render;
use crate::flow::case;
use crate::flow::case::arg::{CaseArgStruct, CaseIdStruct};
use crate::flow::step::arg::{ArgStruct, ChordStruct};
use crate::flow::step::StepRunner;
use crate::flow::task::arg::TaskIdSimple;
use crate::flow::task::Error::*;
use crate::flow::task::res::StageAssessStruct;
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
}

#[derive()]
pub struct TaskRunner {
    step_vec: Arc<TailDropVec<(String, StepRunner)>>,
    stage_round_no: usize,
    stage_id: Arc<String>,
    stage_state: StageState,

    pre_ctx: Option<Arc<Map>>,
    #[allow(dead_code)]
    pre_assess: Option<Box<dyn CaseAsset>>,
    #[allow(dead_code)]
    pre_step_vec: Option<Arc<TailDropVec<(String, StepRunner)>>>,

    task_state: TaskState,

    def_ctx: Option<Arc<Map>>,
    reporter: Box<dyn TaskReporter>,
    loader: Box<dyn TaskLoader>,
    chord: Arc<ChordStruct>,
    id: Arc<TaskIdSimple>,
    flow: Arc<Flow>,
    app: Arc<dyn App>,
}

impl TaskRunner {
    pub fn new(
        loader: Box<dyn TaskLoader>,
        reporter: Box<dyn TaskReporter>,
        app: Arc<dyn App>,
        flow: Arc<Flow>,
        id: Arc<TaskIdSimple>,
    ) -> TaskRunner {
        let runner = TaskRunner {
            step_vec: Arc::new(TailDropVec::from(vec![])),
            stage_id: Arc::new("".into()),
            stage_round_no: 0,

            stage_state: StageState::Ok,

            pre_ctx: None,
            pre_assess: None,
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
        trace!("task run  {}", self.id);
        let start = Utc::now();

        if let Err(e) = self.reporter.start(start).await {
            error!("task Err  {}", self.id);
            return Box::new(TaskAssessStruct::new(
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
                error!("task Err  {}", self.id);
                return Box::new(TaskAssessStruct::new(
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
                    self.id.clone(),
                    self.chord.clone(),
                )
                .await;
                if let Err(e) = pre_step_vec {
                    error!("task Err  {}", self.id);
                    return Box::new(TaskAssessStruct::new(
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
                    error!("task Err  {}", self.id);
                    return Box::new(TaskAssessStruct::new(
                        self.id,
                        start,
                        Utc::now(),
                        TaskState::Err(Box::new(e)),
                    ));
                }

                let pre_assess = case_run(self.app.as_ref(), pre_arg.unwrap()).await;

                match pre_assess.state() {
                    CaseState::Err(_) => {
                        error!("task Err  {}", self.id.clone());
                        return Box::new(TaskAssessStruct::new(
                            self.id,
                            start,
                            Utc::now(),
                            TaskState::Err(Box::new(PreErr)),
                        ));
                    }

                    CaseState::Fail(v) => {
                        error!("task Err  {}", self.id);
                        return Box::new(TaskAssessStruct::new(
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
                        self.pre_assess = Some(pre_assess);
                        self.pre_step_vec = Some(pre_step_vec);
                    }
                }
            }
        };

        let result = self.task_run().await;

        let task_assess = if let Err(e) = result {
            error!("task Err  {}", self.id);
            TaskAssessStruct::new(
                self.id.clone(),
                start,
                Utc::now(),
                TaskState::Err(Box::new(e)),
            )
        } else {
            match self.task_state {
                TaskState::Ok => {
                    info!("task Ok   {}", self.id.clone());
                    TaskAssessStruct::new(self.id.clone(), start, Utc::now(), TaskState::Ok)
                }
                TaskState::Fail(c) => {
                    warn!("task Fail {}", self.id);
                    TaskAssessStruct::new(
                        self.id.clone(),
                        start,
                        Utc::now(),
                        TaskState::Fail(c.clone()),
                    )
                }
                TaskState::Err(e) => {
                    error!("task Err  {}", self.id);
                    TaskAssessStruct::new(self.id.clone(), start, Utc::now(), TaskState::Err(e))
                }
            }
        };

        if let Err(e) = self.reporter.end(&task_assess).await {
            error!("task Err  {}", self.id);
            return Box::new(TaskAssessStruct::new(
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

        Box::new(task_assess)
    }

    async fn task_run(&mut self) -> Result<(), Error> {
        let stage_id_vec: Vec<String> = self
            .flow
            .stage_id_vec()
            .into_iter()
            .map(|s| s.to_owned())
            .collect();
        for state_id in stage_id_vec {
            trace!("stage run   {}-{}", self.id, state_id);
            self.stage_run(state_id.as_str()).await?;
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
            self.id.clone(),
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
        let mut round_count = 0;
        loop {
            self.stage_round_no = round_count;
            self.stage_run_once(stage_id, concurrency).await?;
            round_count += 1;
            if round_count >= round_max {
                break;
            }
        }
        return Ok(());
    }

    async fn stage_run_once(&mut self, stage_id: &str, concurrency: usize) -> Result<(), Error> {
        let start = Utc::now();

        let mut loader = self
            .loader
            .stage(stage_id)
            .await
            .map_err(|e| Loader("stage".to_string(), stage_id.to_string(), e))?;

        let mut reporter = self
            .reporter
            .stage(stage_id)
            .await
            .map_err(|e| Reporter("stage".to_string(), stage_id.to_string(), e))?;

        reporter
            .start(Utc::now())
            .await
            .map_err(|e| Reporter("stage".to_string(), stage_id.to_string(), e))?;

        let result = self
            .stage_run_io(stage_id, loader.as_mut(), reporter.as_mut(), concurrency)
            .await;

        let stage_assess = if let Err(e) = result {
            error!("stage Err  {}-{}, {:?}", self.id, stage_id, e);
            StageAssessStruct::new(
                stage_id.to_string(),
                start,
                Utc::now(),
                StageState::Err(Box::new(e)),
            )
        } else {
            match &self.stage_state {
                StageState::Ok => {
                    info!("stage Ok   {}-{}", self.id.clone(), stage_id);
                    StageAssessStruct::new(stage_id.to_string(), start, Utc::now(), StageState::Ok)
                }
                StageState::Fail(c) => {
                    warn!("stage Fail {}-{}", self.id.clone(), stage_id);
                    StageAssessStruct::new(
                        stage_id.to_string(),
                        start,
                        Utc::now(),
                        StageState::Fail(c.clone()),
                    )
                }
                StageState::Err(_) => unreachable!(),
            }
        };

        reporter
            .end(&stage_assess)
            .await
            .map_err(|e| Reporter("stage".to_string(), stage_id.to_string(), e))?;

        match stage_assess.state() {
            StageState::Ok => Ok(()),
            StageState::Fail(_) => Ok(()),
            StageState::Err(_) => unreachable!(),
        }
    }

    async fn stage_run_io(
        &mut self,
        stage_id: &str,
        loader: &mut dyn StageLoader,
        reporter: &mut dyn StageReporter,
        concurrency: usize,
    ) -> Result<(), Error> {
        let mut load_times = 0;
        loop {
            let case_data_vec: Vec<(String, Value)> = loader
                .load(concurrency)
                .await
                .map_err(|e| Loader("stage".to_string(), stage_id.to_string(), e))?;
            load_times = load_times + 1;
            if case_data_vec.len() == 0 {
                return if load_times == 1 {
                    Err(CaseEmpty(stage_id.to_string()))
                } else {
                    trace!("stage exhaust data {}, {}", self.id, stage_id);
                    Ok(())
                };
            }

            trace!(
                "stage load data {}, {}, {}",
                self.id,
                stage_id,
                case_data_vec.len()
            );

            let case_assess_vec = self.case_data_vec_run(case_data_vec, concurrency).await;
            let first_fail = case_assess_vec.iter().find(|ca| !ca.state().is_ok());
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
                .report(&case_assess_vec)
                .await
                .map_err(|e| Reporter("stage".to_string(), stage_id.to_string(), e))?;
        }
    }

    async fn case_data_vec_run(
        &mut self,
        case_vec: Vec<(String, Value)>,
        concurrency: usize,
    ) -> Vec<Box<dyn CaseAsset>> {
        let ca_vec = self.case_arg_vec(case_vec);

        let mut case_join_result_vec = Vec::<Result<Box<dyn CaseAsset>, JoinError>>::new();
        let mut futures = vec![];
        for ca in ca_vec {
            let f = case_spawn(self.app.clone(), ca);
            futures.push(f);
            if futures.len() >= concurrency {
                let case_assess = join_all(futures.split_off(0)).await;
                case_join_result_vec.extend(case_assess);
            }
        }
        if !futures.is_empty() {
            let case_assess = join_all(futures).await;
            case_join_result_vec.extend(case_assess);
        }

        let mut case_assess_vec = Vec::with_capacity(case_join_result_vec.len());
        for res in case_join_result_vec {
            case_assess_vec.push(res.unwrap());
        }
        case_assess_vec
    }

    fn case_arg_vec<'p>(&self, data: Vec<(String, Value)>) -> Vec<CaseArgStruct> {
        let vec = data
            .into_iter()
            .map(|(id, d)| {
                CaseArgStruct::new(
                    self.flow.clone(),
                    self.step_vec.clone(),
                    d,
                    self.pre_ctx.clone(),
                    self.def_ctx.clone(),
                    self.id.clone(),
                    self.stage_id.clone(),
                    Arc::new(self.stage_round_no.to_string()),
                    id,
                )
            })
            .collect();
        return vec;
    }
}

async fn pre_arg(
    flow: Arc<Flow>,
    task_id: Arc<TaskIdSimple>,
    def_ctx: Option<Arc<Map>>,
    pre_step_vec: Arc<TailDropVec<(String, StepRunner)>>,
) -> Result<CaseArgStruct, Error> {
    Ok(CaseArgStruct::new(
        flow.clone(),
        pre_step_vec,
        Value::Null,
        None,
        def_ctx,
        task_id.clone(),
        Arc::new("pre".into()),
        Arc::new("1".into()),
        "1".into(),
    ))
}

async fn pre_ctx_create(sa_vec: &Vec<Box<dyn StepAsset>>) -> Map {
    let mut pre_ctx = Map::new();
    pre_ctx.insert("step".to_owned(), Value::Object(Map::new()));
    for sa in sa_vec.iter() {
        if let StepState::Ok(av) = sa.state() {
            let mut am = vec![];
            for a in av.iter() {
                if let ActionState::Ok(v) = a.state() {
                    am.push(v.to_value());
                } else if let ActionState::Err(e) = a.state() {
                    am.push(Value::String(e.to_string()));
                }
            }
            pre_ctx["step"][sa.id().step()] = Value::Array(am);
        }
    }
    pre_ctx
}

async fn step_vec_create(
    app: &dyn App,
    flow: &Flow,
    step_id_vec: Vec<String>,
    task_id: Arc<TaskIdSimple>,
    chord: Arc<ChordStruct>,
) -> Result<Vec<(String, StepRunner)>, Error> {
    let mut step_vec = vec![];
    let case_id = Arc::new(CaseIdStruct::new(
        task_id,
        Arc::new("create".to_string()),
        Arc::new("0".to_string()),
        "0".to_string(),
    ));
    for sid in step_id_vec {
        let mut arg = ArgStruct::new(
            app,
            flow,
            RenderContext::wraps(Value::Object(Map::with_capacity(0)))
                .map_err(|e| Step(sid.clone(), Box::new(e)))?,
            case_id.clone(),
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
    Box::new(case::run(flow_ctx, case_arg).await)
}

fn case_spawn(flow_ctx: Arc<dyn App>, case_arg: CaseArgStruct) -> JoinHandle<Box<dyn CaseAsset>> {
    spawn(case_run_arc(flow_ctx, case_arg))
}

async fn case_run_arc(flow_ctx: Arc<dyn App>, case_arg: CaseArgStruct) -> Box<dyn CaseAsset> {
    CTX_ID
        .scope(
            case_arg.id().to_string(),
            case_run(flow_ctx.as_ref(), case_arg),
        )
        .await
}
