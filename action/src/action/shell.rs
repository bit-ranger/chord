use async_std::io::BufReader;
use async_std::process::{Command, Stdio};
use futures::{AsyncBufReadExt, StreamExt};
use log::trace;

use chord::action::prelude::*;
use chord::err;
use chord::Error;

pub struct ShellFactory {}

impl ShellFactory {
    pub async fn new(_: Option<Value>) -> Result<ShellFactory, Error> {
        Ok(ShellFactory {})
    }
}

#[async_trait]
impl Factory for ShellFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Shell {}))
    }
}

struct Shell {}

#[async_trait]
impl Action for Shell {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let args = arg.args(None)?;
        let code = args["code"].as_str().ok_or(err!("109", "missing code"))?;

        let mut command = Command::new("sh");
        command.arg("-c");
        command.arg(code);

        let child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let std_out = child.stdout.ok_or(err!("107", "missing stdout"))?;
        let std_out = BufReader::new(std_out);
        let mut lines = std_out.lines();
        let mut last_line = String::new();
        loop {
            let line = lines.next().await;
            if line.is_none() {
                let value: Value = from_str(last_line.as_str())?;
                return Ok(Box::new(value));
            }
            let line = line.unwrap()?;
            log_line(&line).await;
            last_line = line;
        }
    }
}

async fn log_line(line: &str) {
    trace!("{}", line)
}
