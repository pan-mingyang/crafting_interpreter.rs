use std::vec;

use crate::{bytecode::Chunk, value::Value};



trait DObject {

}
#[derive(Debug, Default, Clone, PartialEq)]
pub enum Object {
    #[default]
    Obj,
    Function(Function),
    List(Vec<Value>),
    String(String),
}


#[derive(Debug, Default, Clone, PartialEq)]
pub struct Function {
    pub arity: i64,
    pub chunk: Chunk,
    pub name: String,
}

impl Function {
    pub fn new(s: String) -> Self {
        Function { arity: 0, chunk: Chunk { code: vec![], lines: vec![] }, name: s }
    }
}


impl Object {
    pub fn to_str(&self) -> String {
        let s = match self {
            Object::Obj => String::from("<Object>"),
            Object::Function(f) => format!("<fn {}>", f.name),
            Object::List(f) => format!("<list> [{:?}]", f),
            Object::String(s) => format!("<string> {}", s),
        };
        s
    }
}




// #[derive(Debug, Default)]
// pub struct DString {
//     obj: Object,
//     chars: Box<char>,
//     length: usize,
// }