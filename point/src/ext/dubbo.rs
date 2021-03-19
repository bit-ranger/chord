use common::point::PointArg;
use crate::model::PointValue;
use log::info;
use crate::{err, err_raw};
use async_std::net::TcpStream;
use async_std::prelude::*;


pub async fn run(arg: &dyn PointArg) -> PointValue {
    let mut server_stream = match TcpStream::connect(arg.config_rendered(vec!["address"]).unwrap()).await {
        Ok(server_stream) => server_stream,

        Err(_) => {
            return err!("connection error", "");
        }
    };

    let cmd = arg.config_rendered(vec!["cmd"]).unwrap();
    server_stream.write_all(cmd.as_bytes()).await?;

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
    let i = value.rfind("\r\nelapsed:").ok_or(err_raw!("0", "elapsed"))?;
    value.truncate(i);


    info!("Data {}", value);
    return err!("Data", "");
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