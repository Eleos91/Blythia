use std::collections::HashMap;

use crate::scopes::systemv::{SystemV, Parameter};
use crate::token::Operator;
use crate::operations::{Operation, Program};
use crate::ast::{ASTNode, ASTNodeType, PrimitiveTypes};


#[derive(Debug, Clone)]
enum ParameterType {
  Integer(usize),
}

#[derive(Debug, Clone)]
enum VarriableType {
  Global(String, PrimitiveTypes),
  Parameter(Parameter),
}

#[derive(Debug, Clone)]
pub struct Builder {
  scopes: Vec<HashMap<String, VarriableType>>,
  functions: HashMap<String, SystemV>,
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

  pub fn build_program(&mut self,ast: &mut Vec<ASTNode>) -> Program {
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

  fn get_ref_number(&mut self) -> usize {
    let n = self.ref_count;
    self.ref_count += 1;
    n
  }

  fn delcare_global_var(&mut self, name: &String, value_type: PrimitiveTypes) {
    if self.scopes.is_empty() {
      panic!("Scopes are empty!!!")
    }
    for key in self.functions.keys() {
      if key == name {
        panic!("var '{}' is trying to shadow a function. this is not allowed.", name)
      }
    }

    let var = VarriableType::Global(name.clone(), value_type.clone());
    self.scopes.last_mut().unwrap().insert(name.clone(), var.clone());
  }

  fn get_var(&self, name: &String) -> Option<VarriableType> {
    for scope in self.scopes.iter().rev() {
      if let Some(var) = scope.get(name) {
        return Some(var.clone());
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
    match node.node_type {
        ASTNodeType::FunctionDef(ref name, ref args, ref body) => {
          for key in self.functions.keys() {
            if key == name {
              panic!{"Duplicate function with name '{}'.", name}
            }
          }

          self.scopes.push(HashMap::new()); // scope for parameters
          operations.push(Operation::BeginFunction(name.clone()));
          let mut  parameters = SystemV::new();
          if let Some(args) = args {
            parameters.add_parameters(args);
            let scope = self.scopes.last_mut().unwrap();
            for (i, (arg_name, arg_type)) in args.iter().enumerate() {
              parameters.translate_save_arguments(i, operations);
              let p = parameters.get(i).unwrap();
              scope.insert(arg_name.clone(), VarriableType::Parameter(p.clone()));
            }
            operations.push(Operation::ReserveParameters(parameters.reserved_stack()));
          };
          self.functions.insert(name.clone(), parameters);
          self.translate_nodes(body, operations);
          operations.push(Operation::EndFunction());
          self.scopes.pop(); // remove parameters
        },
        ASTNodeType::Declaration(ref name, ref typ, _) => {
          self.delcare_global_var(name, typ.clone());
        },

        ASTNodeType::Assignment(_, _) |
        ASTNodeType::BinaryOp(_, _, _, _) |
        ASTNodeType::Literal(_, _) |
        ASTNodeType::Identifier(_, _) |
        ASTNodeType::BuiltinFunction(_, _) |
        ASTNodeType::If(_, _, _) |
        ASTNodeType::While(_, _) |
        ASTNodeType::SExpression(_) |
        ASTNodeType::FunctionCall(_, _) => {},
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

    match node.node_type {
      ASTNodeType::Assignment(ref name, ref value) => {
        let Some(typ) = self.get_var(name) else {
          panic!("'{}' was not declared!", name);
        };
        match typ {
          VarriableType::Global(name, typ) => {
            match typ {
              PrimitiveTypes::U64 => {
                self.translate_node(value, operations);
                operations.push(Operation::StoreInt(name));
              },
              PrimitiveTypes::F64 => {
                self.translate_node(value, operations);
                operations.push(Operation::StoreFloat(name));
              }
              _ => panic!("Unexpected type!"),
            }
          }
          VarriableType::Parameter(p) => {
            self.translate_node(value, operations);
            p.translate_store(operations);
          }
        }
      }
      ASTNodeType::BinaryOp(ref left, ref op, ref right, ref typ) => {
        self.translate_node(left, operations);
        self.translate_node(right, operations);
        let operation = match (typ, op) {
            (PrimitiveTypes::U64,Operator::Plus) => Operation::AddInt,
            (PrimitiveTypes::U64,Operator::Minus) => Operation::MinusInt,
            (PrimitiveTypes::U64,Operator::Mul) => Operation::MultInt,
            (PrimitiveTypes::U64,Operator::Div) => Operation::DivInt,
            (PrimitiveTypes::U64,Operator::Equal) => Operation::EqualInt,
            (PrimitiveTypes::U64,Operator::Greater) => Operation::GreaterInt,
            (PrimitiveTypes::U64,Operator::Less) => Operation::LessInt,
            (PrimitiveTypes::F64, Operator::Plus) => Operation::AddFloat,
            (PrimitiveTypes::F64, Operator::Minus) => Operation::MinusFloat,
            (PrimitiveTypes::F64, Operator::Mul) => Operation::MultFloat,
            (PrimitiveTypes::F64, Operator::Div) => Operation::DivFloat,
            (PrimitiveTypes::F64, _) => todo!(),

            // ambiguous types
            (PrimitiveTypes::Number, _) => panic!("Ambigupus type 'Number'"),
            (PrimitiveTypes::Float,_) => todo!("Ambigupus type 'Float'"),
            (PrimitiveTypes::Integer, _) => todo!("Ambigupus type 'Integer'"),

            // invalid types
            (PrimitiveTypes::Void, _) => panic!("Operations not defined for 'void'"),
            (PrimitiveTypes::COUNT,_) => panic!("Invalid type at BinaryOp translation!"),
        };
        operations.push(operation);
      }
      ASTNodeType::Literal(ref typ, ref symbols) => {
        match typ {
          PrimitiveTypes::U64 => operations.push(Operation::PushInt(symbols.clone())),
          PrimitiveTypes::F64 => operations.push(Operation::PushFloat(symbols.clone())),
          _ => {
            panic!("Found unsupported Primitve Type in translate_node: {:#?}, {symbols}", typ)
          }
        }
      }
      ASTNodeType::Identifier(ref name, _) => {
        let Some(var_type) = self.get_var(name) else {
          panic!("'{}' was not declared!", name)
        };
        match var_type {
          VarriableType::Global(name, value_type) => {
            match value_type {
                PrimitiveTypes::U64 => operations.push(Operation::LoadInt(name)),
                PrimitiveTypes::F64 => operations.push(Operation::LoadFloat(name)),
                _ => panic!("unexpected type")
            }
          }
          VarriableType::Parameter(p) => {
            p.translate_load(operations);
          }
        }
      }
      ASTNodeType::BuiltinFunction(ref name, ref expr) => {
        self.translate_node(expr, operations);
        match name.as_str() {
          "print_int" => operations.push(Operation::PrintInt),
          _ => panic!("Unsupported builtin funcrion for translate_node: {}", name),
        }
      }
      ASTNodeType::Declaration(ref name, ref value_type, ref expr) => {
        self.delcare_global_var(name, value_type.clone());
        self.vars.push(name.clone());
        match expr {
          None => {}
          Some(ref expr) => {
            self.translate_node(expr, operations);
            operations.push(Operation::StoreInt(name.clone()));
          }
        }
      }

      ASTNodeType::If(ref cond, ref then, ref els) => {
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

      ASTNodeType::While(ref cond, ref body) => {
        let n = self.get_ref_number();
        operations.push(Operation::While(n));
        self.translate_node(cond, operations);
        operations.push(Operation::CondWhile(n));
        self.translate_nodes(body, operations);
        operations.push(Operation::EndWhile(n));
      }
      ASTNodeType::SExpression(ref expr) => {
        self.translate_node(expr, operations);
        operations.push(Operation::PopStack);
      }
      ASTNodeType::FunctionCall(ref name, ref args) => {
        let Some(def_args) = self.functions.get(name) else {
          panic!("Tried to call function '{}' with '{}' arguments and was never defined!", name, args.len());
        };
        let def_args = def_args.clone();
        if args.len() != def_args.len() {
          panic!("Not the right amount of parameters")
        }
        for (i, expr) in args.iter().enumerate() {
          self.translate_node(expr, operations);
          def_args.trnslate_caller_argument(i, operations);
        }
        operations.push(Operation::FunctionCall(name.clone(), 0));
      }
      ASTNodeType::FunctionDef( _, _, _ ) => {} //handled by scan
    }
  }
}
