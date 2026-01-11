use crate::ast::{Expr, Op, Stmt};
use crate::lexer::{Lexer, Token};

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current_token = lexer.next_token();
        Self {
            lexer,
            current_token,
        }
    }

    fn eat(&mut self, token: Token) {
        if std::mem::discriminant(&self.current_token) == std::mem::discriminant(&token) {
            self.current_token = self.lexer.next_token();
        } else {
            panic!(
                "Expected token {:?}, but found {:?}",
                token, self.current_token
            );
        }
    }

    pub fn parse_program(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while self.current_token != Token::EOF {
            statements.push(self.parse_statement());
        }
        statements
    }

    fn parse_statement(&mut self) -> Stmt {
        match self.current_token {
            Token::Let => self.parse_let(),
            Token::Print => self.parse_print(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::Loop => self.parse_loop(),
            Token::Break => {
                self.eat(Token::Break);
                Stmt::Break
            }
            Token::Continue => {
                self.eat(Token::Continue);
                Stmt::Continue
            }
            Token::Return => self.parse_return(),
            Token::Fn => self.parse_fn(),
            Token::Identifier(_) => self.parse_identifier_stmt(),
            _ => panic!("Unexpected token in statement: {:?}", self.current_token),
        }
    }

    fn parse_identifier_stmt(&mut self) -> Stmt {
        let name = match &self.current_token {
            Token::Identifier(name) => name.clone(),
            _ => panic!("Expected identifier"),
        };
        self.eat(Token::Identifier(String::new()));

        if self.current_token == Token::Equal {
            self.eat(Token::Equal);
            let value = self.parse_expr();
            Stmt::Assign { name, value }
        } else if self.current_token == Token::LParen {
            self.eat(Token::LParen);
            let args = self.parse_arguments();
            self.eat(Token::RParen);
            Stmt::ExprStmt(Expr::Call(name, args))
        } else {
            panic!(
                "Unexpected token after identifier in statement: {:?}",
                self.current_token
            );
        }
    }

    fn parse_fn(&mut self) -> Stmt {
        self.eat(Token::Fn);
        let name = match &self.current_token {
            Token::Identifier(name) => name.clone(),
            _ => panic!("Expected function name"),
        };
        self.eat(Token::Identifier(String::new()));

        self.eat(Token::LParen);
        let mut params = Vec::new();
        if self.current_token != Token::RParen {
            loop {
                let param_name = match &self.current_token {
                    Token::Identifier(name) => name.clone(),
                    _ => panic!("Expected parameter name"),
                };
                self.eat(Token::Identifier(String::new()));
                params.push(param_name);

                if self.current_token == Token::Comma {
                    self.eat(Token::Comma);
                } else {
                    break;
                }
            }
        }
        self.eat(Token::RParen);

        self.eat(Token::Do);
        let mut body = Vec::new();
        while !self.check_end_of_block() {
            body.push(self.parse_statement());
        }
        self.eat(Token::End);

        Stmt::Fn { name, params, body }
    }

    fn parse_return(&mut self) -> Stmt {
        self.eat(Token::Return);

        if matches!(
            self.current_token,
            Token::End
                | Token::Else
                | Token::ElseIf
                | Token::EOF
                | Token::Let
                | Token::Print
                | Token::If
                | Token::While
                | Token::Loop
                | Token::Break
                | Token::Continue
                | Token::Fn
                | Token::Return
        ) {
            Stmt::Return(Expr::Nil)
        } else {
            Stmt::Return(self.parse_expr())
        }
    }

    fn parse_arguments(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        if self.current_token != Token::RParen {
            loop {
                args.push(self.parse_expr());
                if self.current_token == Token::Comma {
                    self.eat(Token::Comma);
                } else {
                    break;
                }
            }
        }
        args
    }

    fn parse_while(&mut self) -> Stmt {
        self.eat(Token::While);
        let condition = self.parse_expr();
        self.eat(Token::Do);

        let mut body = Vec::new();
        while !self.check_end_of_block() {
            body.push(self.parse_statement());
        }
        self.eat(Token::End);

        Stmt::While { condition, body }
    }

    fn parse_loop(&mut self) -> Stmt {
        self.eat(Token::Loop);
        self.eat(Token::Do);

        let mut body = Vec::new();
        while !self.check_end_of_block() {
            body.push(self.parse_statement());
        }
        self.eat(Token::End);

        Stmt::Loop { body }
    }

    fn parse_if(&mut self) -> Stmt {
        self.eat(Token::If);
        let condition = self.parse_expr();
        self.eat(Token::Then);

        let mut then_branch = Vec::new();
        while !self.check_end_of_block() {
            then_branch.push(self.parse_statement());
        }

        let else_branch = if self.current_token == Token::ElseIf {
            self.eat(Token::ElseIf);
            let cond = self.parse_expr();
            self.eat(Token::Then);
            let mut branch = Vec::new();
            while !self.check_end_of_block() {
                branch.push(self.parse_statement());
            }

            let inner_else = if self.current_token == Token::ElseIf
                || self.current_token == Token::Else
            {
                match self.current_token {
                    Token::ElseIf => Some(vec![self.parse_recursive_elseif()]),
                    Token::Else => {
                        self.eat(Token::Else);
                        let mut stmts = Vec::new();
                        while self.current_token != Token::End && self.current_token != Token::EOF {
                            stmts.push(self.parse_statement());
                        }
                        self.eat(Token::End);
                        Some(stmts)
                    }
                    _ => unreachable!(),
                }
            } else {
                self.eat(Token::End);
                None
            };

            Some(vec![Stmt::If {
                condition: cond,
                then_branch: branch,
                else_branch: inner_else,
            }])
        } else if self.current_token == Token::Else {
            self.eat(Token::Else);
            let mut stmts = Vec::new();
            while self.current_token != Token::End && self.current_token != Token::EOF {
                stmts.push(self.parse_statement());
            }
            self.eat(Token::End);
            Some(stmts)
        } else {
            self.eat(Token::End);
            None
        };

        Stmt::If {
            condition,
            then_branch,
            else_branch,
        }
    }

    fn parse_recursive_elseif(&mut self) -> Stmt {
        self.eat(Token::ElseIf);
        let cond = self.parse_expr();
        self.eat(Token::Then);
        let mut branch = Vec::new();
        while !self.check_end_of_block() {
            branch.push(self.parse_statement());
        }

        let else_branch = if self.current_token == Token::ElseIf {
            Some(vec![self.parse_recursive_elseif()])
        } else if self.current_token == Token::Else {
            self.eat(Token::Else);
            let mut stmts = Vec::new();
            while self.current_token != Token::End && self.current_token != Token::EOF {
                stmts.push(self.parse_statement());
            }
            self.eat(Token::End);
            Some(stmts)
        } else {
            self.eat(Token::End);
            None
        };

        Stmt::If {
            condition: cond,
            then_branch: branch,
            else_branch,
        }
    }

    fn check_end_of_block(&self) -> bool {
        self.current_token == Token::End
            || self.current_token == Token::Else
            || self.current_token == Token::ElseIf
            || self.current_token == Token::EOF
    }

    fn parse_let(&mut self) -> Stmt {
        self.eat(Token::Let);
        let mutable = if self.current_token == Token::Mod {
            self.eat(Token::Mod);
            true
        } else {
            false
        };

        let name = match &self.current_token {
            Token::Identifier(name) => name.clone(),
            _ => panic!("Expected identifier after let"),
        };
        self.eat(Token::Identifier(String::new()));

        self.eat(Token::Equal);
        let value = self.parse_expr();

        Stmt::Let {
            name,
            mutable,
            value,
        }
    }

    fn parse_print(&mut self) -> Stmt {
        self.eat(Token::Print);
        self.eat(Token::LParen);
        let expr = self.parse_expr();
        self.eat(Token::RParen);
        Stmt::Print(expr)
    }

    fn parse_expr(&mut self) -> Expr {
        self.parse_logic_or()
    }

    fn parse_logic_or(&mut self) -> Expr {
        let mut left = self.parse_logic_and();
        while self.current_token == Token::Or {
            self.eat(Token::Or);
            let right = self.parse_logic_and();
            left = Expr::Binary(Box::new(left), Op::Or, Box::new(right));
        }
        left
    }

    fn parse_logic_and(&mut self) -> Expr {
        let mut left = self.parse_equality();
        while self.current_token == Token::And {
            self.eat(Token::And);
            let right = self.parse_equality();
            left = Expr::Binary(Box::new(left), Op::And, Box::new(right));
        }
        left
    }

    fn parse_equality(&mut self) -> Expr {
        let mut left = self.parse_relational();
        while self.current_token == Token::EqualEqual || self.current_token == Token::BangEqual {
            let op = match self.current_token {
                Token::EqualEqual => Op::Equal,
                Token::BangEqual => Op::NotEqual,
                _ => unreachable!(),
            };
            self.eat(self.current_token.clone());
            let right = self.parse_relational();
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        left
    }

    fn parse_relational(&mut self) -> Expr {
        let mut left = self.parse_term();
        while matches!(
            self.current_token,
            Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual
        ) {
            let op = match self.current_token {
                Token::Less => Op::Lt,
                Token::LessEqual => Op::LtEq,
                Token::Greater => Op::Gt,
                Token::GreaterEqual => Op::GtEq,
                _ => unreachable!(),
            };
            self.eat(self.current_token.clone());
            let right = self.parse_term();
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        left
    }

    fn parse_term(&mut self) -> Expr {
        let mut left = self.parse_factor();
        while self.current_token == Token::Plus || self.current_token == Token::Minus {
            let op = match self.current_token {
                Token::Plus => Op::Add,
                Token::Minus => Op::Sub,
                _ => unreachable!(),
            };
            self.eat(self.current_token.clone());
            let right = self.parse_factor();
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        left
    }

    fn parse_factor(&mut self) -> Expr {
        let mut left = self.parse_unary();
        while self.current_token == Token::Star
            || self.current_token == Token::Slash
            || self.current_token == Token::Percent
        {
            let op = match self.current_token {
                Token::Star => Op::Mul,
                Token::Slash => Op::Div,
                Token::Percent => Op::Mod,
                _ => unreachable!(),
            };
            self.eat(self.current_token.clone());
            let right = self.parse_unary();
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        left
    }

    fn parse_unary(&mut self) -> Expr {
        if self.current_token == Token::Not {
            self.eat(Token::Not);
            let expr = self.parse_unary();
            Expr::Unary(Op::Not, Box::new(expr))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Expr {
        match self.current_token.clone() {
            Token::Number(val) => {
                self.eat(Token::Number(0));
                Expr::Number(val)
            }
            Token::True => {
                self.eat(Token::True);
                Expr::Boolean(true)
            }
            Token::False => {
                self.eat(Token::False);
                Expr::Boolean(false)
            }
            Token::Nil => {
                self.eat(Token::Nil);
                Expr::Nil
            }
            Token::Identifier(name) => {
                self.eat(Token::Identifier(String::new()));

                if self.current_token == Token::LParen {
                    self.eat(Token::LParen);
                    let args = self.parse_arguments();
                    self.eat(Token::RParen);
                    Expr::Call(name, args)
                } else {
                    Expr::Variable(name)
                }
            }
            Token::LParen => {
                self.eat(Token::LParen);
                let expr = self.parse_expr();
                self.eat(Token::RParen);
                expr
            }
            _ => panic!("Unexpected token in expression: {:?}", self.current_token),
        }
    }
}
