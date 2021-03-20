use common::point::PointArg;
use std::str::FromStr;

pub mod restapi;
pub mod md5;
pub mod dubbo;
pub mod sleep;


fn config_rendered_default<T>(point_arg: &dyn PointArg, path: Vec<&str>, default: T) -> Result<T, T::Err> where T: FromStr {
    match point_arg.config_rendered(path){
        Some(s) => s.parse(),
        None=> Ok(default)
    }
}