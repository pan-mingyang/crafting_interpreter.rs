use crate::{object::Object, value::Value};


pub trait ToObject {
    fn to_object(&self, obj_list: &mut Vec<Object>) -> Value;
}

impl ToObject for String {
    fn to_object(&self, obj_list: &mut Vec<Object>) -> Value {
        obj_list.push(Object::String(self.clone()));
        Value::Obj(obj_list.len() - 1)
    }
}

impl ToObject for Vec<Value> {
    fn to_object(&self, obj_list: &mut Vec<Object>) -> Value {
        obj_list.push(Object::List(self.clone()));
        Value::Obj(obj_list.len() - 1)
    }
}






