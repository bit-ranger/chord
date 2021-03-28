use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::vec::Vec;

use async_std::fs::File;
use async_std::io::BufWriter;
use crossbeam::channel::{Receiver, Sender, unbounded};
use futures::AsyncWriteExt;
use futures::executor::block_on;
use log;
use log::{Metadata, Record};
use regex::Regex;
use time::{at, get_time, strftime};

use common::error::Error;

struct ChannelLogger {
    target: Regex,
    sender: Sender<Vec<u8>>
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

            let _ = self.sender.try_send(data);
        }
    }

    fn flush(&self) {}
}

async fn log_thread_func(
    receiver: Receiver<Vec<u8>>,
    mut default_log_writer: BufWriter<File>,
    enable: Arc<AtomicBool>
) {
    loop_write(receiver, &mut default_log_writer, enable).await;

    let _ = default_log_writer.flush().await;
}

async fn loop_write(receiver: Receiver<Vec<u8>>,
                    default_log_writer: &mut BufWriter<File>,
                    enable: Arc<AtomicBool>){
    let recv_timeout = Duration::from_secs(2);
    loop {
        let recv = receiver.recv_timeout(recv_timeout);
        if let Err(_) =  recv {
            if !enable.load(Ordering::SeqCst){
                return;
            }
        }

        let data = recv.unwrap();

        println!("{}", String::from_utf8_lossy(&data));

        let _ = default_log_writer.write_all(&data).await;
    }
}


pub async fn init(
    log_target: String,
    log_file_path: &Path,
    enable: Arc<AtomicBool>
) -> Result<JoinHandle<()>, Error> {
    let (sender, receiver) = unbounded();

    log::set_max_level(log::LevelFilter::Trace);
    let _ = log::set_boxed_logger(Box::new(ChannelLogger {
        sender,
        target: Regex::new(&log_target).unwrap(),
    }));

    let file = async_std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_file_path).await?;
    let default_log_writer = BufWriter::new(file);

    let jh = thread::spawn(move || block_on(
        log_thread_func(receiver, default_log_writer, enable.clone())
    ));

    Ok(jh)
}
