use async_std::process::Command;
use chord::action::prelude::*;
use log::trace;

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

        let output = command.output().await?;

        let std_out = String::from_utf8_lossy(&output.stdout).to_string();
        let std_err = String::from_utf8_lossy(&output.stderr).to_string();
        trace!("{}", std_out);
        trace!("{}", std_err);

        if !output.status.success() {
            return Err(err!(
                "104",
                format!("program exit with code {}", output.status.to_string())
            ));
        }

        let out = format!("{}{}", std_out, std_err);
        if args["value_as_json"].as_bool().unwrap_or(true) {
            let value: Value = from_str(out.as_str())?;
            Ok(Box::new(value))
        } else {
            let value: Value = Value::String(out);
            Ok(Box::new(value))
        }
    }
}
