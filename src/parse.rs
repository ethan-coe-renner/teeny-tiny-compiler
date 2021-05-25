use crate::lex::{Lexer, Token, TokenType};
use std::collections::HashSet;

pub struct Parser {
    lexer: Lexer,

    symbols: HashSet<String>,
    labels_declared: HashSet<String>,
    labels_gotoed: HashSet<String>,

    cur_token: Token,
    peek_token: Token,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        let mut parser = Parser {
            lexer,

            symbols: HashSet::new(),
            labels_declared: HashSet::new(),
            labels_gotoed: HashSet::new(),

            cur_token: Token::new(String::new(), TokenType::EOF), //null
            peek_token: Token::new(String::new(), TokenType::EOF), //null
        };
        parser.next_token();
        parser.next_token();
        parser
    }

    fn check_token(&self, kind: TokenType) -> bool {
        kind == self.cur_token.kind
    }

    fn check_peek(&self, kind: TokenType) -> bool {
        kind == self.peek_token.kind
    }

    fn _match(&mut self, kind: TokenType) {
        if !self.check_token(kind) {
            panic!("Expected {}, got {}", kind, self.cur_token.kind);
        }
        self.next_token()
    }

    fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.lexer.get_token();
    }

    // Production rules

    //program ::= {statement}
    pub fn program(&mut self) {
        println!("PROGRAM");

        while self.check_token(TokenType::NEWLINE) {
            self.next_token();
        }

        while !self.check_token(TokenType::EOF) {
            self.statement();
        }

        for label in &self.labels_gotoed {
            if !self.labels_declared.contains(label) {
                panic!("Attempting to GOTO to undeclared label: {}", label);
            }
        }
    }

    fn statement(&mut self) {
        // "PRINT" (expression | string)
        if self.check_token(TokenType::PRINT) {
            println!("STATEMENT-PRINT");
            self.next_token();

            if self.check_token(TokenType::STRING) {
                self.next_token()
            } else {
                self.expression();
            }
        }
        //"IF" comparison "THEN" {statement} "ENDIF"
        else if self.check_token(TokenType::IF) {
            println!("STATEMENT_IF");
            self.next_token();
            self.comparison();

            self._match(TokenType::THEN);
            self.nl();

            while !self.check_token(TokenType::ENDIF) {
                self.statement();
            }

            self._match(TokenType::ENDIF)
        }
        // "WHILE" comparison "REPEAT" {statement} "ENDWHILE"
        else if self.check_token(TokenType::WHILE) {
            println!("STATEMENT-WHILE");
            self.next_token();
            self.comparison();

            self._match(TokenType::REPEAT);
            self.nl();

            while !self.check_token(TokenType::ENDWHILE) {
                self.statement();
            }

            self._match(TokenType::ENDWHILE);
        }
        // "LABEL" ident
        else if self.check_token(TokenType::LABEL) {
            println!("STATEMENT-LABEL");
            self.next_token();

            // make sure label doesn't already exist
            if self.labels_declared.contains(&self.cur_token.text) {
                panic!("Label already exists: {}", self.cur_token.text)
            }
            self.labels_declared.insert(self.cur_token.text.clone());

            self._match(TokenType::IDENT);
        }
        // "GOTO" ident
        else if self.check_token(TokenType::GOTO) {
            println!("STATEMENT-GOTO");
            self.next_token();
            self.labels_gotoed.insert(self.cur_token.text.clone());
            self._match(TokenType::IDENT);
        }
        // "LET" ident "=" expression
        else if self.check_token(TokenType::LET) {
            println!("STATEMENT-LET");
            self.next_token();

            if !self.symbols.contains(&self.cur_token.text) {
                self.symbols.insert(self.cur_token.text.clone());
            }

            self._match(TokenType::IDENT);
            self._match(TokenType::EQ);

            self.expression();
        } else if self.check_token(TokenType::INPUT) {
            println!("STATEMENT-INPUT");
            self.next_token();

            if !self.symbols.contains(&self.cur_token.text) {
                self.symbols.insert(self.cur_token.text.clone());
            }

            self._match(TokenType::IDENT);
        } else {
            panic!(
                "Invalid statement at {} ({})",
                self.cur_token.text, self.cur_token.kind
            )
        }

        self.nl();
    }

    // nl ::= '\n'
    fn nl(&mut self) {
        println!("NEWLINE");

        self._match(TokenType::NEWLINE);

        while self.check_token(TokenType::NEWLINE) {
            self.next_token();
        }
    }

    // comparison ::= expression (("==" | "!=" | ">" | ">=" | "<" | "<=") expression)+
    fn comparison(&mut self) {
        println!("COMPARISON");

        self.expression();

        if self.is_comparison_operator() {
            self.next_token();
            self.expression();
        } else {
            panic!("Expected comparison operator at: {}", self.cur_token.text)
        }

        while self.is_comparison_operator() {
            self.next_token();
            self.expression();
        }
    }

    fn is_comparison_operator(&self) -> bool {
        self.check_token(TokenType::GT)
            || self.check_token(TokenType::GTEQ)
            || self.check_token(TokenType::LT)
            || self.check_token(TokenType::LTEQ)
            || self.check_token(TokenType::EQEQ)
            || self.check_token(TokenType::NOTEQ)
    }

    // expression ::= term {( "-" | "+" ) term}
    fn expression(&mut self) {
        println!("EXPRESSION");

        self.term();

        while self.check_token(TokenType::PLUS) || self.check_token(TokenType::MINUS) {
            self.next_token();
            self.term();
        }
    }

    // term ::= unary {( "/" | "*" ) unary}
    fn term(&mut self) {
        println!("TERM");

        self.unary();

        while self.check_token(TokenType::ASTERISK) || self.check_token(TokenType::SLASH) {
            self.next_token();
            self.unary();
        }
    }

    // unary ::= ["+" | "-"] primary
    fn unary(&mut self) {
        println!("UNARY");

        if self.check_token(TokenType::PLUS) || self.check_token(TokenType::MINUS) {
            self.next_token();
        }
        self.primary();
    }

    // primary ::= number | ident
    fn primary(&mut self) {
        println!("PRIMARY ({})", self.cur_token.text);

        if self.check_token(TokenType::NUMBER) {
            self.next_token();
        } else if self.check_token(TokenType::IDENT) {
            if !self.symbols.contains(&self.cur_token.text) {
                panic!(
                    "Referencing variable before assignment: {}",
                    self.cur_token.text
                )
            }
            self.next_token();
        } else {
            panic!("Unexpected token at {}", self.cur_token.text);
        }
    }
}
