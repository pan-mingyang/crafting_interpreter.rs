use std::panic::AssertUnwindSafe;


#[derive(Debug, Default, Clone, Copy)]
pub enum Precedence{
    #[default]
    None = 0,
    Assign,  // =
    Or,          // or
    And,         // and
    LogicOr,
    LogicXor,
    LogicAnd, // &
    Eq,    // == !=
    Cmp,  // < > <= >=
    Shift,
    Term,        // + -
    Factor,      // * / %
    Unary,       // ! -
    Call,        // . ()
    Primary
}


impl From<i32> for Precedence {
    fn from(value: i32) -> Self {
        match value {
            0  => Self::None,
            1  => Self::Assign,
            2  => Self::Or,
            3  => Self::And,
            4  => Self::LogicOr,
            5  => Self::LogicXor,
            6  => Self::LogicAnd,
            7  => Self::Eq,
            8  => Self::Cmp,
            9  => Self::Shift,
            10 => Self::Term,
            11 => Self::Factor,
            12 => Self::Unary,
            13 => Self::Call,
            14 => Self::Primary,
            _ => Self::None,
        }
    }
}