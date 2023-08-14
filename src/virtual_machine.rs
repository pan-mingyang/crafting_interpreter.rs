use std::{default, ops::{Add, Sub, Mul, Div, Neg}};

use crate::bytecode::*;

pub enum InterpretError {
    RuntimeError,
    CompileError
}

pub type InterpretResult = Result<(), InterpretError>;

/*

#[derive(Default, Debug)]
pub enum StackElem {
    #[default]
    Nil,
    Constant(Constant),
    Ptr(usize)
}

macro_rules! impl_binary_op_for_stack_elem {
    ($clz:ident, $op:ident) => {
        impl $clz for StackElem {
            type Output = StackElem;
        
            fn $op(self, rhs: Self) -> Self::Output {
                if let StackElem::Constant(c1) = self {
                    if let StackElem::Constant(c2) = rhs {
                        return StackElem::Constant(c1.$op(c2));
                    }
                }
                StackElem::Nil
            }
        }
        
    };
}
impl_binary_op_for_stack_elem!(Add, add);
impl_binary_op_for_stack_elem!(Sub, sub);
impl_binary_op_for_stack_elem!(Mul, mul);
impl_binary_op_for_stack_elem!(Div, div);


impl StackElem {
    pub fn to_str(&self) -> String {
        match self {
            StackElem::Nil => String::from("Nil"),
            StackElem::Constant(c) => c.to_str(),
            StackElem::Ptr(c) => c.to_string(),
        }
    }
}
*/

pub type StackElem = Constant;
pub type Stack = Vec<StackElem>;

#[derive(Default, Debug)]
pub struct VirtualMachine {
    pub chunk: Chunk,
    pub stack: Stack,
    pub ip: usize,
    pub debug: bool,
}

macro_rules! apply_op_unary {
    ($this:ident, $check:ident, $func:ident) => {{
        $this.$check($this.peek(0), $this.peek(0)); 
        let a = $this.pop();
        $this.push(Constant::from(a.$func()));
    }};
}

macro_rules! apply_op {
    ($this:ident, $check:ident, $func:ident) => {{
        $this.$check($this.peek(0), $this.peek(1)); 
        let a = $this.pop();
        let b = $this.pop(); 
        $this.push(Constant::from(a.$func(b)));
    }};
}

macro_rules! apply_op_cmp {
    ($this:ident, $func:ident) => {{
        $this.check_number($this.peek(0), $this.peek(1)); 
        let a = $this.pop();
        let b = &$this.pop(); 
        $this.push(Constant::from(a.$func(b)));
    }};
}


impl VirtualMachine {

    pub fn new(chunk: Chunk) -> Self {
        Self { chunk: chunk, stack: Vec::new(), ip: 0, debug: true }
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

    fn check_number(&self, c1: &Constant, c2: &Constant) {
        match (c1, c2) {
            (Constant::Int(_), Constant::Int(_))     |
            (Constant::Int(_), Constant::Float(_))   |
            (Constant::Float(_), Constant::Int(_))   |
            (Constant::Float(_), Constant::Float(_)) => (),
            _ => self.error("The type to be operated shoule be Number")
        }
    }

    fn check_bool(&self, c1: &Constant, c2: &Constant) {
        match (c1, c2) {
            (Constant::Bool(_), Constant::Bool(_))=> (),
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
            match ins {
                ByteCode::Ret => {
                        self.ip = 
                            if let StackElem::Ptr(p) = self.pop() {p}
                            else { return Err(InterpretError::CompileError); } ;
                        continue;
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
                ByteCode::Out => println!("{}", self.top().to_str()),
                ByteCode::Constant(c) => self.push(c.clone()),
                ByteCode::Hlt =>  return Ok(()),
                _ => return Ok(()),
            }
            self.ip += 1;
        }
    }


    pub fn error(&self, msg: &str) -> ! {
        panic!("Runtime Error: {} at line {}", msg, self.chunk.lines[self.ip])
    }

}

/*
ByteCode::Add => { self.check_number(self.peek(0), self.peek(1)); let a = self.pop(); let b = self.pop(); self.push(a + b)},
ByteCode::Sub => { self.check_number(self.peek(0), self.peek(1)); let a = self.pop(); let b = self.pop(); self.push(a - b)},
ByteCode::Mul => { self.check_number(self.peek(0), self.peek(1)); let a = self.pop(); let b = self.pop(); self.push(a * b)},
ByteCode::Div => { self.check_number(self.peek(0), self.peek(1)); let a = self.pop(); let b = self.pop(); self.push(a / b)},
ByteCode::And => { self.check_bool(self.peek(0), self.peek(1)); let a = self.pop(); let b = self.pop(); self.push(a.bool_and(b))},
ByteCode::Or  => { self.check_bool(self.peek(0), self.peek(1)); let a = self.pop(); let b = self.pop(); self.push(a.bool_or(b))},
ByteCode::Neg => { self.check_number(self.peek(0), self.peek(0)); let a = self.pop(); self.push(-a)},
ByteCode::Not => { self.check_bool(self.peek(0), self.peek(0)); let a = self.pop(); self.push(a.bool_not())},

ByteCode::Eq  => { self.check_number(self.peek(0), self.peek(1)); let a = self.pop(); let b = self.pop(); self.push(Constant::Bool(a == b))},
ByteCode::Ne  => { self.check_number(self.peek(0), self.peek(1)); let a = self.pop(); let b = self.pop(); self.push(Constant::Bool(a != b))},
ByteCode::Lt  => { self.check_number(self.peek(0), self.peek(1)); let a = self.pop(); let b = self.pop(); self.push(Constant::Bool(a <  b))},
ByteCode::Le  => { self.check_number(self.peek(0), self.peek(1)); let a = self.pop(); let b = self.pop(); self.push(Constant::Bool(a <= b))},
ByteCode::Gt  => { self.check_number(self.peek(0), self.peek(1)); let a = self.pop(); let b = self.pop(); self.push(Constant::Bool(a >  b))},
ByteCode::Ge  => { self.check_number(self.peek(0), self.peek(1)); let a = self.pop(); let b = self.pop(); self.push(Constant::Bool(a >= b))},

*/
