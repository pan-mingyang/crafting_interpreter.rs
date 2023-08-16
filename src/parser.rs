use std::ops::Index;

use crate::{scanner::*, bytecode::*, precidence::Precedence, value::Value};

#[derive(Default, Debug)]
struct Environment {
    pub local: Vec<Local>,
    pub scope_depth: usize,
}

#[derive(Default, Debug, Clone)]
struct Local {
    pub name: Identifier,
    pub depth: usize,
    pub init: bool,
}

#[derive(Default, Debug)]
pub struct Parser {
    tokens: Vec<TokenWithInfo>,
    ptr: usize,
    chunk: Chunk,
    panic_mode: bool,
    pub constants: Vec<Value>,
    env: Environment,
    end_to_pop: bool,
}

type ExpressionRult = (Option<fn(&mut Parser, bool)>, Option<fn(&mut Parser, bool)>, Precedence);

macro_rules! can_consume {
    ($val:expr, $type:pat) => {
        if let $type = $val.current().token {true} else {false}
    };
}



impl Parser {
    pub fn from_tokens(tokens: Vec<TokenWithInfo>) -> Parser {
        Parser { tokens, ptr: 0, chunk: Chunk::new(), panic_mode: false, constants: vec![],
                env: Environment { local: vec![] , scope_depth: 0 }, end_to_pop: true, }
    }

    pub fn get_chunk(&self) -> &Chunk {
        &self.chunk
    }
    
    pub fn compile(&mut self) -> bool {
        loop {
            self.declaration();
            println!(" consume {:?}", self.current().token);
            self.consume(can_consume!(self, Token::NewLine), "Expect <NEWLINE>");
            println!("{:?}", self.current().token);
            if let Token::Eof = self.current().token {
                break true;
            }
        }
    }

    fn declaration(&mut self) {
        match self.current().token {
            Token::Keyword(Keyword::Let) => self.let_declaration(),
            Token::Keyword(Keyword::If)  => self.if_statement(),
            _ => self.statement(),
        }
    }

    fn if_statement(&mut self) {
        
    }


    fn let_declaration(&mut self) {
        self.advance();
        let mut def_succ = false;
        while let Token::Identifier(identifier) = self.current().token {
            def_succ = true;
            let global = self.parse_variable(identifier.name);
            self.advance();
            let mut to_break = false;
            match self.current().token {
                Token::Assign => {
                    self.advance();
                    self.expression();
                    match self.current().token {
                        Token::Comma => { self.advance(); },
                        Token::NewLine | Token::Eof => to_break = true,
                        _ => self.error("Wrong variable declaration statement")
                    }
                },
                Token::Comma => { self.emit_bytes(ByteCode::Nil); self.advance(); },
                Token::NewLine => { self.emit_bytes(ByteCode::Nil); self.advance(); to_break = true },
                c => self.error(&format!("Wrong variable declaration statement {:?}", c)[..]),
            }
            if global < usize::MAX {
                self.emit_bytes(ByteCode::DefGlobal(global));
            } else {
                let last_idx = self.env.local.len() - 1;
                self.env.local[last_idx].init = true;
                self.end_to_pop = true;
            }
            if to_break {
                break;
            }
        }        
        println!("end decl {:?}", self.current().token);
        if !def_succ { self.error("Wrong declaration"); }
    }

    fn make_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
    
    fn parse_variable(&mut self, variable: String) -> usize {
        if self.env.scope_depth > 0 {
            self.declare_variable();
            usize::MAX
        } else {
            self.make_constant(Value::String(variable))
        }
    }

        
    fn declare_variable(&mut self) {
        let name = self.current().token;
        self.add_local(name);
    }

    fn add_local(&mut self, name: Token) {
        if self.env.local.len() > 0 {
            for i in (0 .. self.env.local.len()).rev() {
                let token = self.env.local[i].clone();
                let tok_name = token.name;
                if token.depth == self.env.scope_depth {
                    match (&name, tok_name) {
                        (Token::Identifier(c1), c2) if c1.name == c2.name 
                            => {self.error("defined variable!");},
                        _ => (),
                    }
                } else {
                    break;
                }
            }
        }
        if let Token::Identifier(name) = name {
            self.env.local.push(Local { name: name, depth: self.env.scope_depth, init: false })
        } else {
            println!("                              local {:?}", name)
        }
    }

    fn get_variable(&mut self, variable: &String) -> ByteCode {
        let length = self.env.local.len();
        println!("            cmp {:?}", self.env.local);
        println!("            cmp {:?}", !self.env.local.is_empty() );
        if !self.env.local.is_empty() {
            for i in (0..length).rev() {
                let s = self.env.local[i].name.name.clone();
                println!("            cmp {} {}", s, variable);
                if s == *variable && self.env.local[i].init {
                    return ByteCode::LoadLocal(i);
                }
            }
        }
        for (i, constant) in self.constants.iter().enumerate() {
            if let Value::String(s) = constant {
                if *s == *variable {
                    return ByteCode::Load(i);
                }
            }
        }
        self.error(&format!("undefined variable {}", variable)[..]);
        ByteCode::Nil
    }

    fn variable(&mut self, can_assign: bool) {
        if let Token::Identifier(name) = self.previous().token {
            let index = self.get_variable(&name.name);
            if can_assign && matches!(self.current().token, Token::Assign) {
                self.advance();
                self.expression();
                match index {
                    ByteCode::Load(c) => self.emit_bytes(ByteCode::Set(c)),
                    ByteCode::LoadLocal(c) => self.emit_bytes(ByteCode::SetLocal(c)),
                    _ => (),
                }
                
            } else {
                self.emit_bytes(index);
            }
        }
    }

    fn group(&mut self, can_assign: bool) {
        self.expression();
        self.consume(can_consume!(self, Token::RBracket), "Wrong Expression");
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assign);
    }

    fn statement(&mut self) {
        match self.current().token {
            Token::Keyword(Keyword::Print) => self.print_statement(),
            Token::Keyword(Keyword::Block) => {
                self.advance();
                self.begin_block();
                self.block();
                self.end_block();
                self.end_to_pop = false;
            },
            _ => self.expression(),
        }
        if self.end_to_pop {
            self.emit_bytes(ByteCode::Pop);
        } else {
            self.end_to_pop = true;
        }
        if self.panic_mode {
            self.synchronize();
        }
    }

    fn block(&mut self) {
        while !(matches!(self.current().token, Token::EndBlock)) {
            println!("    Block {:?}", self.current().token);
            self.declaration();
            self.consume(can_consume!(self, Token::NewLine), "Expect new Line");
        }        
    }

    fn begin_block(&mut self) {
        self.consume(can_consume!(self, Token::Colon), "Expect ':'!");
        self.consume(can_consume!(self, Token::NewLine), "Expect new line!");
        self.consume(can_consume!(self, Token::BeginBlock), "Expect indent!");
        self.env.scope_depth += 1;
    }  

    fn end_block(&mut self) {
        println!("End block! {:?}", self.current().token);
        // self.consume(can_consume!(self, Token::NewLine), "Expect new line");
        self.consume(can_consume!(self, Token::EndBlock), "Expect end block indent!");
        self.env.scope_depth -= 1;
        
        println!("      ENV  {:?}", self.env.local);

        for i in &self.env.local {
            println!("   ENV {:?}", i);
        }

        while !self.env.local.is_empty() && self.env.local.last().unwrap().depth > self.env.scope_depth {
            self.env.local.pop();
            self.emit_bytes(ByteCode::Pop);
        }
    }


    fn print_statement(&mut self) {
        self.advance();
        self.consume(can_consume!(self, Token::LBracket), "Expect '('");
        self.expression();
        self.consume(can_consume!(self, Token::RBracket), "Expect ')'");
        println!("End print {:?}", self.current().token);
        self.emit_bytes(ByteCode::Out);
    }

    fn unary(&mut self, can_assign: bool) {
        let prev = self.previous();
        self.parse_precedence(Precedence::Unary);
        match prev.token {
            Token::Plus => (),
            Token::Minus => self.emit_bytes(ByteCode::Neg),
            Token::Bang =>  self.emit_bytes(ByteCode::Not),
            _ => self.error("Error Unary Operator!"),
        }
    }

    fn binary(&mut self, can_assign: bool) {
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
            _ => self.error("Error Binary Operator!"),
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
            // Token::Assign    => (None,               Some(Self::binary), Precedence::Assign),
            
            Token::CFloat(_)     => (Some(Self::number), None,               Precedence::None),
            Token::CInt(_)       => (Some(Self::number), None,               Precedence::None),
            Token::Identifier(_) => (Some(Self::variable), None,               Precedence::None),

            Token::Keyword(Keyword::True)  => (Some(Self::literal), None, Precedence::None),
            Token::Keyword(Keyword::False) => (Some(Self::literal), None, Precedence::None),
            Token::Keyword(Keyword::And)   => (None, Some(Self::binary), Precedence::And),
            Token::Keyword(Keyword::Or)    => (None, Some(Self::binary), Precedence::Or),
            _ => (None, None, Precedence::None)
        }
    }

    fn number(&mut self, can_assign: bool) {
        let token = &self.previous().token;
        match token {
            Token::CInt(n) => self.emit_bytes(ByteCode::from(*n)),
            Token::CFloat(n) => self.emit_bytes(ByteCode::from(*n)),
            _ => self.error("Expect Number")
        }
    }

    fn literal(&mut self, can_assign: bool) {
        let token = &self.previous().token;
        match token {
            Token::Keyword(Keyword::True) => self.emit_bytes(ByteCode::from(true)),
            Token::Keyword(Keyword::False) => self.emit_bytes(ByteCode::from(false)),
            _ => self.error("Expect boolean literal")
        }
    }

    fn parse_precedence(&mut self, prec: Precedence) {
        self.advance();
        let (prefix, _, _) = Self::get_rule(self.previous().token);
        if let None = prefix {
            self.error(&format!("Expect expression {:?}", self.current().token)[..]);
        }
        println!("unwrap {:?}", self.previous().token);
        let can_assign = prec as i32 <= Precedence::Assign as i32;
        prefix.unwrap()(self, can_assign);
        while (prec as i32) <= (Self::get_rule(self.current().token).2 as i32) {
            self.advance();
            let (_, infix, _) = Self::get_rule(self.previous().token);
            if let None = infix {
                self.error("Expect expression infix operation!");
            }
            infix.unwrap()(self, can_assign);
        }
        if can_assign && matches!(self.current().token, Token::Assign) {
            self.error("invalid assignment target!");
        }
    }


    fn consume(&mut self, can_consume: bool, msg: &str) {
        self.advance();
        if !can_consume {
            self.error(msg);
        }
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.chunk
    }

    pub fn emit_bytes(&mut self, byte_code: ByteCode) {
        let line = self.previous().line;
        let chunk = self.current_chunk();        
        chunk.add(byte_code, line);
    }

    pub fn error(&mut self, msg: &str) {
        self.panic_mode = true;
        let line = self.previous().line;
        panic!("[Parsing Error] '{}' at line {}.", msg, line);
    }

    pub fn current(&self) -> TokenWithInfo {
        self.tokens[self.ptr].clone()
    }

    pub fn previous(&mut self) -> TokenWithInfo {
        if self.ptr > 0 {
            self.tokens[self.ptr - 1].clone()
        } else {
            panic!("There's no previous token")
        }
    }

    pub fn advance(&mut self) {
        self.ptr += 1;       
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;
        while !matches!(self.current().token, Token::Eof) {
            if let Token::NewLine = self.previous().token {
                return;
            }
            match self.current().token {
                Token::Keyword(Keyword::Class) |
                Token::Keyword(Keyword::Let) |
                Token::Keyword(Keyword::Func) |
                Token::Keyword(Keyword::For) |
                Token::Keyword(Keyword::If) |
                Token::Keyword(Keyword::While) |
                Token::Keyword(Keyword::Print) |
                Token::Keyword(Keyword::Block) |
                Token::Keyword(Keyword::Return) => break,
                _ => (),
            }
            self.advance();
        }
    }

}

