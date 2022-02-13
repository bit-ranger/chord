use chord_core::action::prelude::*;
use chord_core::future::net::{Shutdown, TcpStream};
use chord_core::value::to_string;
use futures::io::{AsyncReadExt, AsyncWriteExt};
use log::trace;
use std::str::FromStr;

pub struct DubboFactory {
    address: String,
}

impl DubboFactory {
    pub async fn new(config: Option<Value>) -> Result<DubboFactory, Error> {
        if config.is_none() {
            return Err(err!("dubbo", "missing dubbo.config"));
        }

        let config = config.as_ref().unwrap();

        let address = config["telnet"]["address"]
            .as_str()
            .ok_or(err!("010", "missing telnet.address"))?;
        Ok(DubboFactory {
            address: address.to_owned(),
        })
    }
}

#[async_trait]
impl Factory for DubboFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Dubbo {
            address: self.address.clone(),
        }))
    }
}

struct Dubbo {
    address: String,
}

#[async_trait]
impl Action for Dubbo {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let mut stream = TcpStream::connect(self.address.as_str())
            .await
            .map_err(|e| err!("connection error", format!("{}", e)))?;

        let method_long = args["method"]
            .as_str()
            .ok_or(err!("010", "missing method"))?;
        let parts = method_long
            .split(&['#', '(', ',', ')'][..])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>();
        if parts.len() < 2 {
            return Err(err!("010", "invalid method"));
        }

        let args_raw = &args["args"];
        let args: Vec<Value> = match args_raw {
            Value::Array(aw_vec) => {
                let mut ar_vec: Vec<Value> = vec![];
                for aw in aw_vec {
                    let ar = arg.render_value(aw)?;
                    ar_vec.push(ar);
                }
                ar_vec
            }
            _ => arg
                .render_value(args_raw)?
                .as_array()
                .ok_or(err!("010", "missing args"))?
                .clone(),
        };
        let args_invoke: Vec<String> = args
            .into_iter()
            .map(|a| to_string(&a))
            .filter(|a| a.is_ok())
            .map(|a| a.unwrap())
            .collect();
        let args_invoke = args_invoke.join(",");

        let invoke = format!("invoke {}.{}({})", parts[0], parts[1], args_invoke);
        trace!("{}", invoke);
        stream.write_all(invoke.as_bytes()).await?;
        stream.shutdown(Shutdown::Write)?;

        let suffix = "dubbo>".as_bytes();
        let mut response = vec![0; 0];
        let mut seek_idx = 0;
        let mut resp = String::new();
        let size = stream.read_to_string(&mut resp);
        // loop {
        //     let rx = stream.read(&mut buf).await?;
        //     if
        //     response.extend(&buf);
        //     println!("{}", unsafe {
        //         String::from_utf8_unchecked(response.clone())
        //     });
        //     match sub_vec_index(&response[seek_idx..], &suffix) {
        //         Some(i) => {
        //             response.truncate(seek_idx + i);
        //             break;
        //         }
        //         None => {
        //             seek_idx = std::cmp::min(response.len() - 1, response.len() - suffix.len());
        //         }
        //     }
        //     if rx.is_err() {
        //         break;
        //     }
        // }

        // let mut value = unsafe { String::from_utf8_unchecked(Vec::from(response)) };
        trace!("response: {}", resp);
        // let i = resp.rfind("\r\nelapsed:");
        // match i {
        //     Some(i) => {
        //         value.truncate(i);
        //         let json = Value::from_str(value.as_str())?;
        //         Ok(Box::new(json))
        //     }
        //     None => Err(err!("001", value)),
        // }

        let json = Value::from_str(resp.as_str())?;
        Ok(Box::new(json))
    }
}

fn sub_vec_index(vec: &[u8], sub_vec: &[u8]) -> Option<usize> {
    let mut sub_vec_index = 0;
    let mut match_size = 0;
    for (i, u) in vec.iter().enumerate() {
        if sub_vec[i - sub_vec_index].eq(u) {
            match_size += 1;
            if match_size == sub_vec.len() {
                return Some(sub_vec_index);
            }
        } else {
            sub_vec_index = i + 1;
        }
    }

    return None;
}

#[test]
fn sub_vec_index_test() {
    let vec = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
    assert_eq!(3, sub_vec_index(&vec, &vec![3, 4, 5]).unwrap());

    assert_eq!(6, sub_vec_index(&vec, &vec![6, 7, 8]).unwrap());

    assert_eq!(true, sub_vec_index(&vec, &vec![7, 8, 9]).is_none());
}
