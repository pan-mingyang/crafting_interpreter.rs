use std::collections::HashMap;
use std::ops::{Add, Sub, Mul, Div, Neg, Rem, Shl, BitAnd, BitXor, BitOr, Shr};
use std::path::Components;
use std::vec;
use std::io;

use crate::bytecode::*;
use crate::parser::Parser;
use crate::value::*;
use crate::object::*;


#[derive(Default, Debug)]
pub struct CallFrame {
    pub func_id: usize,
    pub ip: usize,
    pub slot_index: usize,
}


pub enum InterpretError {
    RuntimeError,
    CompileError
}

pub type InterpretResult = Result<(), InterpretError>;
pub type StackElem = Value;
pub type Stack = Vec<StackElem>;

#[derive(Default, Debug)]
pub struct VirtualMachine {
    pub functions: Vec<Function>,
    pub frames: Vec<CallFrame>,
    pub stack: Stack,
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
        let value = Value::from(a.$func());
        if let Value::Nil = value {
            $this.error("Wrong object type for the operator !");
        }
        $this.push(value);
    }};
}

macro_rules! apply_op {
    ($this:ident, $check:ident, $func:ident) => {{
        $this.$check($this.peek(0), $this.peek(1)); 
        let a = $this.pop();
        let b = $this.pop(); 
        let value = Value::from(b.$func(a));
        if let Value::Nil = value {
            $this.error("Wrong object type for the operator !");
        }
        $this.push(value);
    }};
}

macro_rules! apply_op_cmp {
    ($this:ident, $func:ident) => {{
        $this.check_number($this.peek(0), $this.peek(1)); 
        let a = &$this.pop();
        let b = &$this.pop();
        let value = Value::from(b.$func(a));
        if let Value::Nil = value {
            $this.error("Wrong object type for the operator!");
        }
        $this.push(value);
    }};
}


impl VirtualMachine {

    pub fn from_parser(parser: &Parser) -> Self {
        let frame = CallFrame {
            func_id: 0,
            ip: 0,
            slot_index: 0,
        };
        Self { stack: Vec::new(), debug: true,
               global: HashMap::new(), constants: vec![] , 
               functions: parser.functions.clone(), frames: vec![frame] }
    }

    pub fn push(&mut self, s: StackElem) {
        self.stack.push(s);
        self.print_stack();
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
            _ => {
                self.print_stack(); self.error("The type to be operated shoule be Number")
            }
        }
    }

    fn check_bool(&self, c1: &Value, c2: &Value) {
        match (c1, c2) {
            (Value::Bool(_), Value::Bool(_))=> (),
            _ => self.error("The type to be operated shoule be Boolean")
        }
    }

    fn current_chunk(&self) -> &Chunk {
        let id = self.frames.last()
                .unwrap_or_else(|| self.error("Frame empty error")).func_id;
        &self.functions[id].chunk
    }

    fn get_ip(&self) -> usize {
        self.frames.last().unwrap_or_else(|| self.error("Frame id overflow error")).ip        
    }

    fn set_ip(&mut self, ip: usize) {
        let end_idx = self.frames.len() - 1;
        self.frames[end_idx].ip = ip;
    }

    fn current_frame(&mut self) -> &mut CallFrame {
        let end_idx = self.frames.len() - 1;
        &mut self.frames[end_idx]
    }

    fn get_frame(&self) -> &CallFrame {
        let end_idx = self.frames.len() - 1;
        &self.frames[end_idx]
    }


    pub fn interpret(&mut self) -> InterpretResult {
        loop {
            if self.get_ip() >= self.current_chunk().len() {
                return Ok(());
            }
            let ins = &self.current_chunk().code[self.get_ip()];
            let lineno = &self.current_chunk().lines[self.get_ip()];
            if self.debug {
                let mut asm = String::new();
                asm += &format!("I{}\t", self.get_ip().to_string());
                asm += &format!("L{}\t", lineno.to_string());
                asm += &ins.disassemble();
                println!("{}", asm);
            }
            // println!("RUN {}", ins.disassemble());
            let mut next_ip = self.get_ip() + 1;
            // ins.disassemble();
            match ins {
                ByteCode::Ret => {
                        next_ip = 
                            if let StackElem::Ptr(p) = self.pop() {p}
                            else { return Err(InterpretError::CompileError); };
                    },
                ByteCode::Add  => apply_op!(self, check_number, add),
                ByteCode::Sub  => apply_op!(self, check_number, sub),
                ByteCode::Mul  => apply_op!(self, check_number, mul),
                ByteCode::Div  => apply_op!(self, check_number, div),
                ByteCode::Mod  => apply_op!(self, check_number, rem),
                ByteCode::Shl  => apply_op!(self, check_number, shl),
                ByteCode::Shr  => apply_op!(self, check_number, shr),
                ByteCode::LAnd => apply_op!(self, check_number, bitand),
                ByteCode::LOr  => apply_op!(self, check_number, bitor),
                ByteCode::LXor => apply_op!(self, check_number, bitxor),
                ByteCode::And  => apply_op!(self, check_bool, bool_and),
                ByteCode::Or   => apply_op!(self, check_bool, bool_or),
                ByteCode::Eq   => apply_op_cmp!(self, eq),
                ByteCode::Ne   => apply_op_cmp!(self, ne),
                ByteCode::Lt   => apply_op_cmp!(self, lt),
                ByteCode::Le   => apply_op_cmp!(self, le),
                ByteCode::Gt   => apply_op_cmp!(self, gt),
                ByteCode::Ge   => apply_op_cmp!(self, ge),                
                ByteCode::Neg  => apply_op_unary!(self, check_number, neg),
                ByteCode::Not  => apply_op_unary!(self, check_bool, bool_not),
                ByteCode::LNot => apply_op_unary!(self, check_number, bitnot),

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
                    let local_index = *c + self.get_frame().slot_index;
                    if local_index < self.stack.len() {
                        let value = self.stack[local_index].clone();
                        self.push(value.clone());
                    } else {
                        self.error("there's no such local variable !");
                    }
                },
                ByteCode::SetLocal(c) => {
                    let local_index = *c + self.get_frame().slot_index;
                    let value = self.peek(0);
                    if local_index < self.stack.len() {
                        self.stack[local_index] = value.clone();
                    } else {
                        self.error("there's no such local variable !");
                    }
                    // self.pop();
                },
                _ => return Ok(()),
            }
            self.set_ip(next_ip);
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
        panic!("Runtime Error: {} at line {}", msg, self.current_chunk().lines[self.get_ip()])
    }
}