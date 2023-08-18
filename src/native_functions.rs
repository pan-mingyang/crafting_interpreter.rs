use crate::{value::Value, object::Object};


pub struct Native;

impl Native {
    pub fn list(objs: &mut Vec<Object>, argc: usize, args: Vec<Value>) -> Value {
        let mut list = Vec::new();
        for i in 0..=argc {
            list.push(args[i].clone());
        }
        objs.push(Object::List(list));
        Value::Obj(objs.len() - 1)
    }
}

