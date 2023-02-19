#[derive(PartialEq, Eq, Clone)]
pub enum TokenType {
    // Basic tokens
    LParen,
    RParen,
    LBrace,
    RBrace,
    Plus,
    Minus,
    Slash,
    Asterisk,
    Coma,
    Dot,
    Semicolon,
    
    // Comparison tokens
    Bang,
    BangEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    Eq,
    EqEq,

    // Anything that requires extra information to
    // be attached with it
    Ident,
    String,
    Number,

    And,
    Or,
    Class,
    Super,
    This,
    If,
    Else,
    True,
    False,
    Nil,
    For,
    While,
    Fun,
    Return,
    Var,
    Print,

    // An error token
    Error,

    // When we have reached the end of the file
    EndOfFile
}

#[derive(Clone)]
pub struct Token {
    pub ty: TokenType,
    pub text: String,
    pub line: usize,
}

impl Token {
    pub fn new(ty: TokenType, text: String, line: usize) -> Self {
        Self {
            ty,
            text,
            line
        }
    }

    pub fn new_no_text(ty: TokenType, line: usize) -> Self {
        Self::new(ty, "".to_string(), line)
    }
}

#[derive(Clone)]
pub struct Tokenizer {
    current: usize,
    start: usize,
    line: usize,
    source: String
}

impl Tokenizer {
    pub fn new(source: &str) -> Self {
        Self {
            current: 0,
            start: 0,
            line: 1,
            source: source.to_string()
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
    
    fn get_char(&self, idx: usize) -> char {
        if idx >= self.source.len() {
            '\0'
        } else {
            self.source.chars()
                .nth(idx)
                .unwrap()
        }
    }

    fn make_token(&self, ty: TokenType) -> Token {
        Token::new_no_text(ty, self.line)
    }

    fn make_token_text(&self, ty: TokenType, text: &str) -> Token {
        Token::new(ty, text.to_string(), self.line)
    }

    fn make_error(&self, error: &str) -> Token {
        self.make_token_text(TokenType::Error, error)
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.get_char(self.current - 1)
    }

    fn peek(&self) -> char {
        self.get_char(self.current)
    }

    fn peek_next(&self) -> char {
        self.get_char(self.current + 1)
    }

    fn skip_whitespace(&mut self) {
        loop {
            let character = self.peek();

            if character == ' ' || character == '\r' || character == '\t' {
                self.advance();
            } else if character == '\n' {
                self.line += 1;
                self.advance(); 
            } else if character == '/' && self.peek_next() == '/' {
                while self.peek() != '\n' && !self.is_at_end() {
                    self.advance();
                }
            } else {
                return
            }
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false
        }

        if self.source.chars().nth(self.current).unwrap() != expected {
            return false
        }

        self.current += 1;

        true
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();

        self.start = self.current;

        // Return an EOF token if the
        // current position is out
        // of the string's length.
        if self.is_at_end() {
            return self.make_token(TokenType::EndOfFile);
        }
        
        // The character may be used more than once
        let character = self.advance();

        match character {
            c if Self::is_digit(c) => {
                while Self::is_digit(self.peek()) {
                    self.advance();
                }

                if self.peek() == '.' && Self::is_digit(self.peek_next()) {
                    self.advance();

                    while Self::is_digit(self.peek()) {
                        self.advance();
                    }
                }
                
                let text = &self.source[self.start..self.current];
                self.make_token_text(TokenType::Number, text)
            },
            c if Self::is_alpha(c) => {
                while Self::is_alpha(self.peek()) || Self::is_digit(self.peek()) {
                    self.advance();
                }

                let text = &self.source[self.start..self.current];
                let token_type = Self::identifier_type(&self.source[self.start..self.current]);
                return self.make_token_text(token_type, text)
            },
            '(' => self.make_token(TokenType::LParen),
            ')' => self.make_token(TokenType::RParen),
            '{' => self.make_token(TokenType::LBrace),
            '}' => self.make_token(TokenType::RBrace),
            '+' => self.make_token(TokenType::Plus),
            '-' => self.make_token(TokenType::Minus),
            '*' => self.make_token(TokenType::Asterisk),
            '/' => self.make_token(TokenType::Slash),
            ';' => self.make_token(TokenType::Semicolon),
            '!' => if self.match_char('=') {
                self.make_token(TokenType::BangEq)
            } else {
                self.make_token(TokenType::Bang)
            },
            '=' => if self.match_char('=') {
                self.make_token(TokenType::EqEq)
            } else {
                self.make_token(TokenType::Eq)
            },
            '<' => if self.match_char('=') {
                self.make_token(TokenType::LessEq)
            } else {
                self.make_token(TokenType::Less)
            },
            '>' => if self.match_char('=') {
                self.make_token(TokenType::GreaterEq)
            } else {
                self.make_token(TokenType::Greater)
            },
            '"' => self.string(),
            _ => {
                self.make_error(&format!("Unexpected character '{}'", character))
            }
        }
    }

    fn string(&mut self) -> Token {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }

            self.advance();
        }

        if self.is_at_end() {
            return self.make_error("Unterminated string")
        }

        self.advance();
        let text = self.source[self.start+1..self.current-1].to_string();
        return Token::new(TokenType::String, text, self.line)
    }

    fn identifier_type(content: &str) -> TokenType {
        match content {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Ident
        }
    }

    fn is_alpha(c: char) -> bool {
        (c >= 'a' && c <= 'z') ||
        (c >= 'A' && c <= 'Z') ||
        c == '_'
    }

    fn is_digit(c: char) -> bool {
        c >= '0' && c <= '9'
    }
}