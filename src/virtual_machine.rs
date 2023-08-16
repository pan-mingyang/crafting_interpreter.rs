use std::collections::HashMap;
use std::ops::{Add, Sub, Mul, Div, Neg};
use std::path::Components;
use std::vec;
use std::io;

use crate::bytecode::*;
use crate::value::*;
use crate::object::*;

pub enum InterpretError {
    RuntimeError,
    CompileError
}

pub type InterpretResult = Result<(), InterpretError>;
pub type StackElem = Value;
pub type Stack = Vec<StackElem>;

#[derive(Default, Debug)]
pub struct VirtualMachine {
    pub chunk: Chunk,
    pub stack: Stack,
    pub ip: usize,
    pub debug: bool,
    // pub static_table: Vec<dyn DObject>
    // pub panic_mode: bool,
    pub global: HashMap<String, Value>,
    pub constants: Vec<Value>,
}

macro_rules! apply_op_unary {
    ($this:ident, $check:ident, $func:ident) => {{
        $this.$check($this.peek(0), $this.peek(0)); 
        let a = $this.pop();
        $this.push(Value::from(a.$func()));
    }};
}

macro_rules! apply_op {
    ($this:ident, $check:ident, $func:ident) => {{
        $this.$check($this.peek(0), $this.peek(1)); 
        let a = $this.pop();
        let b = $this.pop(); 
        $this.push(Value::from(b.$func(a)));
    }};
}

macro_rules! apply_op_cmp {
    ($this:ident, $func:ident) => {{
        $this.check_number($this.peek(0), $this.peek(1)); 
        let a = &$this.pop();
        let b = &$this.pop(); 
        $this.push(Value::from(b.$func(a)));
    }};
}


impl VirtualMachine {

    pub fn new(chunk: Chunk) -> Self {
        Self { chunk: chunk, stack: Vec::new(), ip: 0, debug: true,
               global: HashMap::new(), constants: vec![] }
    }

    pub fn push(&mut self, s: StackElem) {
        self.stack.push(s);
    }

    pub fn pop(&mut self) -> StackElem {
        self.stack.pop().unwrap()
    }
    
    pub fn top(&self) -> &StackElem {
        &self.stack[self.stack.len()-1]
    }

    pub fn peek(&self, i: usize) -> &StackElem {
        &self.stack[self.stack.len() - 1 - i]
    }

    fn check_number(&self, c1: &Value, c2: &Value) {
        match (c1, c2) {
            (Value::Int(_), Value::Int(_))     |
            (Value::Int(_), Value::Float(_))   |
            (Value::Float(_), Value::Int(_))   |
            (Value::Float(_), Value::Float(_)) => (),
            _ => self.error("The type to be operated shoule be Number")
        }
    }

    fn check_bool(&self, c1: &Value, c2: &Value) {
        match (c1, c2) {
            (Value::Bool(_), Value::Bool(_))=> (),
            _ => self.error("The type to be operated shoule be Boolean")
        }
    }

    pub fn interpret(&mut self) -> InterpretResult {
        self.ip = 0;
        loop {
            if self.ip >= self.chunk.len() {
                return Ok(());
            }
            let ins = &self.chunk.code[self.ip];
            let lineno = &self.chunk.lines[self.ip];
            if self.debug {
                let mut asm = String::new();
                asm += &format!("I{}\t", self.ip.to_string());
                asm += &format!("L{}\t", lineno.to_string());
                asm += &ins.disassemble();
                println!("{}", asm);
            }
            // println!("RUN {}", ins.disassemble());
            let mut next_ip = self.ip + 1;
            match ins {
                ByteCode::Ret => {
                        next_ip = 
                            if let StackElem::Ptr(p) = self.pop() {p}
                            else { return Err(InterpretError::CompileError); };
                    },
                ByteCode::Add => apply_op!(self, check_number, add),
                ByteCode::Sub => apply_op!(self, check_number, sub),
                ByteCode::Mul => apply_op!(self, check_number, mul),
                ByteCode::Div => apply_op!(self, check_number, div),
                ByteCode::And => apply_op!(self, check_bool, bool_and),
                ByteCode::Or  => apply_op!(self, check_bool, bool_or),
                ByteCode::Eq  => apply_op_cmp!(self, eq),
                ByteCode::Ne  => apply_op_cmp!(self, ne),
                ByteCode::Lt  => apply_op_cmp!(self, lt),
                ByteCode::Le  => apply_op_cmp!(self, le),
                ByteCode::Gt  => apply_op_cmp!(self, gt),
                ByteCode::Ge  => apply_op_cmp!(self, ge),                
                ByteCode::Neg => apply_op_unary!(self, check_number, neg),
                ByteCode::Not => apply_op_unary!(self, check_bool, bool_not),

                ByteCode::Out => { println!("[STDOUT] {}", self.top().to_str()); },
                ByteCode::Value(c) => self.push(c.clone()),
                ByteCode::Hlt =>  return Ok(()),
                ByteCode::Pop => {self.pop(); /*self.print_stack();*/},
                ByteCode::J(n) => {
                    next_ip = *n;
                    // println!("GLobal {:?}", self.global);
                    // let mut s = String::new();
                    // io::stdin().read_line(&mut s);
                },
                ByteCode::Nop => (),
                ByteCode::JZ(n) => { 
                    if let Value::Bool(b) = self.peek(0) {
                        if !*b { next_ip = *n; }
                        self.pop();
                    } else {
                        self.error("Expect bool on stack top!");
                    }
                },
                ByteCode::JNZ(n) => { 
                    if let Value::Bool(b) = self.peek(0) {
                        if *b { next_ip = *n; }
                        self.pop();
                    } else {
                        self.error("Expect bool on stack top!");
                    }
                },
                ByteCode::DefGlobal(c) => { 
                    if let Value::String(s) = &self.constants[*c] {
                        let value = self.peek(0);
                        if !self.global.contains_key(s) {
                            self.global.insert(s.clone(), value.clone());
                        } else {
                            self.error(&format!("Variable name '{}' is defined!", s)[..]);
                        }
                    } else {
                        self.error("Error variable name type!");
                    }
                    self.pop();
                },
                ByteCode::Load(c) => { 
                    if let Value::String(s) = &self.constants[*c] {
                        if self.global.contains_key(s) {
                            let value = self.global.get(s).unwrap();
                            self.push(value.clone());
                        }
                        else {
                            self.error(&format!("Variable name '{}' is not defined!", s)[..]);
                        }
                    } else {
                        self.error("Error variable name type!");
                    }
                },
                ByteCode::Set(c) => { 
                    if let Value::String(s) = &self.constants[*c] {
                        if self.global.contains_key(s) {
                            let value = self.peek(0);
                            self.global.insert(s.clone(), value.clone());
                        }
                        else {
                            self.error(&format!("Variable name '{}' is not defined!", s)[..]);
                        }
                    } else {
                        self.error("Error variable name type!");
                    }
                    // self.pop();
                },
                ByteCode::LoadLocal(c) => { 
                    if *c < self.stack.len() {
                        let value = self.stack[*c].clone();
                        self.push(value.clone());
                    } else {
                        self.error("there's no such local variable !");
                    }
                },
                ByteCode::SetLocal(c) => { 
                    let value = self.peek(0);
                    if *c < self.stack.len() {
                        self.stack[*c] = value.clone();
                    } else {
                        self.error("there's no such local variable !");
                    }
                    // self.pop();
                },
                _ => return Ok(()),
            }
            self.ip = next_ip;
            // self.print_stack();
        }
    }

    pub fn print_stack(&self) {
        print!("[ ");
        for i in &self.stack {
            print!("{:?} ", i)
        }
        print!(" ]\n");
    }

    pub fn error(&self, msg: &str) -> ! {
        panic!("Runtime Error: {} at line {}", msg, self.chunk.lines[self.ip])
    }

}