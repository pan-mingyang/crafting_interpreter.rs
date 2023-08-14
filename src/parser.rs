use std::{path::Prefix, process::Child};

use crate::{scanner::*, bytecode::*, precidence::Precedence};


#[derive(Default, Debug)]
pub struct Parser {
    tokens: Vec<TokenWithInfo>,
    ptr: usize,
    chunk: Chunk,
}

type ExpressionRult = (Option<fn(&mut Parser)>, Option<fn(&mut Parser)>, Precedence);

macro_rules! can_consume {
    ($val:expr, $type:pat) => {
        if let $type = $val {true} else {false}
    };
}

impl Parser {
    pub fn from_tokens(tokens: Vec<TokenWithInfo>) -> Parser {
        Parser { tokens, ptr: 0, chunk: Chunk::new() }
    }

    pub fn get_chunk(&self) -> &Chunk {
        &self.chunk
    }
    
    pub fn compile(&mut self) -> bool {
        loop {
            self.expression();
            if let Token::Eof = self.current().token {
                break true;
            }
        }
    }

    fn group(&mut self) {
        self.expression();
        self.cunsume(can_consume!(self.current().token, Token::RBracket), "Wrong Expression");
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assign);
    }

    fn unary(&mut self) {
        let prev = self.previous();
        self.parse_precedence(Precedence::Unary);
        match prev.token {
            Token::Plus => (),
            Token::Minus => self.emit_bytes(ByteCode::Neg),
            Token::Bang =>  self.emit_bytes(ByteCode::Not),
            _ => self.error("Error Unary Operator!", prev.line),
        }
    }

    fn binary(&mut self) {
        let prev = self.previous();
        let (_, _, prec) = Self::get_rule(prev.token.clone());
        self.parse_precedence(Precedence::from((prec as i32) + 1));        
        match prev.token {
            Token::Plus   => self.emit_bytes(ByteCode::Add),
            Token::Minus  => self.emit_bytes(ByteCode::Sub),
            Token::Star   => self.emit_bytes(ByteCode::Mul),
            Token::Slash  => self.emit_bytes(ByteCode::Div),
            Token::Eq     => self.emit_bytes(ByteCode::Eq),
            Token::Ne     => self.emit_bytes(ByteCode::Ne),
            Token::Lt     => self.emit_bytes(ByteCode::Lt),
            Token::Le     => self.emit_bytes(ByteCode::Le),
            Token::Gt     => self.emit_bytes(ByteCode::Gt),
            Token::Ge     => self.emit_bytes(ByteCode::Ge),
            Token::Keyword(Keyword::And) => self.emit_bytes(ByteCode::And),
            Token::Keyword(Keyword::Or)  => self.emit_bytes(ByteCode::Or),
            _ => self.error("Error Binary Operator!", self.previous().line),
        }
    }

    fn get_rule(token: Token) -> ExpressionRult
        // :returns: (prefix_fn, infix_fn, precedence)
    {
        match token {
            Token::LBracket  => (Some(Self::group),  None,               Precedence::None),
            Token::Bang      => (Some(Self::unary),  None,               Precedence::None),
            Token::Plus      => (None,               Some(Self::binary), Precedence::Term),
            Token::Minus     => (Some(Self::unary),  Some(Self::binary), Precedence::Term),
            Token::Star      => (None,               Some(Self::binary), Precedence::Factor),
            Token::Slash     => (None,               Some(Self::binary), Precedence::Factor),
            Token::Eq        => (None,               Some(Self::binary), Precedence::Eq),
            Token::Ne        => (None,               Some(Self::binary), Precedence::Eq),
            Token::Lt        => (None,               Some(Self::binary), Precedence::Cmp),
            Token::Le        => (None,               Some(Self::binary), Precedence::Cmp),
            Token::Gt        => (None,               Some(Self::binary), Precedence::Cmp),
            Token::Ge        => (None,               Some(Self::binary), Precedence::Cmp),
            
            Token::CFloat(_) => (Some(Self::number), None,               Precedence::None),
            Token::CInt(_)   => (Some(Self::number), None,               Precedence::None),
            Token::Keyword(Keyword::True)  => (Some(Self::literal), None, Precedence::None),
            Token::Keyword(Keyword::False) => (Some(Self::literal), None, Precedence::None),
            Token::Keyword(Keyword::And)   => (None, Some(Self::binary), Precedence::And),
            Token::Keyword(Keyword::Or)    => (None, Some(Self::binary), Precedence::Or),
            _ => (None, None, Precedence::None)
        }
    }

    fn number(&mut self) {
        let token = &self.previous().token;
        match token {
            Token::CInt(n) => self.emit_bytes(ByteCode::from(*n)),
            Token::CFloat(n) => self.emit_bytes(ByteCode::from(*n)),
            _ => self.error("Expect Number",  self.previous().line)
        }
    }

    fn literal(&mut self) {
        let token = &self.previous().token;
        match token {
            Token::Keyword(Keyword::True) => self.emit_bytes(ByteCode::from(true)),
            Token::Keyword(Keyword::False) => self.emit_bytes(ByteCode::from(false)),
            _ => self.error("Expect boolean literal",  self.previous().line)
        }
    }

    fn parse_precedence(&mut self, prec: Precedence) {
        self.advance();
        let (prefix, _, _) = Self::get_rule(self.previous().token);
        if let None = prefix {
            self.error("Expect expression", self.previous().line);
            return;
        }
        prefix.unwrap()(self);
        while (prec as i32) <= (Self::get_rule(self.current().token).2 as i32) {
            self.advance();
            let (_, infix, _) = Self::get_rule(self.previous().token);
            let f:Option<fn(&mut Parser)>  = Some(Self::binary);
            if let None = infix {
                self.error("Expect expression infix operation!", self.previous().line);
                return;
            }
            infix.unwrap()(self);
            // self.advance();
        }
    }


    fn cunsume(&mut self, can_consume: bool, msg: &str) {
        if can_consume {
            self.advance();
        } else {
            self.error(msg, self.current().line);
        }
    }

    pub fn emit_bytes(&mut self, byte_code: ByteCode) {
        self.chunk.add(byte_code, self.previous().line);
    }

    pub fn error(&self, msg: &str, line: usize) -> ! {
        panic!("Parsing Error: {} at line {}", msg, line)
    }

    pub fn current(&self) -> TokenWithInfo {
        self.tokens[self.ptr].clone()
    }

    pub fn previous(&self) -> TokenWithInfo {
        if self.ptr > 0 {
            self.tokens[self.ptr - 1].clone()
        } else {
            self.error("There's no previous token", 0)
        }
    }

    pub fn advance(&mut self) {
        self.ptr += 1;       
    }

}

