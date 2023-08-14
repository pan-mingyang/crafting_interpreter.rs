use std::{ops::{DerefMut, Deref, Add, Sub, Mul, Div, Neg}, default, fs::File, io::BufReader, cmp::Ordering};
use std::io::prelude::*;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum Constant {
    #[default]
    Nil,
    Unk,
    Bool(bool),
    Int(i64),
    Float(f64),
    Ptr(usize),
}

pub enum CmpResult {
    G, E, L
}

impl Constant {
    pub fn to_str(&self) -> String {
        match self {
            Constant::Nil => String::from("Nil"),
            Constant::Unk => String::from("[Unk]"),
            Constant::Bool(c) => c.to_string(),
            Constant::Int(c) => c.to_string(),
            Constant::Float(c) => {
                let s = c.to_string();
                if s.contains(".") {s} else {s + "."}
            },
            Constant::Ptr(c) => format!("P_{}", c),
        }
    }

    pub fn bool_and(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Bool(c1), Self::Bool(c2)) => Self::Bool(c1 && c2),
            _ => Self::Nil // todo
        }
    }

    pub fn bool_or(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Bool(c1), Self::Bool(c2)) => Self::Bool(c1 || c2),
            _ => Self::Nil // todo
        }
    }

    pub fn bool_not(self) -> Self {
        match self {
            Self::Bool(c) => Self::Bool(!c),
            _ => Self::Nil // todo
        }
    }


}

macro_rules! impl_binary_op_for_constant {
    ($clz:ident, $op:ident) => {
        impl $clz for Constant {
            type Output = Constant;        
            fn $op(self, rhs: Self) -> Self::Output {
                match (self, rhs) {
                    (Self::Int(c1), Self::Int(c2)) => Self::Int(c1.$op(c2)),
                    (Self::Int(c1), Self::Float(c2)) => Self::Float((c1 as f64).$op(c2)),
                    (Self::Float(c1), Self::Int(c2)) => Self::Float(c1.$op(c2 as f64)),
                    (Self::Float(c1), Self::Float(c2)) => Self::Float(c1.$op(c2)),
                    _ => Self::Nil // todo
                }
            }
        }
    };
}

impl Neg for Constant {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Self::Int(c) => Self::Int(-c),
            Self::Float(c) => Self::Float(-c),
            _ => Self::Nil, // todo
        }
    }
}

impl_binary_op_for_constant!(Add, add);
impl_binary_op_for_constant!(Sub, sub);
impl_binary_op_for_constant!(Mul, mul);

impl Div for Constant {
    type Output = Self;        
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Int(c1), Self::Int(c2)) => Self::Float((c1 as f64).div(c2 as f64)),
            (Self::Int(c1), Self::Float(c2)) => Self::Float((c1 as f64).div(c2)),
            (Self::Float(c1), Self::Int(c2)) => Self::Float(c1.div(c2 as f64)),
            (Self::Float(c1), Self::Float(c2)) => Self::Float(c1.div(c2)),
            _ => Self::Nil // todo
        }
    }
}


impl PartialOrd for Constant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Int(c1),   Self::Int(c2))   => if *c1 == *c2 { Some(Ordering::Equal) } else if *c1 > *c2 { Some(Ordering::Greater) } else { Some(Ordering::Less) },
            (Self::Int(c1),   Self::Float(c2)) => if *c1 as f64 == *c2 { Some(Ordering::Equal) } else if *c1 as f64 > *c2 { Some(Ordering::Greater) } else { Some(Ordering::Less) },
            (Self::Float(c1), Self::Int(c2))   => if *c1 == *c2 as f64 { Some(Ordering::Equal) } else if *c1 > *c2 as f64 { Some(Ordering::Greater) } else { Some(Ordering::Less) },
            (Self::Float(c1), Self::Float(c2)) => if c1 == c2 { Some(Ordering::Equal) } else if c1 > c2 { Some(Ordering::Greater) } else { Some(Ordering::Less) },
            _ => None
        }
    }
}

impl From<bool> for Constant {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

#[derive(Default, Debug, PartialEq)]
pub enum ByteCode {    
    #[default]
    Hlt,
    Ret,
    Out,
    Constant(Constant),
    Add, Sub, Mul, Div, Neg,
    True, False,
    Nil,
    And, Or, Not,
    Eq, Ne, Lt, Le, Gt, Ge,
}


impl ByteCode {

    pub fn disassemble(&self) -> String {
        match self {
            ByteCode::Ret => String::from("RET"),
            ByteCode::Out => String::from("OUT"),
            ByteCode::Add => String::from("ADD"),
            ByteCode::Sub => String::from("SUB"),
            ByteCode::Mul => String::from("MUL"),
            ByteCode::Div => String::from("DIV"),
            ByteCode::Constant(c) => String::from("C\t") + &c.to_str(),
            _ => String::from("[UNK]")
        }
    }
}

impl From<f64> for ByteCode {
    fn from(value: f64) -> Self {
        Self::Constant(Constant::Float(value))
    }
}

impl From<i64> for ByteCode {
    fn from(value: i64) -> Self {
        Self::Constant(Constant::Int(value))
    }
}

impl From<bool> for ByteCode {
    fn from(value: bool) -> Self {
        Self::Constant(Constant::Bool(value))
    }
}

#[derive(Default, Debug)]
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
                        chunk.add(ByteCode::Constant(Constant::Int(x)), 0);
                    } else if let Ok(x) = number.parse::<f64>() {
                        chunk.add(ByteCode::Constant(Constant::Float(x)), 0);
                    } else if let Ok(x) = number.parse::<bool>() {
                        chunk.add(ByteCode::Constant(Constant::Bool(x)), 0);
                    } else if number.starts_with("P_") {
                        let x = number[2..].parse::<usize>().unwrap();
                        chunk.add(ByteCode::Constant(Constant::Ptr(x)), 0);
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