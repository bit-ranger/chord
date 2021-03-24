use log;
use log::{Metadata, Record};
use std::collections::vec_deque::VecDeque;
use std::fs::{File};
use std::io::{Write, BufWriter};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::vec::Vec;
use time::{at, get_time, strftime};
use regex::Regex;
use futures::executor::block_on;
use common::error::Error;
use std::collections::HashMap;
use futures::AsyncWriteExt;
use std::path::Path;
use std::thread::JoinHandle;
use std::sync::atomic::{AtomicBool, Ordering};

struct ChannelLogger {
    target: Regex,
    msg_queue: Arc<(Mutex<VecDeque<(String, Vec<u8>)>>, Condvar)>,
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

            println!("{}", String::from_utf8_lossy(&data));

            // let log_path:String = log_mdc::get("work_path", |x| x.unwrap_or("").into());



            // if log_path.is_empty() {
            //     println!("{}", String::from_utf8_lossy(&data));
            // } else {
            //     let log_file_path = Path::new(&log_path).join("log.log");
            //     println!("{}", log_file_path.to_str().unwrap());
            //     let mut file = std::fs::OpenOptions::new()
            //         .create(true)
            //         .write(true)
            //         .append(true)
            //         .open(&log_file_path).unwrap();
            //     let mut writer = std::io::BufWriter::new(file);
            //     writer.write_all(&data).unwrap();
            // }

            // let &(ref lock, ref cvar) = &*self.msg_queue;
            // let mut queue = lock.lock().unwrap();
            // queue.push_back((log_path,data));
            // cvar.notify_one();
        }
    }

    fn flush(&self) {}
}

async fn log_thread_func(
    execution_id: String,
    msg_queue: Arc<(Mutex<VecDeque<(String, Vec<u8>)>>, Condvar)>,
    mut default_log_writer: async_std::io::BufWriter<async_std::fs::File>,
    switch: Arc<AtomicBool>
) {
    let mut log_writer_map = HashMap::<String, BufWriter<File>>::new();

    loop_write(execution_id, msg_queue, &mut default_log_writer, &mut log_writer_map, switch).await;

    let _ = default_log_writer.flush().await;
    for (_,mut v) in log_writer_map {
        let _ = v.flush();
    }
}

async fn loop_write(execution_id: String,
                    msg_queue: Arc<(Mutex<VecDeque<(String, Vec<u8>)>>, Condvar)>,
                    default_log_writer: &mut async_std::io::BufWriter<async_std::fs::File>,
                    log_writer_map: &mut HashMap<String, BufWriter<File>>,
                    switch: Arc<AtomicBool>){
    loop {
        let &(ref lock, ref cvar) = &*msg_queue;
        let mut queue = lock.lock().unwrap();
        while queue.is_empty() {
            if !switch.load(Ordering::SeqCst){
                return;
            }
            queue = cvar.wait(queue).unwrap();
        }

        let (log_path, data) = queue.pop_front().unwrap();

        if log_path.is_empty() {
            let _ = default_log_writer.write_all(&data).await;
            continue;
        }

        let writer = log_writer_map.get_mut(log_path.as_str());
        match writer {
            Some(writer) => {
                let _ = writer.write_all(&data);
            },
            None => {
                let log_file_path = Path::new(&log_path).join(execution_id.as_str()).join("log.log");
                let file = File::create(log_file_path);
                match file {
                    Ok(file) => {
                        let mut writer = BufWriter::new(file);
                        let _ = writer.write_all(&data);
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
    switch: Arc<AtomicBool>
) -> Result<JoinHandle<()>, Error> {

    let sender = Arc::new((Mutex::new(VecDeque::new()), Condvar::new()));
    let receiver = sender.clone();

    log::set_max_level(log::LevelFilter::Trace);
    let _ = log::set_boxed_logger(Box::new(ChannelLogger {
        msg_queue: sender,
        target: Regex::new(&log_target).unwrap(),
    }));

    let file = async_std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&default_log_path).await?;
    let default_log_writer = async_std::io::BufWriter::new(file);

    let jh = thread::spawn(move || block_on(
        log_thread_func(execution_id, receiver, default_log_writer, switch.clone())
    ));

    Ok(jh)
}
