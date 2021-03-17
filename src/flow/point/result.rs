use chrono::{DateTime, Utc};

use crate::model::value::Json;
use crate::model::point::PointAssess;


pub struct PointAssessStruct {
    result: Json,
    id: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

impl PointAssessStruct {
    pub fn new(result: Json,
               id: &str,
               start: DateTime<Utc>,
               end: DateTime<Utc>) -> PointAssessStruct {
        PointAssessStruct {
            result,
            id: String::from(id),
            start,
            end,
        }
    }


}

impl PointAssess for PointAssessStruct {

    fn id(&self) -> &str {
        &self.id
    }

    fn start(&self) -> DateTime<Utc> {
        self.start
    }

    fn end(&self) -> DateTime<Utc> {
        self.end
    }

    fn result(&self) -> &Json {
        &self.result
    }
}


