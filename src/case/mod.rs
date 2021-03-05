use crate::model::case::{CaseContextStruct, CaseResult};
use crate::model::point::{PointContextStruct, PointResult};
use crate::point::run_point;

pub async fn run_case(context: &mut CaseContextStruct<'_,'_>) -> CaseResult {

    let point_vec: Vec<PointContextStruct> = context.create_point();
    let mut point_result_vec = Vec::<(String, PointResult)>::new();

    for  point in point_vec.iter() {
        let result = run_point(&point).await;

        match &result {
            Ok(r) => {
                point.register_dynamic(r).await;
            },
            Err(_) =>  {
                break;
            }
        }

        point_result_vec.push((String::from(point.get_id()), result));
    }

    return Ok(point_result_vec);
}



