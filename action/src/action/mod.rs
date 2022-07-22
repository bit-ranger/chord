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

pub struct PlayerComposite {
    table: HashMap<String, Box<dyn Player>>,
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

impl PlayerComposite {
    pub async fn new(config: Option<Value>) -> Result<PlayerComposite, Error> {
        let mut table: HashMap<String, Box<dyn Player>> = HashMap::new();

        let config_ref = config.as_ref();

        register!(table, config_ref, "let", lets::LetPlayer::new);
        register!(table, config_ref, "set", set::SetPlayer::new);
        register!(table, config_ref, "block", block::BlockPlayer::new);
        register!(table, config_ref, "while", whiles::WhilePlayer::new);
        register!(table, config_ref, "match", matches::MatchPlayer::new);
        register!(table, config_ref, "assert", assert::AssertPlayer::new);
        register!(table, config_ref, "sleep", sleep::SleepPlayer::new);
        register!(table, config_ref, "log", log::LogPlayer::new);
        register!(table, config_ref, "count", count::CountPlayer::new);

        #[cfg(feature = "act_restapi")]
        register!(table, config_ref, "restapi", restapi::RestapiPlayer::new);

        #[cfg(feature = "act_crypto")]
        register!(table, config_ref, "crypto", crypto::CryptoPlayer::new);

        #[cfg(feature = "act_url")]
        register!(table, config_ref, "url", url::UrlPlayer::new);

        #[cfg(feature = "act_database")]
        register!(table, config_ref, "database", database::DatabasePlayer::new);

        #[cfg(feature = "act_redis")]
        register!(table, config_ref, "redis", redis::RedisPlayer::new);

        #[cfg(feature = "act_mongodb")]
        register!(table, config_ref, "mongodb", mongodb::MongodbPlayer::new);

        #[cfg(feature = "act_lua")]
        register!(table, config_ref, "lua", lua::LuaPlayer::new);

        #[cfg(feature = "act_program")]
        register!(table, config_ref, "program", program::ProgramPlayer::new);

        #[cfg(feature = "act_dubbo")]
        register!(table, config_ref, "dubbo", dubbo::DubboPlayer::new);

        #[cfg(feature = "act_cdylib")]
        register!(table, config_ref, "cdylib", cdylib::CdylibPlayer::new);

        #[cfg(feature = "act_docker")]
        register!(table, config_ref, "docker", docker::Docker::new);

        Ok(PlayerComposite { table })
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

impl From<PlayerComposite> for HashMap<String, Box<dyn Player>> {
    fn from(player: PlayerComposite) -> Self {
        player.table
    }
}
