use crate::lex::{Lexer, Token, TokenType};
use crate::emit::Emitter;
use std::collections::HashSet;

pub struct Parser {
    lexer: Lexer,
    emitter: Emitter,

    symbols: HashSet<String>,
    labels_declared: HashSet<String>,
    labels_gotoed: HashSet<String>,

    cur_token: Token,
    peek_token: Token,
}

impl Parser {
    pub fn new(lexer: Lexer, emitter: Emitter) -> Self {
        let mut parser = Parser {
            lexer,
	    emitter,

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
	self.emitter.header_line("#include <stdio.h>");
	self.emitter.header_line("int main(void){");

	    
        while self.check_token(TokenType::NEWLINE) {
	    self.next_token();
        }

        while !self.check_token(TokenType::EOF) {
            self.statement();
        }

	self.emitter.emit_line("return 0;");
	self.emitter.emit_line("}");
	

        for label in &self.labels_gotoed {
	    if !self.labels_declared.contains(label) {
                panic!("Attempting to GOTO to undeclared label: {}", label);
	    }
        }


	self.emitter.write_file(); // here instead of main to satisfy borrow checker
    }

    fn statement(&mut self) {
        // "PRINT" (expression | string)
        if self.check_token(TokenType::PRINT) {
            self.next_token();

            if self.check_token(TokenType::STRING) {
                self.emitter.emit_line(&format!("printf(\"{}\\n\");",self.cur_token.text ));

		self.next_token()
            } else {
                self.emitter.emit("printf(\"%.2f\\n\", (float)("); 
                self.expression();
		self.emitter.emit_line("));");
            }
        }
        //"IF" comparison "THEN" {statement} "ENDIF"
        else if self.check_token(TokenType::IF) {
            self.next_token();
	    self.emitter.emit("if(");
	    self.lexpression();

            self._match(TokenType::THEN);
            self.nl();
	    self.emitter.emit_line("){");

	    while !self.check_token(TokenType::ENDIF) {
                self.statement();
	    }

            self._match(TokenType::ENDIF);
	    self.emitter.emit_line("}");
        }
        // "WHILE" comparison "REPEAT" {statement} "ENDWHILE"
        else if self.check_token(TokenType::WHILE) {
            self.next_token();
	    self.emitter.emit("while(");
	    self.lexpression();

            self._match(TokenType::REPEAT);
            self.nl();
	    self.emitter.emit_line("){");

	    while !self.check_token(TokenType::ENDWHILE) {
                self.statement();
	    }

            self._match(TokenType::ENDWHILE);
	    self.emitter.emit_line("}");
        }
        // "LABEL" ident
        else if self.check_token(TokenType::LABEL) {
            self.next_token();

            // make sure label doesn't already exist
            if self.labels_declared.contains(&self.cur_token.text) {
                panic!("Label already exists: {}", self.cur_token.text)
            }
            self.labels_declared.insert(self.cur_token.text.clone());

	    self.emitter.emit_line(&format!("{}:", self.cur_token.text));
	    self._match(TokenType::IDENT);
        }
        // "GOTO" ident
        else if self.check_token(TokenType::GOTO) {
            self.next_token();
            self.labels_gotoed.insert(self.cur_token.text.clone());
	    self.emitter.emit_line(&format!("goto {}:", self.cur_token.text));
	    self._match(TokenType::IDENT);
        }
        // "LET" ident "=" expression
        else if self.check_token(TokenType::LET) {
            self.next_token();

            if !self.symbols.contains(&self.cur_token.text) {
                self.symbols.insert(self.cur_token.text.clone());
		self.emitter.header_line(&format!("float {};", self.cur_token.text));
            }

	    self.emitter.emit(&format!("{} = ", self.cur_token.text));
	    self._match(TokenType::IDENT);
            self._match(TokenType::EQ);

            self.expression();
	    self.emitter.emit_line(";");
        } else if self.check_token(TokenType::INPUT) {
            self.next_token();

            if !self.symbols.contains(&self.cur_token.text) {
                self.symbols.insert(self.cur_token.text.clone());
		self.emitter.header_line(&format!("float {};", self.cur_token.text));
            }

	    self.emitter.emit_line(&format!("if(0 == scanf(\"%f\", &{})) {{", self.cur_token.text));
	    self.emitter.emit_line(&format!("{} = 0;",self.cur_token.text));
	    self.emitter.emit("scanf(\"%");
	    self.emitter.emit_line("*s\");");
	    self.emitter.emit_line("}");
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
        self._match(TokenType::NEWLINE);

        while self.check_token(TokenType::NEWLINE) {
            self.next_token();
        }
    }

    // lexpression ::= comparison {("AND" | "OR") comparison}
    fn lexpression(&mut self) {
	self.comparison();

	while self.check_token(TokenType::AND) || self.check_token(TokenType::OR) {
	    if self.check_token(TokenType::AND) {
		self.emitter.emit("&&");
	    }
	    else {
		self.emitter.emit("||");
	    }
	    self.next_token();
	    self.comparison();
	}
    }

    // comparison ::= expression (("==" | "!=" | ">" | ">=" | "<" | "<=") expression)+
    fn comparison(&mut self) {
	if self.check_token(TokenType::NOT) {
	    self.emitter.emit("!");
	    self.next_token();
	}
	self.emitter.emit("(");
	self.expression();

	if self.is_comparison_operator() {
	    self.emitter.emit(&self.cur_token.text);
	    self.next_token();
	    self.expression();
	} else {
	    panic!("Expected comparison operator at: {}", self.cur_token.text)
	}

	while self.is_comparison_operator() {
	    self.emitter.emit(&self.cur_token.text);
	    self.next_token();
	    self.expression();
	}
	self.emitter.emit(")");
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
	self.term();

	while self.check_token(TokenType::PLUS) || self.check_token(TokenType::MINUS) {
	    self.emitter.emit(&self.cur_token.text);
	    self.next_token();
	    self.term();
	}
    }

    // term ::= unary {( "/" | "*" ) unary}
    fn term(&mut self) {
	self.unary();

	while self.check_token(TokenType::ASTERISK) || self.check_token(TokenType::SLASH) {
	    self.emitter.emit(&self.cur_token.text);
	    self.next_token();
	    self.unary();
	}
    }

    // unary ::= ["+" | "-"] primary
    fn unary(&mut self) {
	if self.check_token(TokenType::PLUS) || self.check_token(TokenType::MINUS) {
	    self.emitter.emit(&self.cur_token.text);
	    self.next_token();
	}
	self.primary();
    }

    // primary ::= number | ident
    fn primary(&mut self) {
	if self.check_token(TokenType::NUMBER) {
	    self.emitter.emit(&self.cur_token.text);
	    self.next_token();
	} else if self.check_token(TokenType::IDENT) {
	    if !self.symbols.contains(&self.cur_token.text) {
		panic!(
		    "Referencing variable before assignment: {}",
		    self.cur_token.text
		)
	    }
	    self.emitter.emit(&self.cur_token.text);
	    self.next_token();
	} else {
	    panic!("Unexpected token at {}", self.cur_token.text);
	}
    }
}
