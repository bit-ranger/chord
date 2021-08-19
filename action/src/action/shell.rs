use async_std::io::BufReader;
use async_std::process::{Command, Stdio};
use futures::{AsyncBufReadExt, AsyncWriteExt, StreamExt};
use log::trace;

use async_std::path::PathBuf;
use chord::action::prelude::*;
use chord::err;
use chord::Error;
use std::str::FromStr;

pub struct ShellFactory {
    workdir: PathBuf,
}

impl ShellFactory {
    pub async fn new(config: Option<Value>) -> Result<ShellFactory, Error> {
        if config.is_none() {
            return Err(err!("100", "missing config"));
        }
        let config = config.as_ref().unwrap();

        if config.is_null() {
            return Err(err!("100", "missing config"));
        }

        let workdir = config["workdir"]
            .as_str()
            .ok_or(err!("101", "missing workdir"))?;

        let workdir = PathBuf::from_str(workdir)?;

        async_std::fs::create_dir_all(workdir.as_path()).await?;

        Ok(ShellFactory { workdir })
    }
}

#[async_trait]
impl Factory for ShellFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Shell {
            workdir: self.workdir.clone(),
        }))
    }
}

struct Shell {
    workdir: PathBuf,
}

#[async_trait]
impl Action for Shell {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let args = arg.args(None)?;

        let code = args["code"].as_str().ok_or(err!("109", "missing code"))?;
        let shell_path = self.workdir.join(arg.id().to_string());
        let mut file = async_std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .open(&shell_path)
            .await?;
        file.write_all(code.as_bytes()).await?;
        file.close().await?;
        let perm = std::os::unix::fs::PermissionsExt::from_mode(0o777);
        async_std::fs::set_permissions(shell_path.clone(), perm).await?;
        let mut command = Command::new(shell_path);

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
