use rlua::{AnyUserData, UserData, UserDataMethods};

use chord_core::action::prelude::*;
use chord_core::action::{Context, Id};
use chord_core::future::runtime::Handle;

use crate::action::lua;
use crate::err;

pub struct LuaAction {}

impl LuaAction {
    pub async fn new(_: Option<Value>) -> Result<LuaAction, Error> {
        Ok(LuaAction {})
    }
}

#[async_trait]
impl Action for LuaAction {
    async fn play(&self, _: &dyn Arg) -> Result<Box<dyn Play>, Error> {
        Ok(Box::new(LuaPlay {}))
    }
}

struct LuaPlay {}

struct PlayUserData {
    action: Box<dyn Play>,
}

impl UserData for PlayUserData {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("execute", |_, this, pa: ()| {
            println!("execute");
            Ok(())
        });
    }
}

struct IdStruct {}

struct ContextStruct {}

struct ArgStruct {}

impl Arg for ArgStruct {
    fn id(&self) -> &dyn Id {
        todo!()
    }

    fn args(&self) -> Result<Value, Error> {
        todo!()
    }

    fn args_raw(&self) -> &Value {
        todo!()
    }

    fn context(&self) -> &dyn Context {
        todo!()
    }

    fn context_mut(&mut self) -> &mut dyn Context {
        todo!()
    }

    fn render(&self, context: &dyn Context, raw: &Value) -> Result<Value, Error> {
        todo!()
    }

    fn combo(&self) -> &dyn Combo {
        todo!()
    }

    fn is_static(&self, raw: &Value) -> bool {
        true
    }
}

#[async_trait]
impl Play for LuaPlay {
    async fn execute(&self, arg: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
        let rt = rlua::Lua::new();
        rt.set_memory_limit(Some(1024000));
        rt.context(|lua| {
            let args = arg.args()?;
            let code = args.as_str().ok_or(err!("100", "missing lua"))?;

            for (k, v) in arg.context().data() {
                let v = rlua_serde::to_value(lua, v)?;
                lua.globals().set(k.as_str(), v)?;
            }

            let combo = arg.combo().clone();
            let action =
                lua.create_function_mut(move |c, (name, param): (rlua::String, rlua::Value)| {
                    let action = combo.action(name.to_str().unwrap()).unwrap();
                    let play_arg = ArgStruct {};

                    let handle = Handle::current();
                    let action = handle.block_on(action.play(&play_arg)).unwrap();

                    Ok(PlayUserData { action })
                })?;

            lua.globals().set("action", action);

            self.eval(lua, code.to_string())
        })
    }
}

impl LuaPlay {
    fn eval(&self, lua: rlua::Context, code: String) -> Result<Box<dyn Scope>, Error> {
        match lua.load(code.as_str()).eval::<rlua::Value>() {
            Ok(v) => {
                let v: Value = to_value(&v)?;
                Ok(Box::new(v))
            }
            Err(e) => Err(err!("101", format!("{}", e))),
        }
    }
}

fn to_value(lua_value: &rlua::Value) -> Result<Value, Error> {
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
                    let v = to_value(&v)?;
                    vec.push(v);
                }
                Ok(Value::Array(vec))
            } else {
                let mut map = Map::new();
                for pair in v.clone().pairs::<String, rlua::Value>() {
                    let (k, v) = pair?;
                    let v = to_value(&v)?;
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
