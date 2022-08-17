use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::vec::Vec;

use colored::*;
use flume::{bounded, Receiver, Sender};
use itertools::Itertools;
use log;
use log::{Level, LevelFilter, Metadata, Record};

use chord_core::future::runtime::Handle;

use chrono::{Local};

#[derive(thiserror::Error, Debug)]
pub enum Error {}

struct ChannelLogger {
    target_level: Vec<(String, LevelFilter)>,
    sender: Sender<(log::Level, String)>,
}

impl ChannelLogger {
    fn new(
        target_level: Vec<(String, String)>,
        sender: Sender<(log::Level, String)>,
    ) -> ChannelLogger {
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
            let now = Local::now();

            let ctx_id = chord_flow::CTX_ID
                .try_with(|c| c.clone())
                .unwrap_or("".to_owned());

            let data = format!(
                "{} {:<5} {:<5} --- {:<30} : [{}] {}\n",
                now.format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                std::process::id(),
                format!("{}:{}", record.target(), record.line().unwrap_or(0)),
                ctx_id,
                record.args()
            );

            let _ = self.sender.try_send((record.level(), data));
        }
    }

    fn flush(&self) {}
}

async fn log_thread_func(receiver: Receiver<(log::Level, String)>, enable: Arc<AtomicBool>) {
    loop_write(receiver, enable).await;
}

async fn loop_write(receiver: Receiver<(log::Level, String)>, enable: Arc<AtomicBool>) {
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

        let (level, data) = recv.unwrap();
        println!("{}", color_level(level, data));
    }
}

fn color_level(level: log::Level, data: String) -> ColoredString {
    match level {
        Level::Error => data.truecolor(0xff, 0x6b, 0x68),
        Level::Warn => data.truecolor(0xa6, 0x6f, 0x00),
        Level::Info => data.truecolor(0xc0, 0xc0, 0xc0),
        Level::Debug => data.truecolor(0x93, 0x93, 0x93),
        Level::Trace => data.truecolor(0x5e, 0x5e, 0x5e),
    }
}

pub struct Log {
    enable: Arc<AtomicBool>,
    join_handle: JoinHandle<()>,
}

impl Log {
    pub async fn new(target_level: Vec<(String, String)>) -> Result<Log, Error> {
        let (sender, receiver) = bounded(999999);

        log::set_max_level(LevelFilter::Trace);
        let _ = log::set_boxed_logger(Box::new(ChannelLogger::new(target_level, sender)));

        let enable = Arc::new(AtomicBool::new(true));
        let enable_clone = enable.clone();

        let handle = Handle::current();
        let join_handler =
            thread::spawn(move || handle.block_on(log_thread_func(receiver, enable_clone)));

        Ok(Log {
            enable: enable.clone(),
            join_handle: join_handler,
        })
    }

    pub async fn drop(self) {
        self.enable.store(false, Ordering::SeqCst);
        let _ = self.join_handle.join();
    }
}
