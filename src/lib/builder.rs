use std::collections::HashMap;

use crate::token::{PrimitiveTypes, Operator};
use crate::operations::{Operation, Program};
use crate::ast::ASTNode;


#[derive(Debug, Clone)]
enum ParameterType {
  Integer(usize),
}

#[derive(Debug, Clone)]
enum VarriableType {
  Global(String, usize),
  Parameter(ParameterType),
}

#[derive(Debug, Clone)]
pub struct Builder {
  scopes: Vec<HashMap<String, Vec<VarriableType>>>,
  functions: HashMap<String, Vec<ParameterType>>,
  vars: Vec<String>,
  ref_count: usize,
}

impl Default for Builder {
  fn default() -> Self {
    Self::new()
  }
}

impl  Builder {
  pub fn new() -> Builder {
    Builder {
      scopes: Vec::new(),
      functions: HashMap::new(),
      vars: Vec::new(),
      ref_count: 0,
    }
  }

  pub fn build_program(&mut self, ast: &Vec<ASTNode>) -> Program {
    let mut function_defs: Vec<Operation> = Vec::new();
    self.scan_nodes(ast, &mut function_defs);
    let mut main: Vec<Operation> = Vec::new();
    self.translate_nodes(ast, &mut main);
    let vars: Vec<String> = self.vars.clone();
    Program {
      function_defs,
      main,
      vars,
    }
  }

  fn delcare_var(&mut self, name: &String, typ: VarriableType) -> VarriableType {
    if self.scopes.is_empty() {
      panic!("Scopes are empty!!!")
    }
    for (key, _) in &self.functions {
      if key == name {
        panic!("var '{}' is trying to shadow a function. this is not allowed.", name)
      }
    }

    if let VarriableType::Parameter(n) = &typ {
      if let Some(_) = self.scopes.last().unwrap().get(name) {
        panic!("Expected scope to be empty for parameter!")
      }
      self.scopes.last_mut().unwrap().insert(name.clone(), vec![typ.clone()]);
      return typ;
    }

    let mut n = None;
    let mut arr: Vec<VarriableType> = Vec::new();
    for scope in self.scopes.iter().rev() {
      if let Some(refs) = scope.get(name) {
        for var in refs.iter().rev() {
          if let VarriableType::Global(_, k) = var {
            // n = Some(*refs.last().unwrap());
            n = Some(*k);
          }
        }
      }
    }
    let last_scope = self.scopes.last_mut().unwrap();
    if let Some(n) = n {
      let typ = VarriableType::Global(format!("{}_{}", name, n + 1), n + 1);  
      if let Some(arr) = last_scope.get_mut(name) {
        arr.push(typ.clone());
        typ
      }
      else {
        arr.push(typ.clone());
        last_scope.insert(name.clone(), arr);
        typ
      }
    }
    else {
      let typ = VarriableType::Global(format!("{}_{}", name, 0), 0);
      arr.push(typ.clone());
      last_scope.insert(name.clone(), arr);
      typ
    }
  }

  fn get_var(&self, name: &String) -> Option<VarriableType> {
    for scope in self.scopes.iter().rev() {
      if let Some(arr) = scope.get(name) {
        if let Some(n) = arr.last() {
          return match n {
            VarriableType::Global(_, _) => Some(n.clone()),
            VarriableType::Parameter(ParameterType::Integer(_)) => Some(n.clone()),
          };
        }
        panic!("Empty scope array for '{}'", name);
      }
    }
    None
  }

  fn scan_nodes(&mut self, nodes: &Vec<ASTNode>, operations: &mut Vec<Operation>) {
    self.scopes.push(HashMap::new());
    for node in nodes {
      self.scan_node(node, operations);
    }
    self.scopes.pop();
  }

  fn scan_node(&mut self, node: &ASTNode, operations: &mut Vec<Operation>) {
    match node {
        ASTNode::FunctionDef { name, args, body} => {
          for (key, _) in &self.functions {
            if key == name {
              panic!{"Duplicate function with name '{}'.", name}
            }
          }

          self.scopes.push(HashMap::new()); // scope for parameters
          operations.push(Operation::BeginFunction(name.clone()));
          let mut parameters: Vec<ParameterType> = Vec::new();
          if let Some(args) = args {
            let scope = self.scopes.last_mut().unwrap();
            for (i, arg) in args.iter().enumerate() {
              scope.insert(arg.clone(), vec![VarriableType::Parameter(ParameterType::Integer(i))]);
              parameters.push({ParameterType::Integer(i)});
              if i < 6 {
                operations.push(Operation::ArgumentIntegerLoad(i));
              }
            }
            operations.push(Operation::ReserveParameters(args.len()));
          };
          self.functions.insert(name.clone(), parameters);
          self.translate_nodes(body, operations);
          operations.push(Operation::EndFunction());
          self.scopes.pop(); // remove parameters
        },
        ASTNode::Assignment { name: _, value: _ } => {},
        ASTNode::BinaryOp { left: _, op: _, right: _, typ: _ } => {},
        ASTNode::Literal { typ: _, symbols: _ } => {},
        ASTNode::Identifier(_) => {},
        ASTNode::BuiltinFunction(_, _) => {},
        ASTNode::Declaration(name, _) => {
          self.delcare_var(name, VarriableType::Global(name.clone(), 0));
        },
        ASTNode::If(_, _, _) => {},
        ASTNode::While(_, _) => {},
        ASTNode::SExpression(_) => {},
        ASTNode::FunctionCall(_, _) => {},
    }
  }

  fn translate_nodes(&mut self, nodes: &Vec<ASTNode>, operations: &mut Vec<Operation>) {
    self.scopes.push(HashMap::new());
    for node in nodes {
      self.translate_node(node, operations);
    }
    self.scopes.pop();
  }

  fn translate_node(&mut self, node: &ASTNode, operations: &mut Vec<Operation>) {

    match node {
      ASTNode::Assignment { name, value } => {
        let Some(typ) = self.get_var(name) else {
          panic!("'{}' was not declared!", name);
        };
        match typ {
          VarriableType::Global(name, _) => {
            self.translate_node(value, operations);
            operations.push(Operation::StoreInt(name));
          }
          VarriableType::Parameter(ParameterType::Integer(n)) => {
            self.translate_node(value, operations);
            operations.push(Operation::ParameterIntegerStore(n));
          }
        }
      }
      ASTNode::BinaryOp { left, op, right, typ } => {
        self.translate_node(left, operations);
        self.translate_node(right, operations);
        let operation = match (op, typ) {
            (Operator::Plus, PrimitiveTypes::Integer) => Operation::AddInt,
            (Operator::Minus, PrimitiveTypes::Integer) => Operation::MinusInt,
            (Operator::Mul, PrimitiveTypes::Integer) => Operation::MultInt,
            (Operator::Div, PrimitiveTypes::Integer) => Operation::DivInt,
            (Operator::Equal, PrimitiveTypes::Integer) => Operation::EqualInt,
            (Operator::Greater, PrimitiveTypes::Integer) => Operation::GreaterInt,
            (Operator::Less, PrimitiveTypes::Integer) => Operation::LessInt,
            (_, PrimitiveTypes::Float) => todo!(),
            (_, PrimitiveTypes::COUNT) => panic!("Invalid type at BinaryOp translation!"),
        };
        operations.push(operation);
      }
      ASTNode::Literal { typ, symbols } => { 
        match typ {
          PrimitiveTypes::Integer => operations.push(Operation::PushInt(symbols.clone())),
          // PrimitiveTypes::Float => operations.push(Operation::PushFloat(symbols.clone())),
          _ => panic!("Found unsupported Primitve Type in translate_node: {:#?}", typ),
        }
      }
      ASTNode::Identifier(name) => {
        let Some(typ) = self.get_var(name) else {
          panic!("'{}' was not declared!", name)
        };
        match typ {
          VarriableType::Global(name, _) => {
            operations.push(Operation::LoadInt(name));
          }
          VarriableType::Parameter(ParameterType::Integer(n)) => {
            operations.push(Operation::ParameterIntegerLoad(n));
          }
        }
      }
      ASTNode::BuiltinFunction(name, expr) => {
        self.translate_node(expr, operations);
        match name.as_str() {
          "print_int" => operations.push(Operation::PrintInt),
          _ => panic!("Unsupported builtin funcrion for translate_node: {}", name),
        }
      }
      ASTNode::Declaration(name, expr) => {
        let typ = self.delcare_var(name, VarriableType::Global(name.clone(), 0));
        if let VarriableType::Global(name, _) =  typ {
          self.vars.push(name.clone());
          match expr {
            None => {}
            Some(expr) => {
              self.translate_node(expr, operations);
              operations.push(Operation::StoreInt(name));
            }
          }
        }
        else {
          panic!("Declared global variable 'name', but got {:#?}", typ);
        }
      }

      ASTNode::If(cond, then, els) => {
        self.translate_node(cond, operations);
        let n = self.get_ref_number();
        operations.push(Operation::If(n));
        self.translate_nodes(then, operations);
        operations.push(Operation::Else(n));
        if let Some(els) = els {
          self.translate_nodes(els, operations);
        }
        operations.push(Operation::EndIF(n));
      }

      ASTNode::While(cond, body) => {
        let n = self.get_ref_number();
        operations.push(Operation::While(n));
        self.translate_node(cond, operations);
        operations.push(Operation::CondWhile(n));
        self.translate_nodes(body, operations);
        operations.push(Operation::EndWhile(n));
      }
      ASTNode::SExpression(expr) => {
        self.translate_node(expr, operations);
        operations.push(Operation::PopStack);
      }
      ASTNode::FunctionCall(name, args) => {
        let Some(def_args) = self.functions.get(name) else {
          panic!("Tried to call function '{}' with '{}' arguments and was never defined!", name, args.len());
        };
        if args.len() != def_args.len() {
          panic!("Not enough parameters")
        }
        for (i, expr) in args.iter().enumerate() {
          if i >= 6 {
            break
          }
            self.translate_node(expr, operations);
            operations.push(Operation::ArgumentIntegerStore(i));
        }
        if let Some(args) = args.get(6..) {
          for arg in args.iter().rev() {
            self.translate_node(arg, operations);
          }
        }
        operations.push(Operation::FunctionCall(name.clone(), 0));
      }
      ASTNode::FunctionDef { name: _, args: _, body: _ } => {} //handled by scan
    }
}

fn get_ref_number(&mut self) -> usize {
        let n = self.ref_count;
        self.ref_count += 1;
        n
    }

}