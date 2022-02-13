use chord_core::future::fs::File;
use chord_core::future::io::AsyncWriteExt;
use chord_core::future::io::BufWriter;
use chord_core::future::path::exists;
use Error::Create;
use flume::{bounded, Receiver, Sender};
use futures::executor::block_on;
use itertools::Itertools;
use log;
use log::{LevelFilter, Metadata, Record};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::vec::Vec;
use time::{at, get_time, strftime};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to create log file: {0}")]
    Create(std::io::Error),
}

struct ChannelLogger {
    target_level: Vec<(String, LevelFilter)>,
    sender: Sender<String>,
}

impl ChannelLogger {
    fn new(target_level: Vec<(String, String)>, sender: Sender<String>) -> ChannelLogger {
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
            let now = at(get_time());
            let date = strftime("%F %T", &now).unwrap();
            let ms = now.tm_nsec / 1000000;

            let ctx_id = chord_flow::CTX_ID
                .try_with(|c| c.clone())
                .unwrap_or("".to_owned());

            let data = format!(
                "{}.{:03}  {:<5} {:<5} --- {:<30} : [{}] {}\n",
                date,
                ms,
                record.level(),
                std::process::id(),
                format!("{}:{}", record.target(), record.line().unwrap_or(0)),
                ctx_id,
                record.args()
            );

            let _ = self.sender.try_send(data);
        }
    }

    fn flush(&self) {}
}

async fn log_thread_func(
    receiver: Receiver<String>,
    mut default_log_writer: BufWriter<File>,
    enable: Arc<AtomicBool>,
) {
    loop_write(receiver, &mut default_log_writer, enable).await;

    let _ = default_log_writer.flush().await;
}

async fn loop_write(
    receiver: Receiver<String>,
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

        println!("{}", data);

        let _ = default_log_writer.write_all(data.as_bytes()).await;
    }
}

pub async fn init(
    target_level: Vec<(String, String)>,
    log_file_path: &Path,
) -> Result<LogHandler, Error> {
    let log_file_parent_path = log_file_path.parent();
    if log_file_parent_path.is_some() {
        let log_file_parent_path = log_file_parent_path.unwrap();
        if exists(log_file_parent_path).await {
            use chord_core::future::fs::create_dir_all
            (log_file_parent_path)
                .await
                .map_err(|e| Create(e))?;
        }
    }

    let (sender, receiver) = bounded(999999);

    log::set_max_level(LevelFilter::Trace);
    let _ = log::set_boxed_logger(Box::new(ChannelLogger::new(target_level, sender)));

    let file = use chord_core::future::fs::OpenOptions::new
    ()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_file_path)
        .await
        .map_err(|e| Error::Create(e))?;
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

#[allow(dead_code)]
pub async fn terminal(log_handler: LogHandler) {
    log_handler.log_enable.store(false, Ordering::SeqCst);
    let _ = log_handler.join_handler.join();
}
