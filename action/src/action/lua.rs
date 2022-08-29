use rlua::prelude::LuaError;
use rlua::{ToLua, UserData, UserDataMethods};

use chord_core::action::prelude::*;
use chord_core::action::{Context, Id};
use chord_core::future::runtime::Handle;

use crate::err;

pub struct LuaPlayer {}

impl LuaPlayer {
    pub async fn new(_: Option<Value>) -> Result<LuaPlayer, Error> {
        Ok(LuaPlayer {})
    }
}

#[async_trait]
impl Player for LuaPlayer {
    async fn action(&self, _: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(LuaAction {}))
    }
}

struct LuaAction {}

struct ActionUserData {
    id: Box<dyn Id>,
    action: Box<dyn Action>,
    combo: Box<dyn Combo>,
}

#[derive(Clone)]
struct ContextStruct {
    map: Map,
}

impl Context for ContextStruct {
    fn data(&self) -> &Map {
        &self.map
    }

    fn data_mut(&mut self) -> &mut Map {
        &mut self.map
    }

    fn clone(&self) -> Box<dyn Context> {
        let clone = Clone::clone(self);
        Box::new(clone)
    }
}

struct ArgStruct {
    id: Box<dyn Id>,
    args: Value,
    context: ContextStruct,
    combo: Box<dyn Combo>,
}

impl Arg for ArgStruct {
    fn id(&self) -> &dyn Id {
        self.id.as_ref()
    }

    fn args(&self) -> Result<Value, Error> {
        Ok(self.args.clone())
    }

    fn args_raw(&self) -> &Value {
        &self.args
    }

    fn context(&self) -> &dyn Context {
        &self.context
    }

    fn context_mut(&mut self) -> &mut dyn Context {
        &mut self.context
    }

    fn render(&self, _: &dyn Context, raw: &Value) -> Result<Value, Error> {
        Ok(raw.clone())
    }

    fn combo(&self) -> &dyn Combo {
        self.combo.as_ref()
    }

    fn is_static(&self, _: &Value) -> bool {
        true
    }
}

#[async_trait]
impl Action for LuaAction {
    async fn run(&self, arg: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
        let combo = arg.combo().clone();
        let context = arg.context().data().clone();
        let id = arg.id().clone();
        let code = arg
            .args_raw()
            .as_str()
            .ok_or(err!("100", "missing lua"))?
            .to_string();
        execute(id, code, combo, context)
    }
}

fn execute(
    id: Box<dyn Id>,
    code: String,
    combo: Box<dyn Combo>,
    context: Map,
) -> Result<Box<dyn Scope>, Error> {
    let rt = rlua::Lua::new();
    rt.set_memory_limit(Some(1024000));
    rt.context(|lua| {
        let action_fn =
            lua.create_function_mut(move |_c, (name, param): (rlua::String, rlua::Value)| {
                let name = name.to_str().unwrap();
                let id = id.clone();
                let combo = combo.clone();

                let action = combo
                    .player(name)
                    .ok_or(err!("110", "unsupported action"))
                    .map_err(|e| LuaError::RuntimeError(e.to_string()))?;
                let play_arg = ArgStruct {
                    id: id.clone(),
                    combo: combo.clone(),
                    args: to_serde_value(&param)
                        .map_err(|e| LuaError::RuntimeError(e.to_string()))?,
                    context: ContextStruct { map: Map::new() },
                };
                let handle = Handle::current();
                let _ = handle.enter();
                let play = futures::executor::block_on(action.action(&play_arg)).unwrap();

                Ok(ActionUserData {
                    id,
                    action: play,
                    combo,
                })
            })?;

        lua.globals().set("action", action_fn)?;

        for (k, v) in context {
            let v = to_lua_value(lua, &v)?;
            lua.globals().set(k.as_str(), v)?;
        }

        eval(lua, code.as_str())
    })
}

fn eval(lua: rlua::Context, code: &str) -> Result<Box<dyn Scope>, Error> {
    let chunk = lua.load(code);
    let result: rlua::Result<rlua::Value> = chunk.eval();
    match result {
        Ok(v) => {
            let v: Value = to_serde_value(&v)?;
            Ok(Box::new(v))
        }
        Err(e) => Err(err!("101", format!("{}", e))),
    }
}

fn to_serde_value(lua_value: &rlua::Value) -> Result<Value, Error> {
    match lua_value {
        rlua::Value::Nil => Ok(Value::Null),
        rlua::Value::String(v) => Ok(Value::String(v.to_str()?.to_string())),
        rlua::Value::Integer(v) => Ok(Value::Number(Number::from(v.clone()))),
        rlua::Value::Boolean(v) => Ok(Value::Bool(v.clone())),

        rlua::Value::Number(v) => {
            Ok(Number::from_f64(v.clone()).map_or(Value::Null, |v| Value::Number(v)))
        }
        rlua::Value::Table(v) => {
            if is_array(v)? {
                let mut vec = vec![];
                for pair in v.clone().pairs::<usize, rlua::Value>() {
                    let (_, v) = pair?;
                    let v = to_serde_value(&v)?;
                    vec.push(v);
                }
                Ok(Value::Array(vec))
            } else {
                let mut map = Map::new();
                for pair in v.clone().pairs::<String, rlua::Value>() {
                    let (k, v) = pair?;
                    let v = to_serde_value(&v)?;
                    map.insert(k, v);
                }
                Ok(Value::Object(map))
            }
        }

        _ => Err(err!("102", "invalid value")),
    }
}

fn is_array(table: &rlua::Table) -> Result<bool, Error> {
    for pair in table.clone().pairs::<rlua::Value, rlua::Value>() {
        let (k, _) = pair?;
        match k {
            rlua::Value::Integer(_) => return Ok(true),
            _ => continue,
        }
    }
    return Ok(false);
}

fn to_lua_value<'lua>(
    lua_ctx: rlua::Context<'lua>,
    serde_value: &Value,
) -> Result<rlua::Value<'lua>, LuaError> {
    match serde_value {
        Value::Null => Ok(rlua::Value::Nil),
        Value::String(v) => v.as_str().to_lua(lua_ctx),
        Value::Number(v) => Ok(rlua::Value::Number(rlua::Number::from(v.as_f64().unwrap()))),
        Value::Bool(v) => v.to_lua(lua_ctx),
        Value::Object(map) => {
            let table = lua_ctx.create_table()?;
            for (k, v) in map {
                let v = to_lua_value(lua_ctx, &v)?;
                table.set(k.as_str(), v)?;
            }
            Ok(rlua::Value::Table(table))
        }
        Value::Array(vec) => {
            let table = lua_ctx.create_table()?;
            for (k, v) in vec.iter().enumerate() {
                let v = to_lua_value(lua_ctx, &v)?;
                table.set(k + 1, v)?;
            }
            Ok(rlua::Value::Table(table))
        }
    }
}

impl UserData for ActionUserData {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("run", |lua_ctx, this, param: rlua::Value| {
            let mut play_arg = ArgStruct {
                id: this.id.clone(),
                combo: this.combo.clone(),
                args: to_serde_value(&param).map_err(|e| LuaError::RuntimeError(e.to_string()))?,
                context: ContextStruct { map: Map::new() },
            };
            let handle = Handle::current();
            let _ = handle.enter();
            let scope = futures::executor::block_on(this.action.run(&mut play_arg))
                .map_err(|e| LuaError::RuntimeError(e.to_string()))?;
            let value = scope.as_value();
            let lua_value = to_lua_value(lua_ctx, value);
            Ok(lua_value)
        });
    }
}
