use std::str::FromStr;

use log::trace;

use chord_core::action::{DateTime, Frame, Utc};
use chord_core::action::prelude::*;
use chord_core::future::process::{Child, Command};

use crate::err;

pub struct ProgramCreator {}

impl ProgramCreator {
    pub async fn new(_: Option<Value>) -> Result<ProgramCreator, Error> {
        Ok(ProgramCreator {})
    }
}

#[async_trait]
impl Creator for ProgramCreator {
    async fn create(&self, _chord: &dyn Chord, arg: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        let args_raw = arg.args_raw();
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
    async fn execute(
        &self,
        _chord: &dyn Chord,
        arg: &mut dyn Arg,
    ) -> Result<Asset, Error> {
        let args = arg.args()?;
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
        let lines: Vec<&str> = out.lines().collect();

        let parse_last_rows_count = args["parse_last_rows_count"].as_u64().unwrap_or(0);
        if parse_last_rows_count < 1 {
            Ok(Asset::Value(Value::String(out)))
        } else {
            let begin = if lines.len() as u64 - parse_last_rows_count > 0 {
                (lines.len() as u64 - parse_last_rows_count) as usize
            } else {
                0
            };

            let tail = lines[begin..lines.len()].join("\n");
            let tail_json: Value = from_str(&tail)?;
            if let Value::Object(map) = &tail_json {
                if map.get("chord_report_frames").is_some() {
                    let frames = map.get("frames");
                    if let Some(frames) = frames {
                        if let Value::Array(vec) = frames {
                            let frames: Vec<Box<dyn Frame>> = vec.iter().map(value_to_frame).collect();
                            return Ok(Asset::Frames(frames));
                        }
                    }
                }
                return Ok(Asset::Value(tail_json));
            } else {
                return Ok(Asset::Value(tail_json));
            }
        }
    }

    async fn explain(&self, _chord: &dyn Chord, arg: &dyn Arg) -> Result<Value, Error> {
        let args = arg.args()?;
        let command = program_command_explain(&args)?;
        Ok(Value::String(command))
    }
}

fn value_to_frame(value: &Value) -> Box<dyn Frame> {
    let frame = ProgramFrame {
        id: value["id"].as_str().unwrap_or("").to_string(),
        start: value["start"].as_str().map_or(Utc::now(), |t| DateTime::from_str(t).unwrap_or(Utc::now())),
        end: value["end"].as_str().map_or(Utc::now(), |t| DateTime::from_str(t).unwrap_or(Utc::now())),
        data: value["data"].clone(),
    };
    Box::new(frame)
}

struct ProgramFrame {
    id: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    data: Value,
}


impl Data for ProgramFrame {
    fn to_value(&self) -> Value {
        self.data.clone()
    }
}

impl Frame for ProgramFrame {
    fn id(&self) -> &str {
        self.id.as_str()
    }

    fn start(&self) -> DateTime<Utc> {
        self.start
    }

    fn end(&self) -> DateTime<Utc> {
        self.end
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
    async fn execute(
        &self,
        _chord: &dyn Chord,
        arg: &mut dyn Arg,
    ) -> Result<Asset, Error> {
        let args = arg.args()?;

        let mut command = program_command(&args)?;
        trace!("detach command {:?}", command);
        let child = command.spawn()?;
        trace!("detach pid {:?}", child.id());
        Ok(Asset::Data(Box::new(ChildHolder::new(child))))
    }

    async fn explain(&self, _chord: &dyn Chord, arg: &dyn Arg) -> Result<Value, Error> {
        let args = arg.args()?;
        let command = program_command_explain(&args)?;
        Ok(Value::String(command))
    }
}

struct ChildHolder {
    child: Child,
}

impl ChildHolder {
    fn new(child: Child) -> ChildHolder {
        ChildHolder {
            child,
        }
    }
}

impl Data for ChildHolder {
    fn to_value(&self) -> Value {
        Value::Number(Number::from(self.child.id().unwrap()))
    }
}

impl Drop for ChildHolder {
    fn drop(&mut self) {
        let _ = self.child.kill();
        trace!("kill pid {:?}", self.child.id());
    }
}

fn program_command(args: &Value) -> Result<Command, Error> {
    let cmd_vec = args["cmd"].as_array().ok_or(err!("101", "missing cmd"))?;
    if cmd_vec.len() < 1 {
        return Err(err!("101", "missing cmd"));
    }
    let mut command = Command::new(cmd_vec[0].as_str().ok_or(err!("102", "invalid cmd"))?);

    for ca in &cmd_vec[1..] {
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
    let cmd_vec = args["cmd"].as_array().ok_or(err!("101", "missing cmd"))?;
    if cmd_vec.len() < 1 {
        return Err(err!("101", "missing cmd"));
    }

    let mut command = String::new();

    for ca in cmd_vec {
        let ca = if ca.is_string() {
            ca.as_str().unwrap().to_owned()
        } else {
            ca.to_string()
        };
        command.push_str(ca.as_str());
        command.push_str(" ");
    }
    Ok(command)
}
