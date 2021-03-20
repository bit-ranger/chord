use chrono::{DateTime, Utc};

use common::value::Json;

use common::point::{PointAssess, PointState};

pub struct PointAssessStruct {
    result: Json,
    id: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    state: PointState
}

impl PointAssessStruct {
    pub fn new(result: Json,
               id: &str,
               start: DateTime<Utc>,
               end: DateTime<Utc>,
               state: PointState) -> PointAssessStruct {
        PointAssessStruct {
            result,
            id: String::from(id),
            start,
            end,
            state,
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

    fn state(&self) -> &PointState {
        &self.state
    }

    fn result(&self) -> &Json {
        &self.result
    }
}


