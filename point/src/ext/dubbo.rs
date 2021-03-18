use telnet::{Telnet, TelnetEvent};
use common::point::PointArg;
use crate::model::PointValue;
use common::value::Json;
use log::info;
use crate::err;

pub async fn run(context: &dyn PointArg) -> PointValue {
    let connection = Telnet::connect(("localhost", 123), 256);

    let mut connection = match connection {
        Ok(connection) => connection,
        Err(err) => {
            return err!("connection error", format!("{}", err).as_str());
        }
    };


    let res = connection.read();
    match res {
        Ok(event) => {
            match event {
                TelnetEvent::Data(d) => {
                    let value = String::from_utf8(Vec::from(d)).unwrap();
                    info!("{}", value);
                    return Ok(Json::String(value));
                }
                _ => {
                    return err!("0", "0")
                }
            }

        },
        Err(e) => {
            return err!("1", "1")
        }
    }



}