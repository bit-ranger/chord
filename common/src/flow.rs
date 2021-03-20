use crate::value::Json;

pub struct Flow {
    flow: Json
}

impl Flow{

    pub async fn new(flow: Json) -> Flow{
        Flow{
            flow
        }
    }

    pub async fn point_id_vec(self: &Flow) -> Vec<String> {
        let task_point_chain_arr = self.flow["task"]["chain"].as_array().unwrap();
        let task_point_chain_vec: Vec<String> = task_point_chain_arr.iter()
            .map(|e| {
                e.as_str().map(|s| String::from(s)).unwrap()
            })
            .collect();

        return task_point_chain_vec;
    }
}
