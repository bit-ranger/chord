use log;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use std::collections::vec_deque::VecDeque;
use std::fs::{remove_file, rename, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::vec::Vec;
use time::{at, get_time, strftime};
use regex::Regex;

struct ChannelLogger {
    level: Level,
    target: Regex,
    msg_queue: Arc<(Mutex<VecDeque<Vec<u8>>>, Condvar)>,
}

impl log::Log for ChannelLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level && self.target.is_match(metadata.target())
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

            let &(ref lock, ref cvar) = &*self.msg_queue;
            let mut queue = lock.lock().unwrap();
            queue.push_back(data);
            cvar.notify_one();
        }
    }

    fn flush(&self) {}
}

fn log_thread_func(
    msg_queue: Arc<(Mutex<VecDeque<Vec<u8>>>, Condvar)>,
    log_path: String,
    rotate_count: usize,
    rotate_size: usize,
    console_print: bool
) {
    let mut size = 0;
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path);

    loop {
        let &(ref lock, ref cvar) = &*msg_queue;
        let mut queue = lock.lock().unwrap();
        while queue.is_empty() {
            queue = cvar.wait(queue).unwrap();
        }

        let data = queue.pop_front().unwrap();
        if console_print {
            println!("{}", String::from_utf8(data.clone()).unwrap());
        }
        match file {
            Ok(ref mut f) => {
                let _ = f.write_all(&data);
                size += data.len();
            }
            Err(_) => {}
        }

        if size > rotate_size && rotate_count > 0 {
            rotate_file(&log_path, rotate_count);
            file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path);
            size = 0;
        }
    }
}

fn get_rotate_name(log_path: &String, num: usize) -> String {
    let mut path = log_path.clone();

    if num > 0 {
        path.push('.');
        path.push_str(&num.to_string());
    }

    path
}

fn rotate_file(log_path: &String, rotate_count: usize) {
    let mut rotate_num = rotate_count - 1;
    let _ = remove_file(get_rotate_name(log_path, rotate_num));

    while rotate_num > 0 {
        let to = get_rotate_name(log_path, rotate_num);
        let from = get_rotate_name(log_path, rotate_num - 1);
        let _ = rename(from, to);
        rotate_num -= 1;
    }
}

pub fn init(
    level: Level,
    log_target: String,
    log_path: String,
    rotate_count: usize,
    rotate_size: usize,
    console_print: bool
) -> Result<(), SetLoggerError> {
    let sender = Arc::new((Mutex::new(VecDeque::new()), Condvar::new()));
    let receiver = sender.clone();

    thread::spawn(move || {
        log_thread_func(receiver, log_path, rotate_count, rotate_size, console_print);
    });

    log::set_max_level(LevelFilter::Info);
    log::set_boxed_logger(Box::new(ChannelLogger {
        level,
        target: Regex::new(&log_target).unwrap(),
        msg_queue: sender,
    }))
}
