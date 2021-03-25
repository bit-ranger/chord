use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::vec::Vec;

use async_std::fs::File;
use async_std::fs::OpenOptions;
use async_std::io::BufWriter;
use async_std::prelude::*;
use crossbeam::channel::{Receiver, Sender, unbounded};
use futures::executor::block_on;
use log;
use log::{Metadata, Record};
use regex::Regex;
use time::{at, get_time, strftime};

use common::error::Error;

struct ChannelLogger {
    target: Regex,
    sender: Sender<(String,Vec<u8>)>
}

impl log::Log for ChannelLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level() && self.target.is_match(metadata.target())
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut data = Vec::new();
            let now = at(get_time());
            let date = strftime("%F %T", &now).unwrap();
            let microseconds = now.tm_nsec / 1000;

            let _ = write!(
                &mut data,
                "[{}.{:06}][{}][{}:{}] - {}\n",
                date,
                microseconds,
                record.level(),
                record.target(),
                record.line().unwrap_or(0),
                record.args()
            );

            // let log_path:String = log_mdc::get("work_path", |x| x.unwrap_or("").into());
            let _ = self.sender.try_send(("".into(), data));
        }
    }

    fn flush(&self) {}
}

async fn log_thread_func(
    execution_id: String,
    receiver: Receiver<(String,Vec<u8>)>,
    mut default_log_writer: async_std::io::BufWriter<async_std::fs::File>,
    enable: Arc<AtomicBool>
) {
    let mut log_writer_map = HashMap::<String, BufWriter<File>>::new();

    loop_write(execution_id, receiver, &mut default_log_writer, &mut log_writer_map, enable).await;

    let _ = default_log_writer.flush().await;
    for (_,mut v) in log_writer_map {
        let _ = v.flush();
    }
}

async fn loop_write(execution_id: String,
                    receiver: Receiver<(String,Vec<u8>)>,
                    default_log_writer: &mut async_std::io::BufWriter<async_std::fs::File>,
                    log_writer_map: &mut HashMap<String, BufWriter<File>>,
                    enable: Arc<AtomicBool>){
    let recv_timeout = Duration::from_secs(2);
    loop {
        let recv = receiver.recv_timeout(recv_timeout);
        if let Err(_) =  recv {
            if !enable.load(Ordering::SeqCst){
                return;
            } else {
                continue;
            }
        }

        let (log_path, data) = recv.unwrap();

        println!("{}", String::from_utf8_lossy(&data));

        if log_path.is_empty() {
            let _ = default_log_writer.write_all(&data).await;
            continue;
        }

        let writer = log_writer_map.get_mut(log_path.as_str());
        match writer {
            Some(writer) => {
                let _ = writer.write_all(&data).await;
            },
            None => {
                let log_file_path = Path::new(&log_path).join(execution_id.as_str()).join("log.log");
                let file = File::create(log_file_path).await;
                match file {
                    Ok(file) => {
                        let mut writer = BufWriter::new(file);
                        let _ = writer.write_all(&data).await;
                        log_writer_map.insert(log_path, writer);
                    },
                    Err(_) => {
                        let log_file_path = Path::new(&log_path).join(execution_id.as_str()).join("log.log");
                        println!("failed to create file {}", log_file_path.to_str().unwrap());
                    }
                }

            }
        }
    }
}


pub async fn init(
    execution_id: String,
    log_target: String,
    default_log_path: String,
    enable: Arc<AtomicBool>
) -> Result<JoinHandle<()>, Error> {
    let (sender, receiver) = unbounded();

    log::set_max_level(log::LevelFilter::Trace);
    let _ = log::set_boxed_logger(Box::new(ChannelLogger {
        sender,
        target: Regex::new(&log_target).unwrap(),
    }));

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&default_log_path).await?;
    let default_log_writer = async_std::io::BufWriter::new(file);

    let jh = thread::spawn(move || block_on(
        log_thread_func(execution_id, receiver, default_log_writer, enable.clone())
    ));

    Ok(jh)
}
