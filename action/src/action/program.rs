use std::time::{Duration, UNIX_EPOCH};

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
        trace!("stdout");
        trace!("{}", std_out);
        trace!("stderr");
        trace!("{}", std_err);

        if !output.status.success() {
            return Err(err!(
                "104",
                format!("program exit with code {}", output.status.to_string())
            ));
        }

        let lines: Vec<&str> = std_out.lines().collect();
        let boundary = args["boundary"].as_str();
        let content = if let Some(b) = boundary {
            let found = lines.iter().enumerate().rfind(|(_i, e)| e.starts_with(b));
            if let Some((i, _e)) = found {
                let start = i + 1;
                if start < lines.len() {
                    (&lines[start..]).to_vec()
                } else {
                    vec![]
                }
            } else {
                lines
            }
        } else {
            lines
        };

        let content_type = args["content_type"].as_str().unwrap_or("text/plain");

        match content_type {
            "application/json" => {
                let tail = content.join("\n");
                let tail_json: Value = from_str(&tail)?;
                return Ok(Asset::Value(tail_json));
            }

            "application/chord-frame-1.0" => {
                let tail = content.join("\n");
                let tail_json: Value = from_str(&tail)?;
                if let Value::Array(vec) = tail_json {
                    let frames: Vec<Box<dyn Frame>> = vec.iter()
                        .enumerate()
                        .map(|(i, v)| value_to_frame(i, v))
                        .collect();
                    return Ok(Asset::Frames(frames));
                }

                return Ok(Asset::Value(tail_json));
            }

            _ => {
                Ok(Asset::Value(Value::String(std_out)))
            }
        }
    }

    async fn explain(&self, _chord: &dyn Chord, arg: &dyn Arg) -> Result<Value, Error> {
        let args = arg.args()?;
        let command = program_command_explain(&args)?;
        Ok(Value::String(command))
    }
}

fn value_to_frame(idx: usize, value: &Value) -> Box<dyn Frame> {
    let frame = ProgramFrame {
        id: value["id"].as_str().map_or(idx.to_string(), |v| v.to_string()),
        start: value["start"].as_u64().map_or(DateTime::<Utc>::from(UNIX_EPOCH), timestamp_to_utc),
        end: value["end"].as_u64().map_or(DateTime::<Utc>::from(UNIX_EPOCH), timestamp_to_utc),
        data: value["data"].clone(),
    };
    Box::new(frame)
}

fn timestamp_to_utc(timestamp: u64) -> DateTime<Utc> {
    let d = UNIX_EPOCH + Duration::from_millis(timestamp);
    DateTime::<Utc>::from(d)
}

#[derive(Serialize)]
struct ProgramFrame {
    id: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    data: Value,
}


impl Data for ProgramFrame {
    fn to_value(&self) -> Value {
        json!({
            "id": self.id,
            "start": self.start,
            "end": self.end,
            "data": self.data
        })
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
