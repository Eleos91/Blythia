use std::str::Chars;
use std::iter::Peekable;
use crate::token::{match_builtin_functions, match_keywords, LocToken, Token, OPERATOR_SYMBOLS};

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    last_token: Token,
    position: (usize, usize),
    consumption_length: usize,
    file_name: String,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str, file_name: String) -> Self {
        Lexer {
            input: input.chars().peekable(),
            last_token: Token::EOF,
            position: (1, 0),
            consumption_length: 0,
            file_name,
        }
    }

    fn increment_col(&mut self, n: usize) {
        let (row, col) = self.position;
        self.position = (row, col+n);
    }

    fn increment_row(&mut self) {
        let (row, _) = self.position;
        self.position = (row+1, 0);
    }

    fn find_next_token(&mut self) -> Token {
        if let Some(&ch) = self.input.peek() {
            self.last_token = match ch {
                ' ' | '\t' => {
                    let n = self.consume_whitespace();
                    if self.last_token == Token::Newline {
                        self.consumption_length += n;
                        Token::Indent(n)
                    } else {
                        self.increment_col(n);
                        self.find_next_token()
                    }
                }
                '\n' => {
                    self.input.next();
                    Token::Newline

                }
                '(' => {
                    self.input.next();
                    Token::LParen
                }
                ')' => {
                    self.input.next();
                    Token::RParen
                }
                '+' | '-' | '*' | '/' | '=' | '>' | '<' | '&' | '|' => self.consume_operator(),
                '0'..='9' => self.consume_number(),
                'a'..='z' | 'A'..='Z' | '_' => self.consume_identifier(),
                ',' => {
                    self.input.next();
                    Token::Comma
                }
                ':' => {
                    self.input.next();
                    Token::Colon
                }
                unhandled => {
                    self.input.next();
                    let (row, col) = self.position;
                    eprintln!("{}:{}:{}", self.file_name, row, col);
                    panic!("Can not deal with char '{}'", unhandled);
                }
            };
            return self.last_token.clone();
        }

        Token::EOF
    }

    pub fn next_token(&mut self) -> LocToken {
        if self.last_token == Token::Newline {
            self.increment_row();
        }
        self.consumption_length = 1;
        let t = self.find_next_token();
        let res = (self.position, t);
        self.increment_col(self.consumption_length);
        res
    }

    fn consume_whitespace(&mut self) -> usize {
        let mut count = 0;
        while let Some(&ch) = self.input.peek() {
            if ch == ' ' || ch == '\t' {
                count += 1;
                self.input.next();
            } else {
                return count;
            }
        };
        count
    }

    fn consume_number(&mut self) -> Token {
        let mut num_str = String::new();
        let mut saw_dot = false;
        while let Some(&ch) = self.input.peek() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.input.next();
            }
            else if ch == '.' {
                if saw_dot {
                    panic!("Error parsing number. Floats can't have more then one '.'");
                }
                saw_dot = true;
                num_str.push(ch);
                self.input.next();
            }
            else {
                break;
            }
        }
        self.consumption_length += num_str.len();
        if saw_dot {
            Token::Float(num_str)
        }
        else {
            Token::Integer(num_str)
        }
    }

    fn consume_identifier(&mut self) -> Token {
        let mut id_str = String::new();
        while let Some(&ch) = self.input.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                id_str.push(ch);
                self.input.next();
            } else {
                break;
            }
        }
        self.consumption_length += id_str.len();

        // match id_str.as_str() {
        //     "def" | "if" | "else" => Token::Keyword(id_str),
        //     "print"
        //     _ => Token::Identifier(id_str),
        // }
        if let Some(s) = match_keywords(&id_str) {
            Token::Keyword(s)
        }
        else if let Some(s) = match_builtin_functions(&id_str) {
            Token::Builtin(s)
        }
        else {
            Token::Identifier(id_str)
        }
    }

    fn consume_operator(&mut self) -> Token {
        let mut op_str = String::new();
        while let Some(&ch) = self.input.peek() {
            if OPERATOR_SYMBOLS.contains(&ch) {
                op_str.push(ch);
                self.input.next();
            }
            else {
                break
            }
        }
        self.consumption_length += op_str.len();
        if op_str.as_str() == "=" {
            Token::Assignment
        }
        else {
            Token::Operator(op_str)
        }
    }
}
