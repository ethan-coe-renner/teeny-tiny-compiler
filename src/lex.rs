use strum::IntoEnumIterator; // 0.17.1
use strum_macros::EnumIter; // 0.17.1
use std::fmt;

pub struct Lexer {
    source: Vec<char>,
    pub cur_char: char,
    pub cur_pos: i64,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let mut new = Lexer {
            source: input.chars().collect::<Vec<char>>(),
            cur_char: '\0', // doesn't matter what this character is
            cur_pos: -1,
        };
        new.source.push('\n');
        new.next_char();
        new
    }

    fn next_char(&mut self) {
        self.cur_pos += 1;
        if self.cur_pos >= self.source.len() as i64 {
            self.cur_char = '\0'; // EOF
        } else {
            self.cur_char = self.source[self.cur_pos as usize];
        }
    }

    fn peek(&self) -> char {
        if self.cur_pos + 1 >= self.source.len() as i64 {
            '\0'
        } else {
            self.source[self.cur_pos as usize + 1]
        }
    }

    fn skip_whitespace(&mut self) {
        while self.cur_char == ' ' || self.cur_char == '\t' || self.cur_char == '\r' {
            self.next_char()
        }
    }

    fn skip_comment(&mut self) {
        if self.cur_char == '#' {
            while self.cur_char != '\n' {
                self.next_char();
            }
        }
    }

    pub fn get_token(&mut self) -> Token {
        self.skip_whitespace();
        self.skip_comment();
        let ret = match self.cur_char {
            '+' => Token::new(self.cur_char.to_string(), TokenType::PLUS),
            '-' => Token::new(self.cur_char.to_string(), TokenType::MINUS),
            '*' => Token::new(self.cur_char.to_string(), TokenType::ASTERISK),
            '/' => Token::new(self.cur_char.to_string(), TokenType::SLASH),

            '=' => {
                if self.peek() == '=' {
                    let last_char = self.cur_char;
                    self.next_char();
                    Token::new(
                        last_char.to_string() + &self.cur_char.to_string(),
                        TokenType::EQEQ,
                    )
                } else {
                    Token::new(self.cur_char.to_string(), TokenType::EQ)
                }
            }

            '>' => {
                if self.peek() == '=' {
                    let last_char = self.cur_char;
                    self.next_char();
                    Token::new(
                        last_char.to_string() + &self.cur_char.to_string(),
                        TokenType::GTEQ,
                    )
                } else {
                    Token::new(self.cur_char.to_string(), TokenType::GT)
                }
            }

            '<' => {
                if self.peek() == '=' {
                    let last_char = self.cur_char;
                    self.next_char();
                    Token::new(
                        last_char.to_string() + &self.cur_char.to_string(),
                        TokenType::LTEQ,
                    )
                } else {
                    Token::new(self.cur_char.to_string(), TokenType::LT)
                }
            }

            '!' => {
                if self.peek() == '=' {
                    let last_char = self.cur_char;
                    self.next_char();
                    Token::new(
                        last_char.to_string() + &self.cur_char.to_string(),
                        TokenType::NOTEQ,
                    )
                } else {
                    panic!("Expected !=, got !");
                }
            }

            '\"' => {
                self.next_char();
                let start_pos = self.cur_pos as usize;
                while self.cur_char != '\"' {
                    if self.cur_char == '\r'
                        || self.cur_char == '\n'
                        || self.cur_char == '\t'
                        || self.cur_char == '\\'
                        || self.cur_char == '%'
                    {
                        panic!("Illegal character in string")
                    }
                    self.next_char();
                }
                let tok_text: String = self
                    .source
                    .get(start_pos..self.cur_pos as usize + 1)
                    .unwrap()
                    .into_iter()
                    .collect();
                Token::new(tok_text, TokenType::STRING)
            }
	    digit if digit.is_ascii_digit() => {
		let start_pos = self.cur_pos as usize;
		while self.peek().is_ascii_digit() {
		    self.next_char();
		}
		if self.peek() == '.' {
		    self.next_char();
		    if !self.peek().is_ascii_digit() {
			panic!("Illegal character in number")
		    }
		    while self.peek().is_ascii_digit() {
			self.next_char();
		    }
		}

                let tok_text: String = self
                    .source
                    .get(start_pos..self.cur_pos as usize + 1)
                    .unwrap()
                    .into_iter()
                    .collect();
                Token::new(tok_text, TokenType::NUMBER)
	    }
	    alpha if alpha.is_alphabetic() => {
		let start_pos = self.cur_pos as usize;
		while self.peek().is_alphanumeric() {
		    self.next_char();
		}
		
                let tok_text: String = self
                    .source
                    .get(start_pos..self.cur_pos as usize + 1)
                    .unwrap()
                    .into_iter()
                    .collect();
		let keyword: Option<TokenType>= Token::check_if_keyword(&tok_text);
		match keyword {
		    Some(kind) => Token::new(tok_text, kind),
		    None => Token::new(tok_text,TokenType::IDENT)
		}
	    }
		
	    '\n' => Token::new(self.cur_char.to_string(), TokenType::NEWLINE),
            '\0' => Token::new(self.cur_char.to_string(), TokenType::EOF),
            _ => {
                panic!("Unknown Token: {}", self.cur_char)
            }
        };
        self.next_char();
        ret
    }
}

#[derive(Clone)]
pub struct Token {
    pub text: String,
    pub kind: TokenType,
}

impl Token {
    pub fn new(text: String, kind: TokenType) -> Self {
        Token { text, kind }
    }

    fn check_if_keyword(token_text: &String) -> Option<TokenType> {
	for kind in TokenType::iter() {
	    if kind.to_string() == token_text.to_owned() && kind as i32 >= 100 && (kind as i32) < 200 {
		return Some(kind);
	    }
	}
	None
    }
}

#[derive(PartialEq, Debug, EnumIter, Clone, Copy)]
pub enum TokenType {
    EOF = -1,
    NEWLINE = 0,
    NUMBER = 1,
    IDENT = 2,
    STRING = 3,
    // Keywords.
    LABEL = 101,
    GOTO = 102,
    PRINT = 103,
    INPUT = 104,
    LET = 105,
    IF = 106,
    THEN = 107,
    ENDIF = 108,
    WHILE = 109,
    REPEAT = 110,
    ENDWHILE = 111,
    // Operators.
    EQ = 201 ,
    PLUS = 202,
    MINUS = 203,
    ASTERISK = 204,
    SLASH = 205,
    EQEQ = 206,
    NOTEQ = 207,
    LT = 208,
    LTEQ = 209,
    GT = 210,
    GTEQ = 211,

}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
