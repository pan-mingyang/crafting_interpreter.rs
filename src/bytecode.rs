use std::{ops::{DerefMut, Deref}, fs::File, io::BufReader, cmp::Ordering};
use std::io::prelude::*;

use crate::{value::*, object::Object};

#[derive(Default, Debug, PartialEq, Clone)]
pub enum ByteCode {    
    #[default]
    Hlt,
    Ret,
    Out,
    Value(Value),
    Add, Sub, Mul, Div, Neg, Mod,
    Shr, Shl, LAnd, LOr, LXor, LNot,
    True, False,
    Nil,
    And, Or, Not,
    Eq, Ne, Lt, Le, Gt, Ge,
    Pop, Push(Value),
    DefGlobal(usize),
    Load(usize),
    Set(usize),
    LoadLocal(usize),
    SetLocal(usize),
    JZ(usize),
    JNZ(usize),
    J(usize),
    Nop,
    Call(usize),
}


impl ByteCode {

    pub fn disassemble(&self) -> String {
        match self {
            ByteCode::Ret => String::from("ret"),
            ByteCode::Hlt => String::from("hlt"),
            ByteCode::Out => String::from("out"),
            ByteCode::Add => String::from("add"),
            ByteCode::Sub => String::from("sub"),
            ByteCode::Mul => String::from("mul"),
            ByteCode::Div => String::from("div"),
            ByteCode::And => String::from("and"),
            ByteCode::Or  => String::from("or"),
            ByteCode::Not => String::from("not"),
            ByteCode::Eq  => String::from("eq"),
            ByteCode::Ne  => String::from("ne"),
            ByteCode::Le  => String::from("le"),
            ByteCode::Lt => String::from("lt"),
            ByteCode::Ge  => String::from("gt"),
            ByteCode::Gt  => String::from("ge"),
            ByteCode::Pop  => String::from("pop"),
            ByteCode::Nop  => String::from("nop"),
            ByteCode::Mod  => String::from("mod"),
            ByteCode::Shl  => String::from("shl"),
            ByteCode::Shr  => String::from("shr"),
            ByteCode::LAnd  => String::from("land"),
            ByteCode::LOr  => String::from("lor"),
            ByteCode::LXor  => String::from("lxor"),
            ByteCode::LNot  => String::from("lnot"),
            ByteCode::Push(c)  => String::from("push\t") + &c.to_str(),
            ByteCode::Value(c) => String::from("const\t") + &c.to_str(),
            ByteCode::DefGlobal(c) => String::from("def_global\t") + &c.to_string(),
            ByteCode::Load(c) => String::from("load\t") + &c.to_string(),
            ByteCode::Set(c) => String::from("set\t") + &c.to_string(),
            ByteCode::LoadLocal(c) => String::from("load_local\t") + &c.to_string(),
            ByteCode::SetLocal(c) => String::from("set_local\t") + &c.to_string(),
            ByteCode::JZ(c) => String::from("jz\t") + &c.to_string(),
            ByteCode::JNZ(c) => String::from("jnz\t") + &c.to_string(),
            ByteCode::J(c) => String::from("j\t") + &c.to_string(),
            ByteCode::Call(c) => String::from("call\t") + &c.to_string(),
            _ => String::from("[UNK]")
        }
    }


    pub fn disassemble_detail(&self, obj_list: &Vec<Object>) -> String {
        match self {
            ByteCode::Value(Value::Obj(c)) => String::from("const\t") + obj_list[*c].to_str().as_str(),
            _ => self.disassemble(),
        }
    }
    
}

impl From<f64> for ByteCode {
    fn from(value: f64) -> Self {
        Self::Value(Value::Float(value))
    }
}

impl From<i64> for ByteCode {
    fn from(value: i64) -> Self {
        Self::Value(Value::Int(value))
    }
}

impl From<bool> for ByteCode {
    fn from(value: bool) -> Self {
        Self::Value(Value::Bool(value))
    }
}

// impl From<String> for ByteCode {
//     fn from(value: String) -> Self {
//         Self::Value(Value::String(value))
//     }
// }



#[derive(Default, Debug, Clone, PartialEq)]
pub struct Chunk {
    pub code: Vec<ByteCode>,
    pub lines: Vec<usize>,
    // pub ip: u64,
}


impl Chunk {
    pub fn new() -> Self {
        Self { code: Vec::new(), lines: Vec::new() }
    }

    pub fn disassemble(&self) -> String {
        let mut asm = String::new();
        for (idx, (ins, lineno)) 
            in self.code.iter()
                        .zip(&self.lines)
                        .enumerate()
        {
            asm += &format!("I{}\t{}\t{}\n", idx.to_string(), lineno.to_string(), ins.disassemble());
        }
        asm
    }

    pub fn add(&mut self, ins: ByteCode, lineno: usize) {
        self.push(ins);
        self.lines.push(lineno);
    }

    pub fn from_file(filename: &str) -> Self {
        let f = File::open(filename).unwrap();
        let mut chunk = Self::default();
        let reader = BufReader::new(f);
        for line in reader.lines() {
            let line = line.unwrap();
            if line.contains("\t") {
                // println!("[{}]", line);
                let mut sp = line.split("\t");
                let ins = sp.next().unwrap();
                let number = sp.next().unwrap();
                if ins == "C" {
                    if let Ok(x) = number.parse::<i64>() {
                        chunk.add(ByteCode::Value(Value::Int(x)), 0);
                    } else if let Ok(x) = number.parse::<f64>() {
                        chunk.add(ByteCode::Value(Value::Float(x)), 0);
                    } else if let Ok(x) = number.parse::<bool>() {
                        chunk.add(ByteCode::Value(Value::Bool(x)), 0);
                    } else if number.starts_with("P_") {
                        let x = number[2..].parse::<usize>().unwrap();
                        chunk.add(ByteCode::Value(Value::Ptr(x)), 0);
                    }
                }
            } else {
                let ins = line.as_str();
                match ins {
                    "ADD" => chunk.add(ByteCode::Add, 0),
                    "SUB" => chunk.add(ByteCode::Sub, 0),
                    "MUL" => chunk.add(ByteCode::Mul, 0),
                    "DIV" => chunk.add(ByteCode::Div, 0),
                    "RET" => chunk.add(ByteCode::Ret, 0),
                    "HLT" => chunk.add(ByteCode::Hlt, 0),
                    "OUT" => chunk.add(ByteCode::Out, 0),
                    _ => (),
                }
            }
        }
        chunk
    }

    pub fn write_file(&self, filename: &str) {
        let mut f = File::create(filename).unwrap();
        for ins in &self.code {
            f.write(ins.disassemble().as_bytes()).unwrap();
            f.write("\n".as_bytes()).unwrap();
        }
    }

    pub fn to_string(&self) -> String {
        // let mut f = File::create(filename).unwrap();
        let mut s = String::new();
        for ins in &self.code {
            s.push_str("  ");
            s.push_str(ins.disassemble().as_str());
            s.push_str("\n");
        }
        s
    }

    pub fn to_string_detail(&self, obj_list: &Vec<Object>) -> String {
        // let mut f = File::create(filename).unwrap();
        let mut s = String::new();
        for ins in &self.code {
            s.push_str("  ");
            s.push_str(ins.disassemble_detail(obj_list).as_str());
            s.push_str("\n");
        }
        s
    }
}


impl Deref for Chunk {
    type Target = Vec<ByteCode>;
    fn deref(&self) -> &Self::Target {
        &self.code
    }
}

impl DerefMut for Chunk {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.code
    }
}