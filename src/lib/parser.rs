use core::panic;

use crate::token::{LocToken, Precedences, PrimitiveTypes, Token, OPERATOR_MAP, OPERATOR_PRECEDENCES, Keyword};
use crate::lexer::Lexer;
use crate::ast::ASTNode;

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
                let last = self.indent_stack.last().expect("safe");
                if self.current_token == Token::Indent(*last) {
                    self.advance();
                }
                else if self.current_token < Token::Indent(*last) {
                    self.indent_stack.pop();
                    return nodes;
                }
                else {
                    self.panic_loc("Unexpeted indention!")
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
                Keyword::Else => {
                    self.panic_loc("Unexpected 'else' keyword.")
                }
            }
            Token::Identifier(_) => {
                if Token::Assignment == self.next_token {
                    self.parse_assignment()
                }
                else if Token::LParen == self.next_token {
                   self.parse_function_call()
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

        let mut args: Option<Vec<String>> = None;
        if Token::RParen != self.current_token {
            args = Some(self.parse_function_def_args());
        }
            
        self.advance(); // consume ')'
        if Token::NewScope != self.current_token{
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

        ASTNode::FunctionDef {
            name: func_name,
            args,
            body,
        }
    }

    fn parse_function_def_args(&mut self) -> Vec<String> {
        let mut args = Vec::new();
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

            args.push(name);

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

        {
            println!("WARNING: function calls can not be used as expressions at the moment\n");
            println!("    Treated as a statement instead. This needs to be changed itf");
            if Token::Newline != self.current_token && Token::EOF != self.current_token{
                self.panic_loc("expected newline '\\n' while parsing function call.")
            }
        }
        
        ASTNode::FunctionCall(name.clone(), args)
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
        if Token::Keyword(Keyword::Var) != self.current_token {
            self.panic_loc("Expected 'var' Keyword for declaration.")
        }
        self.advance();

        let Token::Identifier(name) = self.current_token.clone() else {
            self.panic_loc("Expected an identifier for declaration.")
        };
        self.advance();

        if self.current_token == Token::Assignment {
            self.advance();
            let expr = self.parse_expression(Precedences::P0);
            ASTNode::Declaration(name, Some(Box::new(expr)))
        }
        else if self.current_token == Token::Newline {
            ASTNode::Declaration(name, None)
        }
        else {
            self.panic_loc("Unexpeted Token after declaration.")
        }
    }

    fn parse_assignment(&mut self) -> ASTNode {
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
        ASTNode::Assignment {
            name,
            value: Box::new(value),
        }
    }

    fn parse_operant(&mut self) -> ASTNode {
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
                let _s = s.clone();
                self.advance();
                ASTNode::Identifier(_s)
            }
            Token::Float(n) => {
                let _n = n.clone();
                self.advance();
                ASTNode::Literal { typ: PrimitiveTypes::Float, symbols: _n }
            }
            Token::Integer(n) => {
                let _n = n.clone();
                self.advance();
                ASTNode::Literal { typ: PrimitiveTypes::Integer, symbols: _n }
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
        if self.current_token != Token::EOF && self.current_token != Token::Newline && self.current_token != Token::NewScope {
            if let Token::Operator(ch) = self.current_token.clone() {
                if OPERATOR_PRECEDENCES.get(&ch).expect("Wiil always be safe") == &prec {
                    self.advance();
                    let rhs = self.parse_expression(prec);

                    let l_type: PrimitiveTypes = match &lhs {
                        ASTNode::Literal { typ, symbols: _ } => typ.clone(),
                        ASTNode::BinaryOp { left: _, op: _, right: _, typ } => typ.clone(),
                        ASTNode::Identifier(_) => PrimitiveTypes::Integer,
                        _ => self.panic_loc("Unexpecred lhs for op."),
                    };
                    let r_type: PrimitiveTypes = match &rhs {
                        ASTNode::Literal { typ, symbols: _ } => typ.clone(),
                        ASTNode::BinaryOp { left: _, op: _, right: _, typ } => typ.clone(),
                        ASTNode::Identifier(_) => PrimitiveTypes::Integer,
                        _ => self.panic_loc("Unexpecred lhs for op."),
                    };
                    if l_type != r_type {
                        self.panic_loc("Types of oerants don't match!")
                    }
                    if let Some(op) = OPERATOR_MAP.get(&ch) {
                        return ASTNode::BinaryOp { left: Box::new(lhs), op: op.clone(), right: Box::new(rhs), typ: l_type };
                    }
                    else {
                        self.panic_loc(format!("Found unsopported operator while buildine ASTNode: {}", ch).as_str())
                    }
                }
            }
        }
        lhs
    }

    fn parse_statement_expression(&mut self) -> ASTNode {
        ASTNode::SExpression(Box::new(self.parse_expression(Precedences::P0)))
    }

    fn parse_builtin(&mut self) -> ASTNode {
        let Token::Builtin(name) = self.current_token.clone() else {
            self.panic_loc(format!("Tried to parse builtin token but found this: {:#?}", self.current_token).as_str())
        };
        self.advance();
        if self.current_token != Token::LParen {
            self.panic_loc("expected '(' adter print.")
        }
        let expr = self.parse_expression(Precedences::P0);
        ASTNode::BuiltinFunction(name, Box::new(expr))
    }

    fn parse_if_else(&mut self) ->ASTNode {
        if Token::Keyword(Keyword::If) != self.current_token {
            self.panic_loc("Expected if token.")
        }
        self.advance();

        let cond = self.parse_expression(Precedences::P0);
        if Token::NewScope != self.current_token {
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
            return ASTNode::If(Box::new(cond), then, None);
        }
        self.advance();

        if Token::NewScope != self.current_token {
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
        ASTNode::If(Box::new(cond), then, Some(els))
    }

    fn parse_while(&mut self) -> ASTNode{
        if Token::Keyword(Keyword::While) != self.current_token {
            self.panic_loc("Expected 'while' token here.")
        }
        self.advance();

        let cond = self.parse_expression(Precedences::P0);

        if Token::NewScope != self.current_token {
            self.panic_loc("Expected ':' after while expression.")
        }
        self.advance();

        if Token::Newline != self.current_token {
            self.panic_loc("Expected newline '\\n' after : for while loop.")
        }
        self.advance();

        self.increse_indention();
        let body = self.parse();

        ASTNode::While(Box::new(cond), body)
    }
}
