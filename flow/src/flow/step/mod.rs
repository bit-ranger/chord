use std::panic::AssertUnwindSafe;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use futures::FutureExt;
use handlebars::TemplateRenderError;
use itertools::Step;
use log::{debug, error, info, trace, warn};

use chord_core::action::{Action, Factory, RunArg, Scope};
use chord_core::collection::TailDropVec;
use chord_core::flow::Flow;
use chord_core::future::time::timeout;
use chord_core::step::StepState;
use chord_core::value::{to_string_pretty, Value};
use chord_core::value::json;
use Error::*;
use res::StepAssessStruct;

use crate::flow::{assign_by_render, task};
use crate::flow::step::arg::{CreateArgStruct, RunArgStruct};
use crate::flow::step::Error::ValueUnexpected;
use crate::flow::step::res::ActionAssessStruct;
use crate::model::app::{FlowApp, RenderContext};
use crate::TaskIdSimple;

pub mod arg;
pub mod res;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("timeout")]
    Timeout,

    #[error("unwind")]
    Unwind,

    #[error("`{0}` unexpect value `{1}`")]
    ValueUnexpected(String, String),

    #[error("`{0}` render error:\n{1}")]
    Render(String, TemplateRenderError),

    #[error("step `{0}` create:\n{1}")]
    Create(String, Box<dyn std::error::Error + Sync + Send>),
}

pub struct StepRunner {
    action_vec: Arc<TailDropVec<(String, Box<dyn Action>)>>,
}

impl StepRunner {
    pub async fn new(
        flow_app: &dyn FlowApp,
        flow: &Flow,
        task_id: Arc<TaskIdSimple>,
        step_id: String,
    ) -> Result<StepRunner, Error> {
        let obj = flow.step_obj(step_id.as_str());

        let mut action_vec = Vec::with_capacity(obj.len());

        for (aid, _) in obj.iter() {
            let func = flow.step_action_func(step_id.as_str(), aid);

            let create_arg = CreateArgStruct::new(
                flow,
                flow_app.get_handlebars(),
                None,
                task_id.clone(),
                func.into(),
                step_id.clone(),
                aid,
            );

            let action = flow_app
                .get_action_factory()
                .create(&create_arg)
                .await
                .map_err(|e| Create(step_id.clone(), e))?;
            action_vec.push((aid.to_string(), action));
        }

        Ok(StepRunner {
            action_vec: Arc::new(TailDropVec::from(action_vec))
        })
    }


    pub async fn run(&self, arg: &mut RunArgStruct<'_, '_, '_>) -> StepAssessStruct {
        trace!("step start {}", arg.id());
        let start = Utc::now();

        let mut assess_vec = Vec::with_capacity(self.action_vec.len());
        for (aid, action) in self.action_vec.iter() {
            let key: &str = aid;
            let action: &Box<dyn Action> = action;
            let start = Utc::now();
            arg.aid(key);
            let explain = action.explain(arg).await.unwrap_or(Value::Null);
            let value = action.run(arg).await;
            match &value {
                Ok(v) => {
                    info!("step Ok   {}", arg.id());
                    arg.context().insert(key.to_string(), v.as_value().clone());
                    let assess = assess_create(arg, start, explain, value);
                    assess_vec.push(assess);
                },

                Err(e) => {
                    error!(
                        "step Err  {}\n{}\n<<<\n{}",
                        arg.id(),
                        e,
                        explain_to_string(&explain)
                    );
                    break;
                }
            }
        }

        StepAssessStruct::new(
            arg.id().clone(),
            start,
            Utc::now(),
            assess_vec,
        )
    }
}

fn assess_create(
    arg: &mut RunArgStruct<'_, '_, '_>,
    start: DateTime<Utc>,
    explain: Value,
    value: Result<Box<dyn Scope>, chord_core::action::Error>,
) -> ActionAssessStruct {
    let end = Utc::now();
    return if let Err(e) = value.as_ref() {
        error!(
                "step action Err  {}\n{}\n<<<\n{}",
                arg.id(),
                e,
                explain_to_string(&explain),
            );
        ActionAssessStruct::new(
            start,
            end,
            explain,
            StepState::Err(value.err().unwrap()),
        )
    } else {
        info!("step action Ok   {}", arg.id());
        ActionAssessStruct::new(
            start,
            end,
            explain,
            StepState::Ok(value.unwrap()),
        )
    }
}

// fn assert_and_then(arg: &RunArgStruct<'_, '_, '_>) -> Result<(bool, Option<StepThen>), Error> {
//     let assert_success = value_assert(arg, arg.assert()).unwrap_or_else(|e| {
//         debug!("step assert Err {}", e);
//         false
//     });
//     return if !assert_success {
//         Ok((false, None))
//     } else {
//         Ok((true, choose_then(arg)?))
//     };
// }
//
// fn value_assert(
//     arg: &RunArgStruct<'_, '_, '_>,
//     condition: Option<&str>,
// ) -> Result<bool, TemplateRenderError> {
//     if let Some(condition) = condition {
//         assert(arg, condition)
//     } else {
//         Ok(true)
//     }
// }
//
// fn choose_then(arg: &RunArgStruct<'_, '_, '_>) -> Result<Option<StepThen>, Error> {
//     let then_vec = arg.then();
//     if then_vec.is_none() {
//         return Ok(None);
//     }
//     for (idx, then) in then_vec.unwrap().iter().enumerate() {
//         let cond: Option<&str> = then.cond();
//         if cond.is_none()
//             || value_assert(arg, cond).map_err(|e| Render(format!("then.{}.if", idx), e))?
//         {
//             let reg = if let Some(r) = then.reg() {
//                 Some(
//                     arg.render_object(r)
//                         .map_err(|e| Render(format!("then.{}.reg", idx), e))?,
//                 )
//             } else {
//                 None
//             };
//
//             let goto = if let Some(g) = then.goto() {
//                 let goto = arg
//                     .render_str(g)
//                     .map_err(|e| Render(format!("then.{}.goto", idx), e))?;
//                 Some(goto.as_str().map(|s| s.to_string()).ok_or_else(|| {
//                     ValueUnexpected(format!("then.{}.goto", idx), goto.to_string())
//                 })?)
//             } else {
//                 None
//             };
//
//             return Ok(Some(StepThen::new(reg, goto)));
//         }
//     }
//     return Ok(None);
// }
//
// fn assert(arg: &RunArgStruct<'_, '_, '_>, condition: &str) -> Result<bool, TemplateRenderError> {
//     let assert_tpl = format!("{{{{{condition}}}}}", condition = condition);
//     let assert_result = arg.render_str(assert_tpl.as_str())?;
//     Ok(assert_result == "true")
// }

fn explain_to_string(explain: &Value) -> String {
    if explain.is_string() {
        return explain.as_str().unwrap().to_string();
    } else {
        to_string_pretty(&explain).unwrap_or("".to_string())
    }
}

