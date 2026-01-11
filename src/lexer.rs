#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Let,
    Mod,
    Print,
    If,
    Then,
    Else,
    ElseIf,
    End,
    While,
    Do,
    Loop,
    Break,
    Continue,
    Fn,
    Return,
    Nil,
    True,
    False,
    And,
    Or,
    Not,
    Identifier(String),
    Number(i64),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,      // %
    Equal,        // =
    EqualEqual,   // ==
    BangEqual,    // !=
    Less,         // <
    Greater,      // >
    LessEqual,    // <=
    GreaterEqual, // >=
    LParen,
    RParen,
    Comma, // ,
    EOF,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.position >= self.input.len() {
            return Token::EOF;
        }

        let ch = self.input[self.position];

        if ch.is_digit(10) {
            return self.read_number();
        }

        if ch.is_alphabetic() || ch == '_' {
            return self.read_identifier();
        }

        match ch {
            '+' => {
                self.advance();
                Token::Plus
            }
            '-' => {
                self.advance();
                Token::Minus
            }
            '*' => {
                self.advance();
                Token::Star
            }
            '/' => {
                self.advance();
                if self.match_char('/') {
                    // Single-line comment
                    while self.position < self.input.len() && self.input[self.position] != '\n' {
                        self.advance();
                    }
                    self.next_token()
                } else if self.match_char('*') {
                    // Multi-line comment
                    loop {
                        if self.position >= self.input.len() {
                            break;
                        }
                        if self.input[self.position] == '*' {
                            self.advance();
                            if self.match_char('/') {
                                break;
                            }
                        } else {
                            self.advance();
                        }
                    }
                    self.next_token()
                } else {
                    Token::Slash
                }
            }
            '%' => {
                self.advance();
                Token::Percent
            }
            '(' => {
                self.advance();
                Token::LParen
            }
            ')' => {
                self.advance();
                Token::RParen
            }
            ',' => {
                self.advance();
                Token::Comma
            }
            '=' => {
                self.advance();
                if self.match_char('=') {
                    Token::EqualEqual
                } else {
                    Token::Equal
                }
            }
            '!' => {
                self.advance();
                if self.match_char('=') {
                    Token::BangEqual
                } else {
                    panic!("Unexpected character: !");
                }
            }
            '<' => {
                self.advance();
                if self.match_char('=') {
                    Token::LessEqual
                } else {
                    Token::Less
                }
            }
            '>' => {
                self.advance();
                if self.match_char('=') {
                    Token::GreaterEqual
                } else {
                    Token::Greater
                }
            }
            _ => panic!("Unexpected character: {}", ch),
        }
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.position >= self.input.len() {
            return false;
        }
        if self.input[self.position] != expected {
            return false;
        }
        self.position += 1;
        true
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.input[self.position].is_whitespace() {
            self.advance();
        }
    }

    fn read_number(&mut self) -> Token {
        let start = self.position;
        while self.position < self.input.len() && self.input[self.position].is_digit(10) {
            self.advance();
        }
        let number_str: String = self.input[start..self.position].iter().collect();
        Token::Number(number_str.parse().unwrap())
    }

    fn read_identifier(&mut self) -> Token {
        let start = self.position;
        while self.position < self.input.len()
            && (self.input[self.position].is_alphanumeric() || self.input[self.position] == '_')
        {
            self.advance();
        }
        let text: String = self.input[start..self.position].iter().collect();

        match text.as_str() {
            "let" => Token::Let,
            "mod" => Token::Mod,
            "print" => Token::Print,
            "if" => Token::If,
            "then" => Token::Then,
            "else" => Token::Else,
            "elseif" => Token::ElseIf,
            "end" => Token::End,
            "while" => Token::While,
            "do" => Token::Do,
            "loop" => Token::Loop,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "fn" => Token::Fn,
            "return" => Token::Return,
            "nil" => Token::Nil,
            "true" => Token::True,
            "false" => Token::False,
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            _ => Token::Identifier(text),
        }
    }
}
