use async_std::fs::read_dir;
use async_std::path::Path;
use async_std::sync::Arc;
use async_std::task::Builder;
use futures::future::join_all;
use futures::StreamExt;
use log::info;
use log::trace;

use chord::flow::{Flow, ID_PATTERN};
use chord::output::{DateTime, Report, Utc};
use chord::task::{TaskAssess, TaskId, TaskState};
use chord::value::Value;
use chord::Error;
use chord_flow::{Context, TaskIdSimple};
use chord_output::report::{Factory, ReportFactory};

pub async fn run<P: AsRef<Path>>(
    job_path: P,
    job_name: String,
    exec_id: String,
    app_ctx: Arc<dyn Context>,
    report: Option<&Value>,
) -> Result<Vec<TaskState>, Error> {
    trace!(
        "job start {}, {}",
        job_path.as_ref().to_str().unwrap(),
        job_name.as_str()
    );

    let mut job_dir = read_dir(job_path.as_ref()).await?;
    let report_factory = ReportFactory::new(report).await?;
    let report_factory = Arc::new(report_factory);

    let mut futures = Vec::new();
    loop {
        let task_dir = job_dir.next().await;
        if task_dir.is_none() {
            break;
        }
        let task_dir = task_dir.unwrap();
        if task_dir.is_err() {
            continue;
        }
        let task_dir = task_dir.unwrap();
        if !task_dir.path().is_dir().await {
            continue;
        }

        let task_name: String = task_dir.file_name().to_str().unwrap().into();
        if !ID_PATTERN.is_match(task_name.as_str()) {
            continue;
        }

        let builder = Builder::new().name(task_name);

        let task_input_dir = job_path.as_ref().join(task_dir.path());
        let jh = builder.spawn(task_run(
            task_input_dir,
            exec_id.clone(),
            app_ctx.clone(),
            report_factory.clone(),
        ))?;
        futures.push(jh);
    }

    let task_state_vec = join_all(futures).await;
    trace!(
        "job end {}, {}",
        job_path.as_ref().to_str().unwrap(),
        job_name.as_str()
    );
    return Ok(task_state_vec);
}

async fn task_run<P: AsRef<Path>>(
    input_dir: P,
    exec_id: String,
    app_ctx: Arc<dyn Context>,
    report_factory: Arc<ReportFactory>,
) -> TaskState {
    let task_path = Path::new(input_dir.as_ref());
    trace!("task start {}", task_path.to_str().unwrap());
    let task_state = task_run0(task_path, exec_id, app_ctx, report_factory).await;
    return if let Err(e) = task_state {
        info!("task error {}, {}", task_path.to_str().unwrap(), e);
        TaskState::Err(e.clone())
    } else {
        task_state.unwrap()
    };
}

async fn task_run0<P: AsRef<Path>>(
    task_path: P,
    exec_id: String,
    app_ctx: Arc<dyn Context>,
    report_factory: Arc<ReportFactory>,
) -> Result<TaskState, Error> {
    let task_path = Path::new(task_path.as_ref());

    let task_id = task_path.file_name().unwrap().to_str().unwrap();
    let task_id = Arc::new(TaskIdSimple::new(exec_id, task_id.to_owned())?);

    chord_flow::CTX_ID.with(|tid| tid.replace(task_id.to_string()));

    //reporter
    let assess_reporter = report_factory.create(task_id.clone()).await?;

    let rt = task_run1(task_path, task_id.clone(), app_ctx.clone(), assess_reporter).await;
    return if let Err(e) = rt {
        info!("task error {}, {}", task_path.to_str().unwrap(), e);
        task_end(assess_reporter, task_id.clone(), TaskState::Err(e.clone())).await?;
        Ok(TaskState::Err(e))
    } else {
        trace!("task end {}", task_path.to_str().unwrap());
        Ok(rt.unwrap())
    };
}

async fn task_run1<P: AsRef<Path>>(
    task_path: P,
    task_id: Arc<TaskIdSimple>,
    app_ctx: Arc<dyn Context>,
    assess_reporter: Box<dyn Report>,
) -> Result<TaskState, Error> {
    let task_path = Path::new(task_path.as_ref());
    let flow_path = task_path.clone().join("flow.yml");

    let flow = chord_input::load::flow::yml::load(&flow_path)?;
    let flow = Flow::new(flow)?;

    //read
    let data_file_path = task_path.clone().join("case.csv");
    let data_loader = chord_input::load::data::csv::Loader::new(data_file_path).await?;

    //runner
    let mut runner = chord_flow::TaskRunner::new(
        Box::new(data_loader),
        assess_reporter,
        app_ctx,
        Arc::new(flow),
        task_id.clone(),
    )
    .await?;

    let task_assess = runner.run().await?;
    return match task_assess.state() {
        TaskState::Ok => Ok(TaskState::Ok),
        TaskState::Fail => Ok(TaskState::Fail),
        TaskState::Err(e) => Ok(TaskState::Err(e.clone())),
    };
}

async fn task_end(
    assess_reporter: &mut dyn Report,
    task_id: Arc<TaskIdSimple>,
    state: TaskState,
) -> Result<(), Error> {
    let now = Utc::now();
    let assess_struct = TaskAssessStruct {
        id: task_id,
        start: now,
        end: now,
        state,
    };
    reporter.end(&assess_struct).await
}

struct TaskAssessStruct {
    id: Arc<TaskIdSimple>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    state: TaskState,
}

impl TaskAssess for TaskAssessStruct {
    fn id(&self) -> &dyn TaskId {
        self.id.as_ref()
    }

    fn start(&self) -> DateTime<Utc> {
        self.start
    }

    fn end(&self) -> DateTime<Utc> {
        self.end
    }

    fn state(&self) -> &TaskState {
        &self.state
    }
}
