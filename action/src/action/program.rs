use async_std::process::{Child, Command};
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
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let args_raw = Value::Object(arg.args_raw().clone());
        match args_raw["detach"].as_bool().unwrap_or(false) {
            true => Ok(Box::new(DetachProgram::new(&args_raw)?)),
            false => Ok(Box::new(AttachProgram::new(&args_raw)?)),
        }
    }
}

struct AttachProgram {}

impl AttachProgram {
    fn new(_: &Value) -> Result<AttachProgram, Error> {
        Ok(AttachProgram {})
    }
}

#[async_trait]
impl Action for AttachProgram {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let args = Value::Object(arg.args()?);
        let mut command = program_command(&args)?;
        trace!("program attach command {:?}", command);
        let output = command.output().await?;

        let std_out = String::from_utf8_lossy(&output.stdout).to_string();
        let std_err = String::from_utf8_lossy(&output.stderr).to_string();
        trace!("stdout:\n{}", std_out);
        trace!("stderr:\n{}", std_err);

        if !output.status.success() {
            return Err(err!(
                "104",
                format!("program exit with code {}", output.status.to_string())
            ));
        }

        let out = format!("{}{}", std_out, std_err);
        let last_line = out.lines().last();

        match last_line {
            None => Ok(Box::new(Value::Null)),
            Some(last_line) => {
                let value_to_json = args["value_to_json"].as_bool().unwrap_or(false);
                if value_to_json {
                    let value: Value = from_str(last_line)?;
                    Ok(Box::new(value))
                } else {
                    let value: Value = Value::String(last_line.to_string());
                    Ok(Box::new(value))
                }
            }
        }
    }

    async fn explain(&self, arg: &dyn RunArg) -> Result<Value, Error> {
        let args = Value::Object(arg.args()?);
        let command = program_command_explain(&args)?;
        Ok(Value::String(command))
    }
}

struct DetachProgram {}

impl DetachProgram {
    fn new(_: &Value) -> Result<DetachProgram, Error> {
        Ok(DetachProgram {})
    }
}

#[async_trait]
impl Action for DetachProgram {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let args = Value::Object(arg.args()?);

        let mut command = program_command(&args)?;
        trace!("detach command {:?}", command);
        let child = command.spawn()?;
        trace!("detach pid {:?}", child.id());
        Ok(Box::new(ChildHolder::new(child)))
    }

    async fn explain(&self, arg: &dyn RunArg) -> Result<Value, Error> {
        let args = Value::Object(arg.args()?);
        let command = program_command_explain(&args)?;
        Ok(Value::String(command))
    }
}

struct ChildHolder {
    value: Value,
    child: Child,
}

impl ChildHolder {
    fn new(child: Child) -> ChildHolder {
        ChildHolder {
            value: Value::Number(Number::from(child.id())),
            child,
        }
    }
}

impl Scope for ChildHolder {
    fn as_value(&self) -> &Value {
        &self.value
    }
}

impl Drop for ChildHolder {
    fn drop(&mut self) {
        let _ = self.child.kill();
        trace!("kill pid {:?}", self.child.id());
    }
}

fn program_command(args: &Value) -> Result<Command, Error> {
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
    Ok(command)
}

fn program_command_explain(args: &Value) -> Result<String, Error> {
    let cmd = args["program"]
        .as_str()
        .ok_or(err!("103", "missing program"))?;

    let mut command = String::from(cmd);

    let cmd_args = args["args"].as_array().ok_or(err!("103", "missing args"))?;

    for ca in cmd_args {
        let ca = if ca.is_string() {
            ca.as_str().unwrap().to_owned()
        } else {
            ca.to_string()
        };
        command.push_str(" ");
        command.push_str(ca.as_str())
    }
    Ok(command)
}
