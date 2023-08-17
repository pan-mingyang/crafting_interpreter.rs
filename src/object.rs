use std::vec;

use crate::bytecode::Chunk;



trait DObject {

}
#[derive(Debug, Default, Clone, PartialEq)]
pub enum Object {
    #[default]
    Obj,
    Function(Function),
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



// #[derive(Debug, Default)]
// pub struct DString {
//     obj: Object,
//     chars: Box<char>,
//     length: usize,
// }