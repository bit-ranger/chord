use std::sync::Arc;

use chrono::{DateTime, Utc};

use chord_core::case::{CaseAsset, CaseId, CaseState};
use chord_core::value::Value;

use crate::flow::case::arg::CaseIdStruct;

pub struct CaseAssetStruct {
    id: Arc<CaseIdStruct>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    data: Value,
    state: CaseState,
}

impl CaseAssetStruct {
    pub fn new(
        id: Arc<CaseIdStruct>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        data: Value,
        state: CaseState,
    ) -> CaseAssetStruct {
        CaseAssetStruct {
            id,
            start,
            end,
            data,
            state,
        }
    }
}

impl CaseAsset for CaseAssetStruct {
    fn id(&self) -> &dyn CaseId {
        self.id.as_ref()
    }
    fn start(&self) -> DateTime<Utc> {
        self.start
    }
    fn end(&self) -> DateTime<Utc> {
        self.end
    }

    fn data(&self) -> &Value {
        &self.data
    }

    fn state(&self) -> &CaseState {
        &self.state
    }
}
