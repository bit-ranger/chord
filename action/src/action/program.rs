use async_std::io::BufReader;
use async_std::process::{Child, ChildStdout, Command, Stdio};
use async_std::task::Builder;
use chord::action::prelude::*;
use futures::io::Lines;
use futures::{AsyncBufReadExt, StreamExt};
use log::{debug, info, trace};

pub struct ProgramFactory {}

impl ProgramFactory {
    pub async fn new(_: Option<Value>) -> Result<ProgramFactory, Error> {
        Ok(ProgramFactory {})
    }
}

#[async_trait]
impl Factory for ProgramFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Program {}))
    }
}

struct Program {}

#[async_trait]
impl Action for Program {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let args = arg.args(None)?;

        let cmd = args["program"]
            .as_str()
            .ok_or(err!("103", "missing program"))?;

        let mut command = Command::new(cmd);

        let cmd_args = args["args"].as_array().ok_or(err!("103", "missing args"))?;
        for ca in cmd_args {
            let ca = if ca.is_string() {
                ca.as_str().unwrap().to_owned()
            } else {
                ca.to_string()
            };
            command.arg(ca);
        }

        trace!("command {:?}", command);

        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let std_out = child.stdout.ok_or(err!("107", "missing stdout"))?;
        let std_out = BufReader::new(std_out);
        let mut is_result_line = false;
        let mut result_lines = Vec::new();
        let mut lines = std_out.lines();
        loop {
            let line = lines.next().await;
            if line.is_none() {
                break;
            }
            let line = line.unwrap()?;
            if line == "----program-result----" {
                is_result_line = true;
            }
            if is_result_line {
                result_lines.push(line);
            }
        }

        let result_text: String = result_lines.join("");
        let result: Value = from_str(result_text.as_str())?;
        Ok(Box::new(result))
    }
}
