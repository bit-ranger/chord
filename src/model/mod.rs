use std::collections::{BTreeMap, HashMap};
use async_std::sync::{Arc, RwLock};
use std::borrow::Borrow;
use std::ops::Index;
use handlebars::Handlebars;
use serde::Serialize;
use serde_json::{Value, to_value};

#[derive(Debug)]
pub struct TaskContext {
    data: Vec<BTreeMap<String,String>>,
    config: Value
}


impl TaskContext {

    pub fn new(config: Value, data: Vec<BTreeMap<String,String>>) -> TaskContext {
        let context = TaskContext {
            config,
            data
        };
        return context;
    }

    pub fn share(self: TaskContext) -> SharedTaskContext{
        Arc::new(RwLock::new(self))
    }

    pub async fn create_case(task_ctx: SharedTaskContext) -> Vec<CaseContext>{
        return task_ctx.read().await.data.iter()
            .enumerate()
            .map(|(idx, _d)| {
                CaseContext::new(
                    task_ctx.clone(),
                    idx
                )
            })
            .collect();
    }

    pub async fn get_point_vec(self: &TaskContext) -> Vec<String>{
        let task_point_chain_arr = self.config["task"]["chain"].as_array().unwrap();
        let task_point_chain_vec:Vec<String> = task_point_chain_arr.iter()
            .map(|e| {
                e.as_str().map(|s|String::from(s)).unwrap()
            })
            .collect();

        return task_point_chain_vec;
    }
}

#[derive(Debug)]
pub struct CaseContext {

    task_context: SharedTaskContext,
    data_index: usize,
    point_context_register: HashMap<String, HashMap<String,Value>>
}


impl CaseContext {

    fn new(task_context: SharedTaskContext, data_index: usize) -> CaseContext{
        let context = CaseContext {
            task_context,
            data_index,
            point_context_register: HashMap::new()
        };

        return context;
    }

    pub fn share(self: CaseContext) -> SharedCaseContext{
        Arc::new(RwLock::new(self))
    }




    pub async fn create_point(case_ctx: SharedCaseContext) -> Vec<PointContext>{
        return case_ctx.read().await
            .task_context.read().await
            .get_point_vec().await
            .into_iter()
            // .filter(|point_id| async {
            //     let none = case_ctx.read().await
            //         .task_context.read().await
            //         .config["point"][point_id].as_object().is_none();
            //     if none {
            //         panic!("missing point config {}", point_id);
            //     } else {
            //         return true;
            //     }
            // })
            .map(|point_id| {
                PointContext::new(
                    case_ctx.clone(),
                    String::from(point_id)
                )
            })
            .collect();
    }


}


#[derive(Debug)]
pub struct PointContext{
    case_context: SharedCaseContext,
    point_id: String,
    context_register: HashMap<String,Value>
}


impl PointContext {

    fn new(case_context: SharedCaseContext, point_id: String) -> PointContext{
        let context = PointContext {
            case_context,
            point_id: String::from(point_id),
            context_register: HashMap::new()
        };
        return context;
    }

    pub fn share(self: PointContext) -> SharedPointContext{
        Arc::new(RwLock::new(self))
    }

    pub async fn get_config_str(self: &PointContext, path: Vec<&str>) -> Option<String>
    {
        let config = self.case_context.read().await
            .task_context.read().await
            .config["point"][&self.point_id]["config"].borrow();

        let raw_config = path.iter()
            .fold(config,
                  |acc, k| acc[k].borrow()
            );

        match raw_config.as_str(){
            Some(s) => Some(self.render(s, &Value::Null).await),
            None=> None
        }

    }

    pub async fn get_meta_str(&self, path: Vec<&str>) ->Option<String>
    {
        let config = self.case_context.read().await
            .task_context.read().await
            .config["point"][&self.point_id].borrow();

        let raw_config = path.iter()
            .fold(config,
                  |acc, k| acc[k].borrow()
            );

        return raw_config.as_str().map(|x| async {self.render(x, &Value::Null).await});
    }


    async fn render<T>(self: &PointContext, text: &str, ext_data: &T) -> String
        where
            T: Serialize
    {
        let mut data :HashMap<&str, Value> = HashMap::new();

        let config_def = self.case_context.read().await
            .task_context.read().await
            .config["task"]["def"].as_object();
        match config_def{
            Some(def) => {
                data.insert("def", to_value(def).unwrap());
            },
            None => {}
        }

        let case_data = self.case_context.read().await.task_context.data[self.case_context.data_index].borrow();
        data.insert("data", to_value(case_data).unwrap());


        data.insert("ext", to_value(ext_data).unwrap());

        let mut handlebars = Handlebars::new();
        let render = handlebars.render_template(text, &data).unwrap();
        return render;
    }

    pub async fn assert <T>(&self, condition: &str, ext_data: &T) -> bool
        where
            T: Serialize
    {
        let template = format!(
            "{{{{#if {condition}}}}}true{{{{else}}}}false{{{{/if}}}}",
            condition = condition
        );

        let result = self.render(&template, ext_data).await;
        return if result.eq("true") {true} else {false};
    }

    pub fn register_context<T>(&mut self, name: String, context: T)
        where
            T: Serialize{
        self.context_register.insert(name, to_value(context).unwrap());
    }


}


pub type PointResult = std::result::Result<Value, ()>;
pub type SharedPointContext = Arc<RwLock<PointContext>>;
pub type SharedCaseContext = Arc<RwLock<CaseContext>>;
pub type SharedTaskContext = Arc<RwLock<TaskContext>>;
