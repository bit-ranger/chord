use std::fmt::Display;

use chrono::{DateTime, Utc};

pub type Error = Box<dyn std::error::Error + Sync + Send>;

pub trait TaskId: Sync + Send + Display {
    fn task(&self) -> &str;

    fn exec(&self) -> &str;
}


pub enum TaskState {
    Ok,
    Fail(String),
    Err(Error),
}

impl TaskState {
    #[allow(dead_code)]
    pub fn is_ok(&self) -> bool {
        match self {
            TaskState::Ok => true,
            _ => false,
        }
    }
}

pub trait TaskAsset: Sync + Send {
    fn id(&self) -> &dyn TaskId;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn state(&self) -> &TaskState;
}


pub trait StageId: Sync + Send + Display {
    fn task(&self) -> &dyn TaskId;

    fn stage(&self) -> &str;

    fn exec(&self) -> &str;
}

pub enum StageState {
    Ok,
    Fail(String),
    Err(Error),
}

impl StageState {
    #[allow(dead_code)]
    pub fn is_ok(&self) -> bool {
        match self {
            StageState::Ok => true,
            _ => false,
        }
    }
}

pub trait StageAsset: Sync + Send {
    fn id(&self) -> &dyn StageId;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn state(&self) -> &StageState;
}
