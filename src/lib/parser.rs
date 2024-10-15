use core::panic;
use std::cmp::Ordering;

use crate::token::{LocToken, Precedences, Token, OPERATOR_MAP, OPERATOR_PRECEDENCES, Keyword};
use crate::lexer::Lexer;
use crate::ast::{match_type, ASTNode, ASTNodeType, PrimitiveTypes};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
    current_loc_token: LocToken,
    next_token: Token,
    next_loc_token: LocToken,
    indent_stack: Vec<usize>,
    file_name: String,
    // multi_line: bool,
}

impl<'a> Parser<'a> {

    // PUBLIC
    pub fn new(lexer: Lexer<'a>, file_name: String) -> Self {
        let mut parser = Parser {
            lexer,
            current_token: Token::EOF,
            current_loc_token: ((0,0), Token::EOF),
            next_token: Token::EOF,
            next_loc_token: ((0,0), Token::EOF),
            indent_stack: Vec::new(),
            file_name,
            // multi_line: false,
        };
        parser.advance(); // Load the first token
        parser.advance(); // Load the second token
        parser
    }

    fn panic_loc<T>(&self, msg: &str) -> T {
        let mut fmt: String = String::new();
        let ((row, col), _) = &self.current_loc_token;
        fmt.push_str(&self.file_name);
        fmt.push(':');
        fmt.push_str(&row.to_string());
        fmt.push(':');
        fmt.push_str(&col.to_string());
        fmt.push('\n');
        eprintln!("{}", fmt);
        panic!("{}", msg);
    }

    fn get_current_loc(&self) -> (usize, usize) {
        self.current_loc_token.0
    }

    fn advance(&mut self) {
        self.current_loc_token = self.next_loc_token.clone();
        self.next_loc_token = self.lexer.next_token();
        let (_, t) = &self.current_loc_token;
        self.current_token = t.clone();
        let (_, t) = &self.next_loc_token;
        self.next_token = t.clone();
    }

    fn increse_indention(&mut self) {
        let Token::Indent(indent) = self.current_token else {
            self.panic_loc("exprected indentinon!")
        };
        if self.indent_stack.is_empty() {
            self.indent_stack.push(indent);
            return;
        }
        let &last = self.indent_stack.last().expect("safe");
        if last < indent {
            self.indent_stack.push(indent);
        }
        else {
            self.panic_loc("Indention must be incresed!")
        }
    }

    pub fn parse(&mut self) -> Vec<ASTNode> {
        let mut nodes = Vec::new();

        while self.current_token != Token::EOF {
            if self.current_token == Token::Newline {
                self.advance();
                continue;
            }
            if !self.indent_stack.is_empty() {
                if let Token::Indent(new_indent) = self.current_token {
                    let &last = self.indent_stack.last().unwrap();
                    match new_indent.cmp(&last) {
                        Ordering::Equal => {
                            self.advance();
                        }
                        Ordering::Less => {
                            self.indent_stack.pop();
                            return nodes;
                        }
                        Ordering::Greater => {
                            self.panic_loc("Unexpeted indention!")
                        }
                    }
                }
                else {
                    self.indent_stack.clear();
                    return nodes;
                }
            }
            else if let Token::Indent(_) = self.current_token {
                self.panic_loc("Unexpeted indention!")
            }
            let node = self.parse_statement();
            nodes.push(node);
        }

        nodes
    }

    fn parse_statement(&mut self) -> ASTNode {
        while self.current_token == Token::Newline {
            self.advance();
        }
        let res: ASTNode = match &self.current_token {
            // empty line
            Token::Keyword(keyword) => match keyword {
                Keyword::Def => {
                    self.parse_function_def()
                }
                Keyword::Var => {
                    self.parse_declaration()
                }
                Keyword::If => {
                    self.parse_if_else()
                }
                Keyword::While => {
                    self.parse_while()
                }
                Keyword::True => {
                    self.parse_expression(Precedences::P0)
                }
                Keyword::False => {
                    self.parse_expression(Precedences::P0)
                }
                Keyword::Return => {
                    self.parse_return()
                }
                Keyword::Else => {
                    self.panic_loc("Unexpected 'else' keyword.")
                }
            }
            Token::Identifier(_) => {
                if Token::Assignment == self.next_token {
                    self.parse_assignment()
                }
                else {
                    self.parse_statement_expression()
                }
            },
            Token::Builtin(_) => self.parse_builtin(),
            _ => self.parse_statement_expression()
        };
        if self.current_token == Token::Newline {
            self.advance();
        }
        res
    }

    fn parse_function_def(&mut self) -> ASTNode {
        if Token::Keyword(Keyword::Def) != self.current_token{
            self.panic_loc("expected def keyword while parsing function definition.")
        }
        let loc = self.get_current_loc();
        self.advance(); // consume 'def'

        let Token::Identifier(name) = &self.current_token  else {
            self.panic_loc("Expected function name after 'def'")
        };
        let func_name = name.clone();
        self.advance(); // consume function name

        if Token::LParen != self.current_token{
            self.panic_loc("expected '(' afer function name while parsing function definition.")
        }
        self.advance(); // consume '('

        let mut args: Option<Vec<(String, PrimitiveTypes)>> = None;
        if Token::RParen != self.current_token {
            args = Some(self.parse_function_def_args());
        }
        self.advance(); // consume ')'

        let mut return_type: Option<PrimitiveTypes> = None;
        if let Token::Operator(op) = self.current_token.clone() {
            if op == "->" {
                self.advance(); // consume '->'
                let Token::Identifier(type_str) = self.current_token.clone() else {
                    self.panic_loc("Expected a type after '->' during function defenition")
                };
                let Some(found_type) = match_type(&type_str) else {
                    self.panic_loc(&format!("'{}' is not a valid type", type_str))
                };
                return_type = Some(found_type);
                self.advance(); // consume type
            }
            else {
                self.panic_loc(&format!("Unexpected operator '{}' found during dunciton definition", op))
            }
        }

        if Token::Colon != self.current_token{
            self.panic_loc("expected ':' while parsing function definition.")
        }
        self.advance(); // consume ':'
        if Token::Newline != self.current_token{
            self.panic_loc("expected newline '\\n' while parsing function definition.")
        }
        self.advance(); // consume '\n'

        if !self.indent_stack.is_empty() {
            self.panic_loc("functions can only be declared in the global scope.")
        }

        self.increse_indention();
        let body = self.parse();

        ASTNode {
            node_type: ASTNodeType::FunctionDef(func_name, args, return_type, body,),
             loc,
        }

    }

    fn parse_function_def_args(&mut self) -> Vec<(String, PrimitiveTypes)> {
        let mut args: Vec<(String, PrimitiveTypes)> = Vec::new();
        while self.current_token != Token::RParen {
            if self.current_token == Token::Newline {
                self.panic_loc("Newlines '\\n' are currenlty not allowed during the definition of function parameters")
            }
            if self.current_token == Token::EOF {
                self.panic_loc("Unexpected EOF while parsing function parameters")
            }

            let Token::Identifier(name) = self.current_token.clone() else {
                self.panic_loc("Expected identifier, got unexpected token at definition of funciton parameters")
            };
            self.advance();

            let Token::Colon = self.current_token.clone() else {
                self.panic_loc("Expected ':', got unexpected token at definition of funciton parameters")
            };
            self.advance();

            let Token::Identifier(type_str) = self.current_token.clone() else {
                self.panic_loc("Expected type identifier, got unexpected token at definition of funciton parameters")
            };
            self.advance();

            let Some(typ) = match_type(type_str.as_str()) else {
                self.panic_loc("Uknown type while declaring function parameters")
            };

            args.push((name, typ));

            if self.current_token == Token::Comma {
                if let Token::Identifier(_) = self.next_token {
                    self.advance();
                }
                else {
                    self.panic_loc("Expected identifier after ',' during function parameter definition")
                }
            }
        }
        args
    }

    fn parse_function_call(&mut self) -> ASTNode {
        let Token::Identifier(name) = self.current_token.clone() else {
            self.panic_loc("Expected identifier for function call.")
        };
        let loc = self.get_current_loc();
        self.advance();
        if Token::LParen != self.current_token{
            self.panic_loc("expected '(' afer function name while parsing function call.")
        }
        self.advance(); // consume '('

        let mut args = Vec::new();
        if Token::RParen != self.current_token{
            args = self.parse_function_call_args();
        }
        self.advance(); // consume ')'

        // {
        //     // println!("WARNING: function calls can not be used as expressions at the moment");
        //     // println!("    Treated as a statement instead. This needs to be changed itf");
        //     if Token::Newline != self.current_token && Token::EOF != self.current_token{
        //         self.panic_loc("expected newline '\\n' while parsing function call.")
        //     }
        // }

        ASTNode {
            node_type: ASTNodeType::FunctionCall(name.clone(), args, PrimitiveTypes::Void),
            loc
        }
    }

    fn parse_function_call_args(&mut self) -> Vec<ASTNode> {
        let mut args = Vec::new();
        while self.current_token != Token::RParen {
            if self.current_token == Token::Newline {
                self.panic_loc("Newlines '\\n' are currenlty not allowed for parameters of a function call")
            }
            if self.current_token == Token::EOF {
                self.panic_loc("Unexpected EOF while parsing function call parameters")
            }

            let expr = self.parse_expression(Precedences::P0);
            args.push(expr);

            if self.current_token == Token::Comma {
                if self.next_token == Token::RParen {
                    self.panic_loc("Expected an other expression after ',' during function call parameters")
                }
                self.advance();
            }
        }
        args
    }

    fn parse_declaration(&mut self) -> ASTNode {
        let loc = self.get_current_loc();
        if Token::Keyword(Keyword::Var) != self.current_token {
            self.panic_loc("Expected 'var' Keyword for declaration.")
        }
        self.advance();

        let Token::Identifier(name) = self.current_token.clone() else {
            self.panic_loc("Expected an identifier for declaration.")
        };
        self.advance();

        let Token::Colon = self.current_token.clone() else {
            self.panic_loc("Expected ':' after identifier for declaration.")
        };
        self.advance();

        let Token::Identifier(typ_str) = self.current_token.clone() else {
            self.panic_loc("Expected type (identefier) after identifier for declaration.")
        };
        self.advance();

        let Some(typ) = match_type(&typ_str) else {
            self.panic_loc(format!("Type with name '{}' does not exist", typ_str).as_str())
        };

        if self.current_token == Token::Assignment {
            self.advance();
            let expr = self.parse_expression(Precedences::P0);
            ASTNode {
                node_type: ASTNodeType::Declaration(name, typ, Some(Box::new(expr))),
                loc,
            }
        }
        else if self.current_token == Token::Newline {
            ASTNode {
                node_type: ASTNodeType::Declaration(name, typ, None),
                loc,
            }
        }
        else {
            self.panic_loc("Unexpeted Token after declaration.")
        }
    }

    fn parse_assignment(&mut self) -> ASTNode {
        let loc = self.get_current_loc();
        let Token::Identifier(var_name) = &self.current_token else {
            self.panic_loc("Expected identifier for assignment")
        };
        let name = var_name.clone();
        self.advance(); // consume variable name

        if self.current_token != Token::Assignment {
            self.panic_loc(format!("Expected '=' after identifier: {:#?}", self.current_loc_token).as_str())
        }
        self.advance(); // consume '='

        let value = self.parse_expression(Precedences::P0);
        ASTNode {
            node_type: ASTNodeType::Assignment(name, Box::new(value)),
            loc,
        }
    }

    fn parse_operant(&mut self) -> ASTNode {
        let loc = self.get_current_loc();
        match &self.current_token {
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression(Precedences::P0);
                if self.current_token != Token::RParen {
                    self.panic_loc("Expecred a ')' here!")
                }
                self.advance();
                expr
            }
            Token::RParen => {
                self.panic_loc("Did not exprect ')' here!")
            }
            Token::Identifier(s) => {
                if Token::LParen == self.next_token {
                    self.parse_function_call()
                }
                else {
                    let _s = s.clone();
                    self.advance();
                    ASTNode {
                        node_type: ASTNodeType::Identifier(_s, PrimitiveTypes::Void),
                        loc,
                    }
                }
            }
            Token::Float(n) => {
                let _n = n.clone();
                self.advance();
                ASTNode {
                    node_type: ASTNodeType::Literal(PrimitiveTypes::Float, _n),
                    loc,
                }
            }
            Token::Keyword(Keyword::True) => {
                self.advance();
                ASTNode {
                    node_type: ASTNodeType::Literal(PrimitiveTypes::Bool, "true".to_string()),
                    loc,
                }
            }
            Token::Keyword(Keyword::False) => {
                self.advance();
                ASTNode {
                    node_type: ASTNodeType::Literal(PrimitiveTypes::Bool, "false".to_string()),
                    loc,
                }
            }
            Token::Integer(n) => {
                let _n = n.clone();
                self.advance();
                ASTNode {
                    node_type: ASTNodeType::Literal(PrimitiveTypes::Number, _n),
                    loc,
                }
            }
            _ => {
                self.panic_loc(format!("Unexoected Token here: {:#?}", self.current_token).as_str())
            }
        }
    }

    fn parse_expression(&mut self, prec: Precedences) -> ASTNode {
        if prec >= Precedences::Count {
            return self.parse_operant();
        }

        let lhs = self.parse_expression(prec.increment());
        if self.current_token != Token::EOF && self.current_token != Token::Newline && self.current_token != Token::Colon {
            if let Token::Operator(ch) = self.current_token.clone() {
                let loc = self.get_current_loc();
                if OPERATOR_PRECEDENCES.get(&ch).expect("Wiil always be safe") == &prec {
                    self.advance();
                    let rhs = self.parse_expression(prec);

                    let Some(op) = OPERATOR_MAP.get(&ch) else {
                        self.panic_loc(format!("Found unsopported operator while buildine ASTNode: {}", ch).as_str())
                    };
                    return ASTNode {
                        node_type: ASTNodeType::BinaryOp(Box::new(lhs), op.clone(), Box::new(rhs), PrimitiveTypes::Void),
                        loc,
                    };
                }
            }
        }
        lhs
    }

    fn parse_statement_expression(&mut self) -> ASTNode {
        ASTNode {
            node_type: ASTNodeType::SExpression(Box::new(self.parse_expression(Precedences::P0))),
            loc: self.get_current_loc(),
        }
    }

    fn parse_builtin(&mut self) -> ASTNode {
        let Token::Builtin(name) = self.current_token.clone() else {
            self.panic_loc(format!("Tried to parse builtin token but found this: {:#?}", self.current_token).as_str())
        };
        let loc = self.get_current_loc();
        self.advance();
        if self.current_token != Token::LParen {
            self.panic_loc("expected '(' adter print.")
        }
        let expr = self.parse_expression(Precedences::P0);
        ASTNode {
            node_type: ASTNodeType::BuiltinFunction(name, Box::new(expr)),
            loc,
        }
    }

    fn parse_if_else(&mut self) ->ASTNode {
        // I relized know that i loose the location of the else keyword...
        let loc = self.get_current_loc();
        if Token::Keyword(Keyword::If) != self.current_token {
            self.panic_loc("Expected if token.")
        }
        self.advance();

        let cond = self.parse_expression(Precedences::P0);
        if Token::Colon != self.current_token {
            self.panic_loc("Expected ':' after the expression of the if condition.")
        }
        self.advance();

        if Token::Newline != self.current_token {
            self.panic_loc("Expected new line '\\n' after ':' for the if condition.")
        }
        self.advance();

        self.increse_indention();
        let then = self.parse();

        if Token::Keyword(Keyword::Else) != self.current_token {
            return ASTNode {
                node_type: ASTNodeType::If(Box::new(cond), then, None),
                loc,
            }
        }
        self.advance();

        if Token::Colon != self.current_token {
            self.panic_loc("Expected ':' after else keyword.")
        }
        self.advance();

        if Token::Newline != self.current_token {
            self.panic_loc("Expected newline '\\n' after ':' for else keyword.")
        }
        self.advance();

        // Indetion check
        if let Token::Indent(_) = self.current_token {
            self.increse_indention();
        }

        let els = self.parse();
        ASTNode {
            node_type: ASTNodeType::If(Box::new(cond), then, Some(els)),
            loc,
        }
    }

    fn parse_while(&mut self) -> ASTNode{
        let loc = self.get_current_loc();
        if Token::Keyword(Keyword::While) != self.current_token {
            self.panic_loc("Expected 'while' token here.")
        }
        self.advance();

        let cond = self.parse_expression(Precedences::P0);

        if Token::Colon != self.current_token {
            self.panic_loc("Expected ':' after while expression.")
        }
        self.advance();

        if Token::Newline != self.current_token {
            self.panic_loc("Expected newline '\\n' after : for while loop.")
        }
        self.advance();

        self.increse_indention();
        let body = self.parse();

        ASTNode {
            node_type: ASTNodeType::While(Box::new(cond), body),
            loc,
        }
    }

    fn parse_return(&mut self) -> ASTNode {
        if self.current_token != Token::Keyword(Keyword::Return) {
            self.panic_loc("Expected 'return' here.")
        }
        let loc = self.get_current_loc();
        self.advance();

        let mut expr = None;
        if self.current_token != Token::Newline || self.current_token != Token::EOF {
            expr = Some(Box::new(self.parse_expression(Precedences::P0)));
        }
        ASTNode {
            loc,
            node_type: ASTNodeType::Return(expr)
        }
    }
}
