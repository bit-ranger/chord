use log::{debug};
use async_std::net::TcpStream;
use async_std::prelude::*;
use chord_common::value::{Json};
use std::str::FromStr;
use chord_common::point::{PointArg, PointValue};
use chord_common::{rerr};

pub async fn run(arg: &dyn PointArg) -> PointValue {
    let mut server_stream = match TcpStream::connect(arg.config_rendered(vec!["address"]).unwrap()).await {
        Ok(server_stream) => server_stream,
        Err(e) => {
            return rerr!("connection error", format!("{}", e));
        }
    };

    let invoke = arg.config_rendered(vec!["invoke"]).unwrap();
    let invoke = format!("invoke {}", invoke);
    server_stream.write_all(invoke.as_bytes()).await?;

    let suffix ="dubbo>".as_bytes();
    let mut response = vec![0; 0];
    let mut seek_idx = 0;
    loop {
        let mut buf = vec![0; 50];
        server_stream.read(&mut buf).await.unwrap();
        response.extend(&buf);
        match sub_vec_index(&response[seek_idx..], &suffix) {
            Some(i) => {
                response.truncate(seek_idx + i);
                break;
            },
            None => {
                seek_idx = std::cmp::min(response.len()-1,response.len()-suffix.len());
            }
        }
    }

    let mut value = unsafe { String::from_utf8_unchecked(Vec::from(response)) };
    debug!("Response: {}", value);
    let i = value.rfind("\r\nelapsed:");
    match i {
        Some(i) => {
            value.truncate(i);
            let json = Json::from_str(value.as_str())?;
            PointValue::Ok(json)
        },
        None => {
            rerr!("001", value)
        }
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
            sub_vec_index = i+1;
        }
    }

    return None;
}


#[test]
fn sub_vec_index_test() {
    let vec = vec![0,1,2,3,4,5,6,7,8];
    assert_eq!(3, sub_vec_index(&vec, &vec![3,4,5]).unwrap());

    assert_eq!(6, sub_vec_index(&vec, &vec![6,7,8]).unwrap());

    assert_eq!(true, sub_vec_index(&vec, &vec![7,8, 9]).is_none());
}