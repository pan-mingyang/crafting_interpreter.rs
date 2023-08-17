use std::{rc::Rc, cell::RefCell};

use crate::{scanner::*, bytecode::*, precidence::Precedence, value::Value, object::Function};



#[derive(Default, Debug, Clone)]
enum FunctionType {
    #[default]
    Func,
    Script,
}

#[derive(Default, Debug, Clone)]
struct Environment {
    pub enclosing: Option<Box<Environment>>,
    pub func_id: usize,
    pub func_type: FunctionType,
    pub local: Vec<Local>,
    pub scope_depth: usize,
}

impl Environment {
    pub fn new() -> Self {
        Environment { 
            enclosing: None, func_id: 0, func_type: FunctionType::Script, local: vec![], scope_depth: 0 }
    }
}

#[derive(Default, Debug, Clone)]
struct Local {
    pub name: Identifier,
    pub depth: usize,
    pub init: bool,
}

#[derive(Default, Debug)]
pub struct Parser {
    pub functions: Vec<Function>,
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
        let default_function = Function {
            name: String::from("$main"),
            arity: 0,
            chunk: Chunk::new()
        };
        let mut local: Vec<Local> = Vec::new();
        // local.push(Local { name: Identifier { name: String::from("$main") }, depth: 0, init: true });
        Parser { tokens, ptr: 0, chunk: Chunk::new(), panic_mode: false, constants: vec![],
                 env: Environment::new(), end_to_pop: true, functions: vec![default_function] }
    }

    pub fn get_chunk(&self) -> &Chunk {
        &self.functions[0].chunk
    }
    
    fn set_env(&mut self, env: Environment) {
        let x = self.env.clone();
        self.functions.push(Function { arity: 0, chunk: Chunk::new(), name: String::new() });
        self.env = env;
        self.env.func_id = self.functions.len() - 1;
        self.env.enclosing = Some(Box::new(x));
    }

    fn reset_env(&mut self) {
        if let Some(c) = &self.env.enclosing {
            self.env = *c.clone()
        } else {
            self.error("no enclosing envs!")
        }
    }

    pub fn compile(&mut self) -> bool {
        let result = loop {
            self.statement();
            println!(" consume {:?}", self.current().token);
            self.consume(can_consume!(self, Token::NewLine), "Expect <NEWLINE>");
            println!("{:?}", self.current().token);
            if let Token::Eof = self.current().token {
                break true;
            }
        };
        self.emit_byte(ByteCode::Hlt);
        result
    }

    fn if_statement(&mut self) {
        self.advance();
        // self.consume(can_consume!(self, Token::LBracket), "Exprec '('");
        self.expression();
        // self.consume(can_consume!(self, Token::RBracket), "Exprec ')'");
        let to_jump = self.emit_byte_to_fill_back(ByteCode::Nop);
        self.statement();
        let to_jump_end_if = self.emit_byte_to_fill_back(ByteCode::Nop);
        let ip = self.current_chunk().len();
        self.set_chunk(to_jump, ByteCode::JZ(ip));
        println!(" NOW {:>?}", self.current().token);
        self.consume(can_consume!(self, Token::NewLine), "Expect new Line");
        let mut has_else = false;

        while let Token::Keyword(Keyword::Else) = self.current().token {
            has_else = true;
            self.advance();
            match self.current().token {
                Token::Keyword(Keyword::If) |                
                Token::Colon => self.statement(),
                _ => self.error("Expect 'if' or ':'"),                
            }
        }
        if has_else {
            let ip = self.current_chunk().len();
            self.set_chunk(to_jump_end_if, ByteCode::J(ip));
        } else {
            self.back();
        }
    }


    fn while_statement(&mut self) {
        self.advance();
        let ip_while_start = self.current_chunk().len();
        self.expression();
        let to_jump = self.emit_byte_to_fill_back(ByteCode::Nop);
        self.statement();
        // while !self.env.local.is_empty() && self.env.local.last().unwrap().depth > self.env.scope_depth {
        //     self.env.local.pop();
        //     self.emit_byte(ByteCode::Pop);
        // }
        let to_jump_while_start = self.emit_byte_to_fill_back(ByteCode::Nop);
        self.set_chunk(to_jump_while_start, ByteCode::J(ip_while_start));
        let ip = self.current_chunk().len();
        self.set_chunk(to_jump, ByteCode::JZ(ip));

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
                Token::Comma => { self.emit_byte(ByteCode::Nil); self.advance(); },
                Token::NewLine => { self.emit_byte(ByteCode::Nil); self.advance(); to_break = true },
                c => self.error(&format!("Wrong variable declaration statement {:?}", c)[..]),
            }
            if global < usize::MAX {
                self.emit_byte(ByteCode::DefGlobal(global));
            } else {
                let last_idx = self.env.local.len() - 1;
                self.env.local[last_idx].init = true;
                // self.end_to_pop = false;
            }
            if to_break {
                break;
            }
        }
        println!("end decl {:?}", self.current());
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
        if !self.env.local.is_empty() {
            for i in (0..length).rev() {
                let s = self.env.local[i].name.name.clone();
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
                    ByteCode::Load(c) => self.emit_byte(ByteCode::Set(c)),
                    ByteCode::LoadLocal(c) => self.emit_byte(ByteCode::SetLocal(c)),
                    _ => (),
                }
            } else {
                self.emit_byte(index);
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
            Token::Keyword(Keyword::Print) => { self.print_statement(); self.end_to_pop = false; },
            Token::Keyword(Keyword::If)    => {
                self.if_statement();
                self.end_to_pop = false;
            },
            Token::Keyword(Keyword::While)    => {
                self.while_statement();
                self.end_to_pop = false;
            },
            Token::Keyword(Keyword::Block) => {
                self.advance();
                self.begin_block();
                self.block();
                self.end_block();
                self.end_to_pop = false;
            },            
            Token::Colon => {
                self.begin_block();
                self.block();
                self.end_block();
                self.end_to_pop = false;
            },
            Token::Keyword(Keyword::Let) => {
                self.let_declaration();
                self.end_to_pop = false;
            },
            Token::Keyword(Keyword::Func) => {
                self.func_declaration();
                self.end_to_pop = false
            }
            _ => self.expression(),
        }
        if self.end_to_pop {
            self.emit_byte(ByteCode::Pop);
        } else {
            self.end_to_pop = true;
        }
        if self.panic_mode {
            self.synchronize();
        }
    }


    fn func_declaration(&mut self) {
        // if self.env.scope_depth != 0 {
        //     self.error("inner function is not supported!");
        // }
        self.advance();
        if let Token::Identifier(Identifier{ name: func_name }) = self.current().token {
            let global = self.parse_variable(func_name);
            // mark initialized
            if global == usize::MAX {
                let last_idx = self.env.local.len() - 1;
                self.env.local[last_idx].init = true;
            }  
            let env = Environment::new();
            self.set_env(env);
            self.func_param();

            // define global
            // todo!
            self.reset_env();
        } else {
            self.error("Expect function name!");
        }                
    }

    fn func_param(&mut self) {

    }

    fn block(&mut self) {
        while !(matches!(self.current().token, Token::EndBlock)) {
            println!("    Block {:?}", self.current().token);
            self.statement();
            println!("delc then {:?}", self.current());
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
        
        for i in &self.env.local {
            println!("   ENV {:?}", i);
        }

        while !self.env.local.is_empty() && self.env.local.last().unwrap().depth > self.env.scope_depth {
            self.env.local.pop();
            self.emit_byte(ByteCode::Pop);
        }
    }


    fn print_statement(&mut self) {
        self.advance();
        self.consume(can_consume!(self, Token::LBracket), "Expect '('");
        self.expression();
        self.consume(can_consume!(self, Token::RBracket), "Expect ')'");
        println!("End print {:?}", self.current().token);
        self.emit_byte(ByteCode::Out);
        self.emit_byte(ByteCode::Pop);
    }

    fn unary(&mut self, can_assign: bool) {
        let prev = self.previous();
        self.parse_precedence(Precedence::Unary);
        match prev.token {
            Token::Plus => (),
            Token::Minus => self.emit_byte(ByteCode::Neg),
            Token::Bang =>  self.emit_byte(ByteCode::Not),
            Token::LNot =>  self.emit_byte(ByteCode::LNot),
            _ => self.error("Error Unary Operator!"),
        }
    }

    fn binary(&mut self, can_assign: bool) {
        let prev = self.previous();
        let (_, _, prec) = Self::get_rule(prev.token.clone());
        self.parse_precedence(Precedence::from((prec as i32) + 1));        
        match prev.token {
            Token::Plus   => self.emit_byte(ByteCode::Add),
            Token::Minus  => self.emit_byte(ByteCode::Sub),
            Token::Star   => self.emit_byte(ByteCode::Mul),
            Token::Slash  => self.emit_byte(ByteCode::Div),
            Token::Mod    => self.emit_byte(ByteCode::Mod),
            Token::Eq     => self.emit_byte(ByteCode::Eq),
            Token::Ne     => self.emit_byte(ByteCode::Ne),
            Token::Lt     => self.emit_byte(ByteCode::Lt),
            Token::Le     => self.emit_byte(ByteCode::Le),
            Token::Gt     => self.emit_byte(ByteCode::Gt),
            Token::Ge     => self.emit_byte(ByteCode::Ge),
            Token::Shr    => self.emit_byte(ByteCode::Shr),
            Token::Shl    => self.emit_byte(ByteCode::Shl),
            Token::LAnd   => self.emit_byte(ByteCode::LAnd),
            Token::LOr    => self.emit_byte(ByteCode::LOr),
            Token::LXor   => self.emit_byte(ByteCode::LXor),
            Token::Keyword(Keyword::And) => self.emit_byte(ByteCode::And),
            Token::Keyword(Keyword::Or)  => self.emit_byte(ByteCode::Or),
            _ => self.error(&format!("Error Binary Operator! {:?}", prev)[..]),
        }
    }

    fn get_rule(token: Token) -> ExpressionRult
        // :returns: (prefix_fn, infix_fn, precedence)
    {
        match token {
            Token::LBracket  => (Some(Self::group),  None,               Precedence::None),
            Token::Bang | Token::Keyword(Keyword::Not) | Token::LNot
                             => (Some(Self::unary),  None,               Precedence::None),
            Token::Plus      => (None,               Some(Self::binary), Precedence::Term),
            Token::Minus     => (Some(Self::unary),  Some(Self::binary), Precedence::Term),
            Token::Star      => (None,               Some(Self::binary), Precedence::Factor),
            Token::Slash     => (None,               Some(Self::binary), Precedence::Factor),
            Token::Mod       => (None,               Some(Self::binary), Precedence::Factor),
            Token::Shr       => (None,               Some(Self::binary), Precedence::Shift),
            Token::Shl       => (None,               Some(Self::binary), Precedence::Shift),
            Token::LOr       => (None,               Some(Self::binary), Precedence::LogicOr),
            Token::LAnd      => (None,               Some(Self::binary), Precedence::LogicAnd),
            Token::LXor      => (None,               Some(Self::binary), Precedence::LogicAnd),
            Token::Eq        => (None,               Some(Self::binary), Precedence::Eq),
            Token::Ne        => (None,               Some(Self::binary), Precedence::Eq),
            Token::Lt        => (None,               Some(Self::binary), Precedence::Cmp),
            Token::Le        => (None,               Some(Self::binary), Precedence::Cmp),
            Token::Gt        => (None,               Some(Self::binary), Precedence::Cmp),
            Token::Ge        => (None,               Some(Self::binary), Precedence::Cmp),
            // Token::Assign    => (None,               Some(Self::binary), Precedence::Assign),
            
            Token::CFloat(_)     => (Some(Self::number),   None,  Precedence::None),
            Token::CInt(_)       => (Some(Self::number),   None,  Precedence::None),
            Token::CStr(_)       => (Some(Self::number),   None,  Precedence::None),
            Token::Identifier(_) => (Some(Self::variable), None,  Precedence::None),

            Token::Keyword(Keyword::Nil)  => (Some(Self::literal), None, Precedence::None),
            Token::Keyword(Keyword::True)  => (Some(Self::literal), None, Precedence::None),
            Token::Keyword(Keyword::False) => (Some(Self::literal), None, Precedence::None),
            Token::Keyword(Keyword::And)   => (None, Some(Self::binary), Precedence::And),
            Token::Keyword(Keyword::Or)    => (None, Some(Self::binary), Precedence::Or),
            _ => (None, None, Precedence::None)
        }
    }

    fn number(&mut self, _: bool) {
        let token = &self.previous().token;
        match token {
            Token::CInt(n) => self.emit_byte(ByteCode::from(*n)),
            Token::CFloat(n) => self.emit_byte(ByteCode::from(*n)),
            Token::CStr(s) => self.emit_byte(ByteCode::from(s.clone())),
            _ => self.error("Expect Number")
        }
    }

    fn literal(&mut self, _: bool) {
        let token = &self.previous().token;
        match token {
            Token::Keyword(Keyword::True) => self.emit_byte(ByteCode::from(true)),
            Token::Keyword(Keyword::False) => self.emit_byte(ByteCode::from(false)),
            Token::Keyword(Keyword::Nil) => self.emit_byte(ByteCode::Value(Value::Nil)),
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
        &mut self.functions[self.env.func_id].chunk
    }

    fn set_chunk(&mut self, ip: usize, value: ByteCode) {
        let chunk = self.current_chunk();
        chunk[ip] = value;
    }

    pub fn emit_byte(&mut self, byte_code: ByteCode) {
        let line = self.previous().line;
        let chunk = self.current_chunk();        
        chunk.add(byte_code, line);
    }

    pub fn emit_byte_to_fill_back(&mut self, byte_code: ByteCode) -> usize {
        let line = self.previous().line;
        let chunk = self.current_chunk();        
        chunk.add(byte_code, line);
        chunk.len() - 1
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

    pub fn back(&mut self) {
        if self.ptr > 0 {
            self.ptr -= 1;   
        } else {
            self.error("Cannot back `ip`!")
        } 
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

