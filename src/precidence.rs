use std::panic::AssertUnwindSafe;


#[derive(Debug, Default, Clone, Copy)]
pub enum Precedence{
    #[default]
    None = 0,
    Assign,  // =
    Or,          // or
    And,         // and
    Eq,    // == !=
    Cmp,  // < > <= >=
    Term,        // + -
    Factor,      // * /
    Unary,       // ! -
    Call,        // . ()
    Primary
}


impl From<i32> for Precedence {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Assign,
            2 => Self::Or,
            3 => Self::And,
            4 => Self::Eq,
            5 => Self::Cmp,
            6 => Self::Term,
            7 => Self::Factor,
            8 => Self::Unary,
            9 => Self::Call,
            10 => Self::Primary,
            _ => Self::None,
        }
    }
}