use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::os::systemv::{SystemV, Parameter};
use crate::token::Operator;
use crate::operations::{Operation, OperationsType, Program};
use crate::ast::{ASTNode, ASTNodeType, PrimitiveTypes};


#[derive(Debug, Clone)]
enum VarriableType {
  Global(String, PrimitiveTypes),
  Parameter(Parameter),
}

#[derive(Debug, Clone)]
struct Scope {
  name: HashMap<Rc<String>, VarriableType>,
}

impl Scope {
  pub fn new() -> Self {
    Scope {
      name: HashMap::new(),
    }
  }
}

impl Deref for Scope {
    type Target = HashMap<Rc<String>, VarriableType>;

    fn deref(&self) -> &Self::Target {
        &self.name
    }
}

impl DerefMut for Scope {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.name
    }
}

#[derive(Debug, Clone)]
struct Scopes {
  scopes: Vec<Scope>,
}

impl Deref for Scopes {
    type Target = Vec<Scope>;

    fn deref(&self) -> &Self::Target {
        &self.scopes
    }
}

impl DerefMut for Scopes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scopes
    }
}

impl Scopes {
  pub fn new() -> Self {
    Scopes { scopes: Vec::new() }
  }

  // pub fn get_current_function(&self) -> Option<Rc<String>> {
  //   for scope in self.scopes.iter().rev() {
  //     if let ScopeType::Function(name) = &scope.scope_type {
  //       return Some(name.clone())
  //     }
  //   }
  //   None
  // }
}

#[derive(Debug, Clone)]
pub struct Builder {
  scopes: Scopes,
  functions: HashMap<Rc<String>, SystemV>,
  vars: Vec<String>,
  ref_count: usize,
  file_name: String,
}

impl Default for Builder {
  fn default() -> Self {
    Self::new(String::new())
  }
}

impl  Builder {
  pub fn new(file_name: String) -> Builder {
    Builder {
      scopes: Scopes::new(),
      functions: HashMap::new(),
      vars: Vec::new(),
      ref_count: 0,
      file_name,
    }
  }

  pub fn panic_loc<T>(&self, node: &ASTNode, msg: &str) -> T {
    let mut fmt: String = String::new();
    let (row, col) = node.get_loc();
    fmt.push_str(&self.file_name);
    fmt.push(':');
    fmt.push_str(&row.to_string());
    fmt.push(':');
    fmt.push_str(&col.to_string());
    fmt.push('\n');
    eprintln!("{}", fmt);
    panic!("{}", msg);
  }

  pub fn build_program(&mut self,ast: &mut Vec<ASTNode>) -> Program {
    let mut program: Program = Program::new();
    self.scan_nodes(ast);
    // self.translate_nodes(ast, &mut program, ScopeType::Root);
    self.translate_nodes(ast, &mut program);
    program
  }

  fn get_ref_number(&mut self) -> usize {
    let n = self.ref_count;
    self.ref_count += 1;
    n
  }

  fn delcare_global_var(&mut self,node: &ASTNode, name: &String, value_type: PrimitiveTypes) {
    if self.scopes.is_empty() {
      self.panic_loc(node, "Scope is emppty!")
    }
    for key in self.functions.keys() {
      if key.deref() == name {
        self.panic_loc(node, format!("var '{}' is trying to shadow a function. this is not allowed.", name).as_str())
      }
    }

    let var = VarriableType::Global(name.clone(), value_type.clone());
    self.scopes.last_mut().unwrap().insert(Rc::new(name.to_string()), var);
  }

  fn get_var(&self, name: &String) -> Option<VarriableType> {
    for scope in self.scopes.iter().rev() {
      if let Some(var) = scope.get(name) {
        return Some(var.clone());
      }
    }
    None
  }

  fn scan_nodes(&mut self, nodes: &Vec<ASTNode>) {
    // self.scopes.push(Scope { scope_type, name: HashMap::new() });
    for node in nodes {
      self.scan_node(node);
    }
    self.scopes.pop();
  }

  fn scan_node(&mut self, node: &ASTNode) {
    match node.node_type {
        ASTNodeType::FunctionDef(ref name, ref args, _) => {
          for key in self.functions.keys() {
            if key.deref() == name {
              self.panic_loc(node, format!("Duplicate function with name '{}'.", name).as_str())
            }
          }

          let fucn_name = Rc::new(name.clone());
          // self.scopes.push(Scope {scope_type: ScopeType::Function(fucn_name), name: HashMap::new()}); // scope for parameters
          // operations.function_defs.push(Operation::BeginFunction(name.clone()));
          let mut  parameters = SystemV::new();
          if let Some(args) = args {
            parameters.add_parameters(args);
            // let scope = self.scopes.last_mut().unwrap();
            // for (i, (arg_name, _)) in args.iter().enumerate() {
              // parameters.translate_save_arguments(i, operations);
              // let p = parameters.get(i).unwrap();
              // let arg_name = Rc::new(arg_name.clone());
              // scope.insert(arg_name, VarriableType::Parameter(p.clone()));
            // }
            // operations.function_defs.push(Operation::ReserveParameters(parameters.reserved_stack()));
          };
          self.functions.insert(fucn_name, parameters);
          // self.translate_nodes(body, operations);
          // operations.push(Operation::EndFunction());
          // self.scopes.pop(); // remove parameters
        },

        ASTNodeType::Declaration(_, _, _) |
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

  fn translate_nodes(&mut self, nodes: &Vec<ASTNode>, program: &mut Program) {
    // match scope_type {
    //     ScopeType::Root => program.cureent_target = OperationsType::Main,
    //     ScopeType::Function(_) => program.cureent_target = ,
    //     ScopeType::ControlFlow => {}
    // };
    self.scopes.push(Scope::new());
    for node in nodes {
      self.translate_node(node, program);
    }
    self.scopes.pop();
  }

  fn translate_node(&mut self, node: &ASTNode, program: &mut Program) {
    // let scope = self.scopes.last_mut().unwrap();
    // let mut operations = &mut self.scopes.last_mut().unwrap().target.clone();
    match node.node_type {
      ASTNodeType::Assignment(ref name, ref value) => {
        let Some(typ) = self.get_var(name) else {
          self.panic_loc(node, format!("'{}' was not declared!", name).as_str())
        };
        match typ {
          VarriableType::Global(name, typ) => {
            match typ {
              PrimitiveTypes::U64 => {
                self.translate_node(value, program);
                program.push(Operation::StoreInt(name));
              },
              PrimitiveTypes::F64 => {
                self.translate_node(value, program);
                program.push(Operation::StoreFloat(name));
              }
              _ => self.panic_loc(node, "Unexpected type!"),
            }
          }
          VarriableType::Parameter(p) => {
            self.translate_node(value, program);
            p.translate_store(program);
          }
        }
      }
      ASTNodeType::BinaryOp(ref left, ref op, ref right, ref typ) => {
        self.translate_node(left, program);
        self.translate_node(right, program);
        let left_t = left.get_type().unwrap();
        let right_t = left.get_type().unwrap();
        let operation = match (typ, op, left_t, right_t) {
            (PrimitiveTypes::U64,Operator::Plus, _, _) => Operation::AddInt,
            (PrimitiveTypes::U64,Operator::Minus, _, _) => Operation::MinusInt,
            (PrimitiveTypes::U64,Operator::Mul, _, _) => Operation::MultInt,
            (PrimitiveTypes::U64,Operator::Div, _, _) => Operation::DivInt,
            (PrimitiveTypes::U64, op, _, _) => self.panic_loc(node, format!("Type 'u64' is not defined for '{:#?}", op).as_str()),
            (PrimitiveTypes::F64, Operator::Plus, _, _) => Operation::AddFloat,
            (PrimitiveTypes::F64, Operator::Minus, _, _) => Operation::MinusFloat,
            (PrimitiveTypes::F64, Operator::Mul, _, _) => Operation::MultFloat,
            (PrimitiveTypes::F64, Operator::Div, _, _) => Operation::DivFloat,
            (PrimitiveTypes::F64, op, _, _) => self.panic_loc(node, format!("Type 'f64' is not defined for '{:#?}", op).as_str()),
            (PrimitiveTypes::Bool, Operator::And, PrimitiveTypes::Bool, PrimitiveTypes::Bool) => Operation::AndBool,
            (PrimitiveTypes::Bool, Operator::Or, PrimitiveTypes::Bool, PrimitiveTypes::Bool) => Operation::OrBool,
            (PrimitiveTypes::Bool, Operator::Less, PrimitiveTypes::U64, PrimitiveTypes::U64) => Operation::LessInt,
            (PrimitiveTypes::Bool, Operator::Equal, PrimitiveTypes::U64, PrimitiveTypes::U64) => Operation::EqualInt,
            (PrimitiveTypes::Bool, Operator::Greater, PrimitiveTypes::U64, PrimitiveTypes::U64) => Operation::GreaterInt,
            (PrimitiveTypes::Bool, op, _, _) => self.panic_loc(node, format!("Type 'bool' is not defined for '{:#?}", op).as_str()),

            // ambiguous types
            (PrimitiveTypes::Number, _, _, _) => self.panic_loc( node, "Ambigupus type 'Number'"),
            (PrimitiveTypes::Float,_, _, _) => self.panic_loc(node, "Ambigupus type 'Float'"),
            (PrimitiveTypes::Integer, _, _, _) => self.panic_loc(node, "Ambigupus type 'Integer'"),

            // invalid types
            (PrimitiveTypes::Void, _, _, _) => self.panic_loc(node, "Operations not defined for 'void'"),
            (PrimitiveTypes::COUNT,_, _, _) => self.panic_loc(node, "Invalid type at BinaryOp translation!"),
        };
        program.push(operation);
      }
      ASTNodeType::Literal(ref typ, ref symbols) => {
        match typ {
          PrimitiveTypes::U64 => program.push(Operation::PushInt(symbols.clone())),
          PrimitiveTypes::F64 => program.push(Operation::PushFloat(symbols.clone())),
          PrimitiveTypes::Bool => program.push(Operation::PushBool(symbols.clone())),

          PrimitiveTypes::Number |
          PrimitiveTypes::Float |
          PrimitiveTypes::Integer |
          PrimitiveTypes::Void |
          PrimitiveTypes::COUNT => {
            self.panic_loc(node, format!("Found unsupported Primitve Type in translate_node: {:#?}, {symbols}", typ).as_str())
          }
        }
      }
      ASTNodeType::Identifier(ref name, _) => {
        let Some(var_type) = self.get_var(name) else {
          self.panic_loc(node, format!("'{}' was not declared!", name).as_str())
        };
        match var_type {
          VarriableType::Global(name, value_type) => {
            match value_type {
              PrimitiveTypes::Bool |
              PrimitiveTypes::U64 => program.push(Operation::LoadInt(name)),
              PrimitiveTypes::F64 => program.push(Operation::LoadFloat(name)),

              PrimitiveTypes::Number |
              PrimitiveTypes::Float |
              PrimitiveTypes::Integer |
              PrimitiveTypes::Void |
              PrimitiveTypes::COUNT => self.panic_loc(node, "unexpected type"),
            }
          }
          VarriableType::Parameter(p) => {
            p.translate_load(program);
          }
        }
      }
      ASTNodeType::BuiltinFunction(ref name, ref expr) => {
        self.translate_node(expr, program);
        match name.as_str() {
          "print_int" => program.push(Operation::PrintInt),
          _ => self.panic_loc(node, format!("Unsupported builtin funcrion for translate_node: {}", name).as_str()),
        }
      }
      ASTNodeType::Declaration(ref name, ref value_type, ref expr) => {
        self.delcare_global_var(node, name, value_type.clone());
        self.vars.push(name.clone());
        program.vars.push(name.clone());
        match expr {
          None => {}
          Some(ref expr) => {
            self.translate_node(expr, program);
            program.push(Operation::StoreInt(name.clone()));
          }
        }
      }

      ASTNodeType::If(ref cond, ref then, ref els) => {
        self.translate_node(cond, program);
        let n = self.get_ref_number();
        program.push(Operation::If(n));
        self.translate_nodes(then, program);
        program.push(Operation::Else(n));
        if let Some(els) = els {
          self.translate_nodes(els, program);
        }
        program.push(Operation::EndIF(n));
      }

      ASTNodeType::While(ref cond, ref body) => {
        let n = self.get_ref_number();
        program.push(Operation::While(n));
        self.translate_node(cond, program);
        program.push(Operation::CondWhile(n));
        self.translate_nodes(body, program);
        program.push(Operation::EndWhile(n));
      }
      ASTNodeType::SExpression(ref expr) => {
        self.translate_node(expr, program);
        program.push(Operation::PopStack);
      }
      ASTNodeType::FunctionCall(ref name, ref args) => {
        let Some(def_args) = self.functions.get(name) else {
          self.panic_loc(node, format!("Tried to call function '{}' with '{}' arguments and was never defined!", name, args.len()).as_str())
        };
        let def_args = def_args.clone();
        if args.len() != def_args.len() {
          self.panic_loc(node, "Not the right amount of parameters")
        }
        for (i, expr) in args.iter().enumerate() {
          self.translate_node(expr, program);
          def_args.trnslate_caller_argument(i, program);
        }
        program.push(Operation::FunctionCall(name.clone(), 0));
      }
      ASTNodeType::FunctionDef( ref name, ref args, ref body ) => {
        if program.target != OperationsType::Main {
          self.panic_loc(node, "Can not define function inside a function")
        }

        let func_name = Rc::new(name.clone());
        program.target = OperationsType::Function(func_name);
        self.scopes.push(Scope {name: HashMap::new()}); // scope for parameters
        program.push(Operation::BeginFunction(name.clone()));
        let Some(parameters) = self.functions.get(name) else {
          self.panic_loc(node, &format!("Could not find function '{}' while building the program", name))
        };
        if let Some(args) = args {
          let scope = self.scopes.last_mut().unwrap();
          for (i, (arg_name, _)) in args.iter().enumerate() {
            parameters.translate_save_arguments(i, program);
            let p = parameters.get(i).unwrap();
            let arg_name = Rc::new(arg_name.clone());
            scope.insert(arg_name, VarriableType::Parameter(p.clone()));
          }
          program.push(Operation::ReserveParameters(parameters.reserved_stack()));
        };
        self.translate_nodes(body, program);
        program.push(Operation::EndFunction());
        program.target = OperationsType::Main;
        self.scopes.pop(); // remove parameters
      }
    }
  }
}
