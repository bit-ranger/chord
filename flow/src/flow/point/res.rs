use chrono::{DateTime, Utc};

use common::point::{PointAssess, PointState};

pub struct PointAssessStruct {
    id: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    state: PointState
}

impl PointAssessStruct {
    pub fn new(id: &str,
               start: DateTime<Utc>,
               end: DateTime<Utc>,
               state: PointState) -> PointAssessStruct {
        PointAssessStruct {
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
}


unsafe impl Send for PointAssessStruct
{
}

unsafe impl Sync for PointAssessStruct
{
}

