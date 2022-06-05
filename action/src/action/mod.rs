use std::collections::HashMap;

use chord_core::action::prelude::*;

mod assert;
mod count;
// mod iter;
mod block;
mod lets;
mod log;
mod matches;
mod set;
mod sleep;
mod whiles;

#[cfg(feature = "act_cdylib")]
mod cdylib;
#[cfg(feature = "act_crypto")]
mod crypto;
#[cfg(feature = "act_database")]
mod database;
#[cfg(feature = "act_docker")]
mod docker;
#[cfg(feature = "act_dubbo")]
mod dubbo;
#[cfg(feature = "act_lua")]
mod lua;
#[cfg(feature = "act_mongodb")]
mod mongodb;
#[cfg(feature = "act_program")]
mod program;
#[cfg(feature = "act_redis")]
mod redis;
#[cfg(feature = "act_restapi")]
mod restapi;
#[cfg(feature = "act_url")]
mod url;

pub struct ActionComposite {
    table: HashMap<String, Box<dyn Action>>,
}

macro_rules! register {
    ($table:ident, $config_ref:ident, $name:expr, $module:path) => {
        if enable($config_ref, $name) {
            $table.insert(
                $name.into(),
                Box::new($module($config_ref.map(|c| c[$name].clone())).await?),
            );
        }
    };
}

impl ActionComposite {
    pub async fn new(config: Option<Value>) -> Result<ActionComposite, Error> {
        let mut table: HashMap<String, Box<dyn Action>> = HashMap::new();

        let config_ref = config.as_ref();

        register!(table, config_ref, "let", lets::LetAction::new);
        register!(table, config_ref, "set", set::SetAction::new);
        register!(table, config_ref, "block", block::BlockAction::new);
        register!(table, config_ref, "while", whiles::WhileAction::new);
        register!(table, config_ref, "match", matches::MatchAction::new);
        register!(table, config_ref, "assert", assert::AssertAction::new);
        register!(table, config_ref, "sleep", sleep::SleepAction::new);
        register!(table, config_ref, "log", log::LogAction::new);
        register!(table, config_ref, "count", count::CountAction::new);

        #[cfg(feature = "act_restapi")]
        register!(table, config_ref, "restapi", restapi::RestapiAction::new);

        #[cfg(feature = "act_crypto")]
        register!(table, config_ref, "crypto", crypto::CryptoAction::new);

        #[cfg(feature = "act_url")]
        register!(table, config_ref, "url", url::UrlAction::new);

        #[cfg(feature = "act_database")]
        register!(table, config_ref, "database", database::DatabaseAction::new);

        #[cfg(feature = "act_redis")]
        register!(table, config_ref, "redis", redis::RedisAction::new);

        #[cfg(feature = "act_mongodb")]
        register!(table, config_ref, "mongodb", mongodb::MongodbAction::new);

        #[cfg(feature = "act_lua")]
        register!(table, config_ref, "lua", lua::LuaAction::new);

        #[cfg(feature = "act_program")]
        register!(table, config_ref, "program", program::ProgramAction::new);

        #[cfg(feature = "act_dubbo")]
        register!(table, config_ref, "dubbo", dubbo::DubboAction::new);

        #[cfg(feature = "act_cdylib")]
        register!(table, config_ref, "cdylib", cdylib::CdylibAction::new);

        #[cfg(feature = "act_docker")]
        register!(table, config_ref, "docker", docker::Docker::new);

        Ok(ActionComposite { table })
    }
}

fn enable(config: Option<&Value>, action_name: &str) -> bool {
    let default_enable = true;
    if config.is_none() {
        return default_enable;
    }
    let config_ref = config.unwrap();
    if config_ref.is_null() {
        return default_enable;
    }

    return config_ref[action_name]["enable"]
        .as_bool()
        .unwrap_or(default_enable);
}

impl From<ActionComposite> for HashMap<String, Box<dyn Action>> {
    fn from(fac: ActionComposite) -> Self {
        fac.table
    }
}
