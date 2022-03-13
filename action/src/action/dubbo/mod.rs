use chord_core::action::prelude::*;

use crate::err;

mod gateway;
// mod telnet;

pub struct DubboFactory {
    delegate: Box<dyn Factory>,
}

impl DubboFactory {
    pub async fn new(config: Option<Value>) -> Result<DubboFactory, Error> {
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
            "gateway" => Ok(DubboFactory {
                delegate: Box::new(gateway::DubboFactory::new(config).await?),
            }),
            // "telnet" => Ok(DubboFactory {
            //     delegate: Box::new(telnet::DubboFactory::new(config).await?),
            // }),
            _ => Err(err!("103", "unsupported mode")),
        }
    }
}

#[async_trait]
impl Factory for DubboFactory {
    async fn create(&self, arg: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        self.delegate.create(arg).await
    }
}
