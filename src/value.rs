use std::cmp::Ordering;
use std::ops::{Add, Sub, Mul, Div, Neg, Rem, Shr, Shl, BitAnd, BitOr, BitXor};

use crate::object::{Object, Function};

#[derive(Default, Debug, Clone, PartialEq, Copy)]
pub enum Value {
    #[default]
    Nil,
    Unk,
    Bool(bool),
    Int(i64),
    Float(f64),
    Ptr(usize),
    StaticPtr(usize),
    // String(String),
    Obj(usize),
    Function(usize),
    NativeFunction(usize),
}

impl Value {
    pub fn to_str(&self) -> String {
        match self {
            Value::Nil => String::from("Nil"),
            Value::Unk => String::from("[Unk]"),
            Value::Bool(c) => c.to_string(),
            Value::Int(c) => c.to_string(),
            Value::Float(c) => {
                let s = c.to_string();
                if s.contains(".") {s} else {s + "."}
            },
            Value::Ptr(c) => format!("Ph_{}", c),
            Value::StaticPtr(c) => format!("Ps_{}", c),
            // Value::String(c) => format!("{}", c),
            Value::Obj(c) => format!("<Object> {}", c),
            Value::Function(c) => format!("<Function> {}", c),
            Value::NativeFunction(c) => format!("<Native Fn> {}", c),
            _ => String::new(),
        }
    }

    pub fn to_str_detail(&self, obj_list: &Vec<Object>) -> String {
        match self {
            Value::Obj(c) => obj_list[*c].to_str(),
            _ => String::new(),
        }
    }



    pub fn bool_and(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Bool(c1), Self::Bool(c2)) => Self::Bool(c1 && c2),
            _ => Self::Nil
        }
    }

    pub fn bool_or(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Bool(c1), Self::Bool(c2)) => Self::Bool(c1 || c2),
            _ => Self::Nil
        }
    }

    pub fn bool_not(self) -> Self {
        match self {
            Self::Bool(c) => Self::Bool(!c),
            _ => Self::Nil
        }
    }

    pub fn bitnot(self) -> Self {
        match self {
            Self::Int(c) => Self::Int((-1) ^ c),
            _ => Self::Nil,
        }
    }
}

macro_rules! impl_binary_op_for_constant {
    ($clz:ident, $op:ident) => {
        impl $clz for Value {
            type Output = Value;        
            fn $op(self, rhs: Self) -> Self::Output {
                match (self, rhs) {
                    (Self::Int(c1), Self::Int(c2)) => Self::Int(c1.$op(c2)),
                    (Self::Int(c1), Self::Float(c2)) => Self::Float((c1 as f64).$op(c2)),
                    (Self::Float(c1), Self::Int(c2)) => Self::Float(c1.$op(c2 as f64)),
                    (Self::Float(c1), Self::Float(c2)) => Self::Float(c1.$op(c2)),
                    _ => Self::Nil
                }
            }
        }
    };
}


macro_rules! impl_binary_op_for_integer {
    ($clz:ident, $op:ident) => {
        impl $clz for Value {
            type Output = Value;        
            fn $op(self, rhs: Self) -> Self::Output {
                match (self, rhs) {
                    (Self::Int(c1), Self::Int(c2)) => Self::Int(c1.$op(c2)),
                    _ => Self::Nil
                }
            }
        }
    };
}


impl Neg for Value {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Self::Int(c) => Self::Int(-c),
            Self::Float(c) => Self::Float(-c),
            _ => Self::Nil,
        }
    }
}

impl_binary_op_for_constant!(Add, add);
impl_binary_op_for_constant!(Sub, sub);
impl_binary_op_for_constant!(Mul, mul);

impl_binary_op_for_integer!(Rem, rem);
impl_binary_op_for_integer!(Shr, shr);
impl_binary_op_for_integer!(Shl, shl);
impl_binary_op_for_integer!(BitAnd, bitand);
impl_binary_op_for_integer!(BitOr,  bitor);
impl_binary_op_for_integer!(BitXor, bitxor);

impl Div for Value {
    type Output = Self;        
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Int(c1), Self::Int(c2)) => Self::Float((c1 as f64).div(c2 as f64)),
            (Self::Int(c1), Self::Float(c2)) => Self::Float((c1 as f64).div(c2)),
            (Self::Float(c1), Self::Int(c2)) => Self::Float(c1.div(c2 as f64)),
            (Self::Float(c1), Self::Float(c2)) => Self::Float(c1.div(c2)),
            _ => Self::Nil
        }
    }
}


impl PartialOrd for Value {
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

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
