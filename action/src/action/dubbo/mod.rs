use chord_core::action::prelude::*;

use crate::err;

mod gateway;
// mod telnet;

pub struct DubboAction {
    delegate: Box<dyn Action>,
}

impl DubboAction {
    pub async fn new(config: Option<Value>) -> Result<DubboAction, Error> {
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
            "gateway" => Ok(DubboAction {
                delegate: Box::new(gateway::DubboAction::new(config).await?),
            }),
            // "telnet" => Ok(DubboAction {
            //     delegate: Box::new(telnet::DubboAction::new(config).await?),
            // }),
            _ => Err(err!("103", "unsupported mode")),
        }
    }
}

#[async_trait]
impl Action for DubboAction {
    async fn play(&self, arg: &dyn Arg) -> Result<Box<dyn Play>, Error> {
        self.delegate.play(arg).await
    }
}
