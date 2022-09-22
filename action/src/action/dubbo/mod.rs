use chord_core::action::prelude::*;

use crate::err;

mod gateway;
// mod telnet;

pub struct DubboCreator {
    delegate: Box<dyn Creator>,
}

impl DubboCreator {
    pub async fn new(config: Option<Value>) -> Result<DubboCreator, Error> {
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
            "gateway" => Ok(DubboCreator {
                delegate: Box::new(gateway::DubboCreator::new(config).await?),
            }),
            // "telnet" => Ok(DubboCreator {
            //     delegate: Box::new(telnet::DubboCreator::new(config).await?),
            // }),
            _ => Err(err!("103", "unsupported mode")),
        }
    }
}

#[async_trait]
impl Creator for DubboCreator {
    async fn create(&self, chord: &dyn Chord, arg: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        self.delegate.create(chord, arg).await
    }
}
