use chrono::{DateTime, Utc};

use chord_common::point::{PointAssess, PointState, PointId};
use crate::flow::point::arg::PointIdStruct;

pub struct PointAssessStruct {
    id: PointIdStruct,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    state: PointState,
}

impl PointAssessStruct {
    pub fn new(
        id: PointIdStruct,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        state: PointState,
    ) -> PointAssessStruct {
        PointAssessStruct {
            id,
            start,
            end,
            state,
        }
    }
}

impl PointAssess for PointAssessStruct {
    fn id(&self) -> &dyn PointId {
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

unsafe impl Send for PointAssessStruct {}

unsafe impl Sync for PointAssessStruct {}
