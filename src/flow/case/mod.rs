use crate::flow::case::model::CaseContextStruct;
use crate::flow::point::model::PointContextStruct;
use crate::flow::point::run_point;
use crate::model::context::AppContext;
use crate::model::context::{CaseResult, PointResult};
use crate::model::error::Error;

pub mod model;

pub async fn run_case(app_context: &dyn AppContext, context: &mut CaseContextStruct<'_,'_>) -> CaseResult {
    let render_context = context.create_render_context();
    let point_vec: Vec<PointContextStruct> = context.create_point(app_context, &render_context);
    let mut point_result_vec = Vec::<(String, PointResult)>::new();

    for  point in point_vec.iter() {
        let result = run_point(&point).await;

        match &result {
            Ok(r) => {
                point.register_dynamic(r).await;
            },
            Err(_) =>  {
                return Err(Error::new("000", "point failure"));
            }
        }

        point_result_vec.push((String::from(point.get_id()), result));
    }

    return Ok(point_result_vec);
}



