use std::mem::replace;

use chord_core::action::prelude::*;
use chord_core::collection::TailDropVec;

use crate::err;

struct ArgStruct<'o, 'c> {
    block: &'o dyn Arg,
    context: &'c mut Box<ContextStruct>,
    aid: String,
    action: String,
}

impl<'o, 'c> Arg for ArgStruct<'o, 'c> {
    fn id(&self) -> &dyn Id {
        self.block.id()
    }

    fn body(&self) -> Result<Value, Error> {
        self.block.render(self.context(), self.body_raw())
    }

    fn body_raw(&self) -> &Value {
        &self.block.body_raw()[&self.aid][&self.action]
    }

    fn init(&self) -> Option<&Value> {
        let raw = self.body_raw();
        if let Value::Object(obj) = raw {
            obj.get("__init__")
        } else {
            None
        }
    }

    fn context(&self) -> &dyn Context {
        self.context.as_ref()
    }

    fn context_mut(&mut self) -> &mut dyn Context {
        self.context.as_mut()
    }

    fn render(&self, context: &dyn Context, raw: &Value) -> Result<Value, Error> {
        self.block.render(context, raw)
    }

    fn combo(&self) -> &dyn Combo {
        self.block.combo()
    }
}

#[derive(Clone)]
struct ContextStruct {
    data: Map,
}

impl Context for ContextStruct {
    fn data(&self) -> &Map {
        &self.data
    }

    fn data_mut(&mut self) -> &mut Map {
        &mut self.data
    }

    fn clone(&self) -> Box<dyn Context> {
        let ctx = Clone::clone(self);
        Box::new(ctx)
    }
}

pub struct BlockCreator {}

impl BlockCreator {
    pub async fn new(_: Option<Value>) -> Result<BlockCreator, Error> {
        Ok(BlockCreator {})
    }
}

#[async_trait]
impl Creator for BlockCreator {
    async fn create(&self, arg: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        let args_raw = arg.body_raw();
        let map = args_raw.as_object().unwrap();
        let mut context = Box::new(ContextStruct {
            data: arg.context().data().clone(),
        });

        let mut action_vec = Vec::with_capacity(map.len());

        for (aid, fo) in map {
            let only = fo.as_object().unwrap().iter().last().unwrap();
            let action = only.0.as_str();

            let mut create_arg = ArgStruct {
                block: arg,
                context: &mut context,
                aid: aid.to_string(),
                action: action.to_string(),
            };

            let action_obj = arg
                .combo()
                .creator(action.into())
                .ok_or_else(|| err!("100", "unsupported action"))?
                .create(&mut create_arg)
                .await
                .map_err(|_| err!("100", "create error"))?;
            action_vec.push((aid.to_string(), action.to_string(), action_obj));
        }

        Ok(Box::new(Block {
            action_vec: TailDropVec::from(action_vec),
        }))
    }
}

struct Block {
    action_vec: TailDropVec<(String, String, Box<dyn Action>)>,
}

#[async_trait]
impl Action for Block {
    async fn execute(&self, arg: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
        let mut context = Box::new(ContextStruct {
            data: arg.context().data().clone(),
        });
        let mut scope_vec = Vec::with_capacity(self.action_vec.len());
        for (aid, action, action_obj) in self.action_vec.iter() {
            let mut run = ArgStruct {
                block: arg,
                context: &mut context,
                aid: aid.to_string(),
                action: action.to_string(),
            };
            let v = action_obj.execute(&mut run).await?;
            scope_vec.push((aid.to_string(), v));
        }

        let _ = replace(arg.context_mut().data_mut(), context.data);

        let scope_vec = TailDropVec::from(scope_vec);
        let mut value = Map::new();
        for (aid, scope) in scope_vec.iter() {
            value.insert(aid.to_string(), scope.as_value().clone());
        }

        Ok(Box::new(Value::Object(value)))
    }
}
