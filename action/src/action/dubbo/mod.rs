use chord_core::action::prelude::*;

use crate::err;

mod gateway;
// mod telnet;

pub struct DubboPlayer {
    delegate: Box<dyn Player>,
}

impl DubboPlayer {
    pub async fn new(config: Option<Value>) -> Result<DubboPlayer, Error> {
        if config.is_none() {
            return Err(err!("100", "missing dubbo.config"));
        }
        let config_ref = config.as_ref().unwrap();

        if config_ref.is_null() {
            return Err(err!("101", "missing dubbo.config"));
        }

        let mode = config_ref["mode"]
            .as_str()
            .ok_or(err!("102", "missing dubbo.mode"))?
            .to_owned();

        match mode.as_str() {
            "gateway" => Ok(DubboPlayer {
                delegate: Box::new(gateway::DubboPlayer::new(config).await?),
            }),
            // "telnet" => Ok(DubboPlayer {
            //     delegate: Box::new(telnet::DubboPlayer::new(config).await?),
            // }),
            _ => Err(err!("103", "unsupported mode")),
        }
    }
}

#[async_trait]
impl Player for DubboPlayer {
    async fn action(&self, arg: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        self.delegate.action(arg).await
    }
}
