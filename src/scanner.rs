use core::prelude;
use std::{fs::File, error::Error, io::Read};


#[derive(Clone, Default, Debug)]
pub enum Token {
    #[default]
    Unk, Empty,
    LBrace, RBrace, LSBracket, RSBracket, LBracket, RBracket,
    Comma, Dot, Dollar, Colon, Pound, At, Semicolon,
    Plus, Minus, Star, Slash, Mod, LAnd, LOr, LNot, LXor, Assign, Shr, Shl,
    Eq, Ne, Le, Ge, Lt, Gt,
    Point, To, VSlash, Bang,
    BeginBlock, EndBlock,
    NewLine,
    Space(usize),
    CInt(i64), CFloat(f64), CStr(String),
    // Keywords
    Keyword(Keyword),
    Identifier(Identifier),
    Eof
}

#[derive(Clone, Default, Debug, Copy)]
pub enum Keyword {
    #[default] Unk,
    Let, Func, Class,
    If, Else, While, For, In, Break, Continue,
    And, Or, Not, Int, Str, Float, Bool,
    True, False, Nil,
    Import, Return, Kself,
    Print, Block
}


#[derive(Clone, Default, Debug)]
pub struct Identifier {
    pub name: String,
}



#[derive(Default, Debug)]
pub struct Scanner {
    pub code: String,
    ptr: usize,
    line: usize,
}

#[derive(Default, Debug, Clone)]
pub struct TokenWithInfo {
    pub token: Token,
    pub line: usize,
    pub level: usize
}


impl Scanner {
    
    pub fn scan(&mut self) -> Vec<TokenWithInfo> {
        let mut token_seq: Vec<(Token, usize)> = Vec::new();
        while !self.is_finished() {
            // println!("{} {}", self.ptr, self.cur_char());
            let ch = self.cur_char();
            let mut next_flag = true;
            let tok = match ch {                
                'a'..='z' | 'A'..='Z' | '_'  => {next_flag = false; self.match_identity()},
                '0'..='9' => { next_flag = false; self.match_number()},
                '"' => { next_flag = false; self.match_str() },                
                '!' | '>' | '<' | '-' | '=' | '/' | '|' | '&' => {
                    self.next(); 
                    let mut token = Token::Unk;
                    if !self.is_finished() {
                        token = match (ch, self.cur_char()) {
                            ('!', '=') => Token::Ne,
                            ('>', '=') => Token::Ge,
                            ('>', '>') => Token::Shr,
                            ('<', '=') => Token::Le,
                            ('<', '<') => Token::Shl,
                            ('-', '>') => Token::Point,
                            ('=', '>') => Token::To,
                            ('=', '=') => Token::Eq,
                            ('|', '|') => Token::Keyword(Keyword::Or),
                            ('&', '&') => Token::Keyword(Keyword::And),
                            ('/', '*') | ('/', '/') => {self.skip_comment(); /* self.back(); */ Token::Empty},
                            _ => Token::Unk,                             
                        }
                    } 
                    if let Token::Unk = token { 
                        token = match ch {
                            '!' => {self.back(); Token::Bang},
                            '>' => {self.back(); Token::Gt},
                            '<' => {self.back(); Token::Lt},
                            '-' => {self.back(); Token::Minus},
                            '=' => {self.back(); Token::Assign},
                            '/' => {self.back(); Token::Slash},
                            '|' => {self.back(); Token::LOr},
                            '&' => {self.back(); Token::LAnd},
                            _ => Token::Unk,
                        }
                     }
                     token
                },
                '(' => Token::LBracket,
                ')' => Token::RBracket,
                '[' => Token::LSBracket,
                ']' => Token::RSBracket,
                '{' => Token::LBrace,
                '}' => Token::RBrace,
                ',' => Token::Comma,
                '.' => Token::Dot,
                '$' => Token::Dollar,
                ':' => Token::Colon,
                '#' => Token::Pound,
                '@' => Token::At,
                ';' => Token::Semicolon,
                '+' => Token::Plus,
                '*' => Token::Star,
                '%' => Token::Mod,
                '~' => Token::LNot,
                '^' => Token::LXor,
                '\\' => Token::VSlash,
                _ => {
                    if ch.is_ascii_whitespace() {
                        next_flag = false;
                        self.skip_whitespace()
                    } else {
                        Token::Unk
                    }
                },
            };
            token_seq.push((tok, self.line));
            if next_flag {
                self.next();
            }
        }
        let line = if token_seq.is_empty() {0usize} else {token_seq.last().unwrap().1};
        token_seq.push((Token::Eof, line));
        self.post_process(&mut token_seq)
    }

    fn post_process(&mut self, token_seq: &mut Vec<(Token, usize)>) -> Vec<TokenWithInfo> {
        let mut result: Vec<TokenWithInfo> = Vec::new();
        let mut pre_line = 0usize;
        let mut indents: Vec<isize> = vec![0];
        let token_seq: Vec<_> = token_seq
                        .iter()
                        .enumerate()
                        .filter(|(i, (tok, line))| {
                            if *i <= 1 || *i >= token_seq.len() - 2 {true}
                            else {!(matches!(tok, Token::Space(_)) && *line != token_seq[*i+1].1) }
                        })
                        .map(|(_, k)| k)
                        .filter(|(token, _)| !matches!(token, Token::Empty))
                        .collect();

        for (i, (token, line)) in token_seq.iter().enumerate() {
            if let Token::Empty = token {
                continue
            }
            let mut has_space = false;
            // let pre_token = result.last().unwrap().token.clone();
            if !result.is_empty() && *line != pre_line && !matches!(result.last().unwrap().token.clone(), Token::NewLine) {
                result.push(TokenWithInfo {token: Token::NewLine, line: pre_line + 1, level: 0});
                has_space = true;
            }
            pre_line = *line;
            if has_space || matches!(token, Token::Space(_)) {
                let space = match token {
                    Token::Space(s) => *s,
                    _ => 0,
                };
                // let Token::Space(space) = token
                if (space as isize) > (*indents.last().unwrap()) {
                    indents.push(space as isize);
                    result.push(TokenWithInfo { token: Token::BeginBlock, line: *line + 1, level: 0 });
                } else if (space as isize) < (*indents.last().unwrap()) {
                    let mut cnt: i32 = 0;
                    while (space as isize) < (*indents.last().unwrap()) {
                        indents.pop();
                        result.push(TokenWithInfo { token: Token::EndBlock, line: *line, level: 0 });
                        result.push(TokenWithInfo { token: Token::NewLine,  line: *line, level: 0 });
                    }
                    if (space as isize) > (*indents.last().unwrap()) {
                        panic!("Wrong indent at line {}", *line + 1);
                    }
                }
            }

            if !matches!(token, Token::Space(_)) {
                result.push(TokenWithInfo {
                    token: token.clone(),
                    line: *line + 1,
                    level: 0
                });
            }
        }

        // println!("Final Tok Seq");
        // for TokenWithInfo { token, line, level: _ } in &mut result.iter() {
        //     println!("{}\t{:?}", line, token);
        // }
        // println!();
        // println!();
        result
    }

    fn match_identity(&mut self) -> Token {
        // println!("match_identity");
        let mut ident = String::new();
        ident.push(self.cur_char());
        // let mut ch: char;
        self.next();
        while !self.is_finished() && 
            (self.cur_char().is_ascii_alphanumeric() || self.cur_char() == '_') 
        {
            ident.push(self.cur_char());
            self.next();
        }
        match &ident[..] {
            "let" => Token::Keyword(Keyword::Let),
            "func" => Token::Keyword(Keyword::Func),
            "class" => Token::Keyword(Keyword::Class),
            "if" => Token::Keyword(Keyword::If),
            "else" => Token::Keyword(Keyword::Else),
            "while" => Token::Keyword(Keyword::While),
            "for" => Token::Keyword(Keyword::For),
            "in" => Token::Keyword(Keyword::In),
            "break" => Token::Keyword(Keyword::Break),
            "continue" => Token::Keyword(Keyword::Continue),
            "and" => Token::Keyword(Keyword::And),
            "or" => Token::Keyword(Keyword::Or),
            "not" => Token::Keyword(Keyword::Not),
            "int" => Token::Keyword(Keyword::Int),
            "str" => Token::Keyword(Keyword::Str),
            "float" => Token::Keyword(Keyword::Float),
            "bool" => Token::Keyword(Keyword::Bool),
            "true" => Token::Keyword(Keyword::True),
            "false" => Token::Keyword(Keyword::False),
            "nil" => Token::Keyword(Keyword::Nil),
            "import" => Token::Keyword(Keyword::Import),
            "return" => Token::Keyword(Keyword::Return),
            "self" => Token::Keyword(Keyword::Kself),
            "print" => Token::Keyword(Keyword::Print),
            "block" => Token::Keyword(Keyword::Block),
            s => Token::Identifier(Identifier { name: String::from(s) }),
        }

    }

    fn match_number(&mut self) -> Token {
        // println!("match_number");
        let mut s = String::new();
        while !self.is_finished() && 
            (self.cur_char().is_numeric() || self.cur_char() == '.') 
        {
            s.push(self.cur_char());
            self.next();
        }

        if let Ok(n) = str::parse::<i64>(&s[..]) {
            Token::CInt(n)
        } else if let Ok(n) = str::parse::<f64>(&s[..]) {
            Token::CFloat(n)
        } else {
            panic!("{}",  format!("Wrong number at line {}\n", self.line))
        }
    }

    fn match_str(&mut self) -> Token {
        // println!("match_str");
        self.next();
        let mut s = String::new();
        while !self.is_finished()  {
            match self.cur_char() {
                '\\' => {
                    self.next();
                    if self.is_finished() {
                        panic!("Err String!");
                    }
                    match self.cur_char() {
                        'n'  => s.push('\n'),
                        'r'  => s.push('\r'),
                        't'  => s.push('\t'),
                        '\\' => s.push('\\'),
                        '\'' => s.push('\''),
                        '\"' => s.push('\"'),
                        _ => panic!("Unk \\{} token!", self.cur_char()),
                    }
                },
                '\"' => {self.next(); break},
                _ => s.push(self.cur_char()),
            }
            self.next();
        }
        Token::CStr(s)
    }

    fn skip_whitespace(&mut self) -> Token {
        let mut new_line = false;
        if let Some(ch) = self.prev_char() {
            if ch == '\n' {
                new_line = true;
            }
        }
        let mut space = 0;
        while !self.is_finished() {
            if !self.cur_char().is_ascii_whitespace() {
                break;
            }
            match self.cur_char() {
                '\n' => { new_line = true; space = 0; self.line += 1; },
                ' ' => if new_line {space += 1},
                '\t' => if new_line {space += 4},
                _ => (),
            }
            self.next();
        }
        if space > 0 {Token::Space(space)} else {Token::Empty}
    }

    fn skip_comment(&mut self) {
        // self.next();
        // println!("skip_comment");
        match self.cur_char() {
            '/' => {
                while !self.is_finished() {
                    self.next();
                    let ch = self.cur_char();
                    if ch == '\n' { self.line += 1; return; }
                }
            },
            '*' => {
                while !self.is_finished() {
                    self.next();
                    let ch = self.cur_char();
                    if ch == '*'  { if let Some('/') = self.peek() { self.next(); return; }}
                    else if ch == '\n' {self.line += 1;}
                }
            },
            _ => panic!("Error Comment"),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let mut f = File::open(path)?;
        let mut code = String::new();
        _ = f.read_to_string(&mut code)?;
        code.push('\n');
        Ok(Self { code, ..Default::default() })
    }

    fn is_finished(&self) -> bool {
        self.code.len() <= self.ptr
    }

    fn peek(&self) -> Option<char> {
        self.code.chars().nth(self.ptr + 1)
    }

    pub fn cur_char(&self) -> char {
        self.code.chars().nth(self.ptr).unwrap()
    }
    
    pub fn prev_char(&self) -> Option<char> {
        if self.ptr > 0 {
            self.code.chars().nth(self.ptr - 1)
        } else {
            None
        }
    }

    pub fn next(&mut self) {
        self.ptr += 1;
        // self.code.chars().nth(self.ptr).unwrap()
    }
    
    pub fn back(&mut self) {
        self.ptr -= 1;
    }
}
