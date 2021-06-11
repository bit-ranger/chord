use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::vec::Vec;

use async_std::fs::File;
use async_std::io::BufWriter;
use futures::executor::block_on;
use futures::AsyncWriteExt;
use log;
use log::{LevelFilter, Metadata, Record};
use time::{at, get_time, strftime};

use chord::Error;
use flume::{bounded, Receiver, Sender};
use itertools::Itertools;

struct ChannelLogger {
    target_level: Vec<(String, LevelFilter)>,
    sender: Sender<Vec<u8>>,
}

impl ChannelLogger {
    fn new(target_level: Vec<(String, String)>, sender: Sender<Vec<u8>>) -> ChannelLogger {
        let target_level = target_level
            .into_iter()
            .map(|(t, l)| {
                if t.starts_with("root") {
                    ("".to_owned(), l)
                } else {
                    (t, l)
                }
            })
            .map(|(t, l)| {
                (
                    t,
                    LevelFilter::from_str(l.as_str()).unwrap_or(log::max_level()),
                )
            })
            .sorted_by(|(a, _), (b, _)| b.cmp(a))
            .collect_vec();

        return ChannelLogger {
            target_level,
            sender,
        };
    }
}

impl log::Log for ChannelLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        for (t, l) in self.target_level.iter() {
            if metadata.target().starts_with(t) {
                return &metadata.level() <= l;
            }
        }

        return metadata.level() <= log::max_level();
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut data = Vec::new();
            let now = at(get_time());
            let date = strftime("%F %T", &now).unwrap();
            let microseconds = now.tm_nsec / 1000;

            let ctx_id = chord_flow::CTX_ID
                .try_with(|c| c.borrow().clone())
                .unwrap_or("".to_owned());

            let _ = write!(
                &mut data,
                "[{}.{:06}][{}][{}:{}] - [{}], {}\n",
                date,
                microseconds,
                record.level(),
                record.target(),
                record.line().unwrap_or(0),
                ctx_id,
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
    enable: Arc<AtomicBool>,
) {
    loop_write(receiver, &mut default_log_writer, enable).await;

    let _ = default_log_writer.flush().await;
}

async fn loop_write(
    receiver: Receiver<Vec<u8>>,
    default_log_writer: &mut BufWriter<File>,
    enable: Arc<AtomicBool>,
) {
    let recv_timeout = Duration::from_secs(2);
    loop {
        let recv = receiver.recv_timeout(recv_timeout);
        if let Err(_) = recv {
            if !enable.load(Ordering::SeqCst) {
                return;
            } else {
                continue;
            }
        }

        let data = recv.unwrap();

        println!("{}", String::from_utf8_lossy(&data));

        let _ = default_log_writer.write_all(&data).await;
    }
}

pub async fn init(
    target_level: Vec<(String, String)>,
    log_file_path: &Path,
) -> Result<LogHandler, Error> {
    let (sender, receiver) = bounded(999999);

    log::set_max_level(LevelFilter::Trace);
    let _ = log::set_boxed_logger(Box::new(ChannelLogger::new(target_level, sender)));

    let file = async_std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_file_path)
        .await?;
    let default_log_writer = BufWriter::new(file);

    let log_enable = Arc::new(AtomicBool::new(true));
    let log_enable_move = log_enable.clone();
    let join_handler = thread::spawn(move || {
        block_on(log_thread_func(
            receiver,
            default_log_writer,
            log_enable_move,
        ))
    });

    Ok(LogHandler {
        log_enable: log_enable.clone(),
        join_handler,
    })
}

pub struct LogHandler {
    log_enable: Arc<AtomicBool>,
    join_handler: JoinHandle<()>,
}

pub async fn terminal(log_handler: LogHandler) -> Result<(), Error> {
    log_handler.log_enable.store(false, Ordering::SeqCst);
    let _ = log_handler.join_handler.join();
    Ok(())
}
