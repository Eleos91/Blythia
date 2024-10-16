use std::{collections::HashMap, mem};
use std::mem::replace;

use crate::ast::{ASTNode, ASTNodeType, PrimitiveTypes, ConstLiteral};
use crate::token::Operator;

pub struct TypeChecker {
  scopes: Vec<HashMap<String, (String, PrimitiveTypes)>>,
  var_types: HashMap<String, PrimitiveTypes>,
  var_ref_count: HashMap<String, usize>,
  functions: HashMap<String, (Vec<PrimitiveTypes>, Option<PrimitiveTypes>)>,
  current_function_return_type: Option<PrimitiveTypes>
}

impl Default for TypeChecker {
    fn default() -> Self {
      Self::new()
    }
}

impl TypeChecker {
  pub fn new() -> Self {
    TypeChecker {
      scopes: Vec::new(),
      var_types: HashMap::new(),
      functions: HashMap::new(),
      var_ref_count: HashMap::new(),
      current_function_return_type: None,
    }
  }

  pub fn prepare_ast(&mut self, ast: &mut Vec<ASTNode>) {
    self.register_functions(ast);
    println!("{:?}", self.functions);
    self.rename_global_variables(ast);
    self.resolve_types(ast);
  }

  fn advance_ref_counter(&mut self, name: String) -> Option<usize> {
    let Some(&n) = self.var_ref_count.get(&name) else {
      self.var_ref_count.insert(name, 0);
      return None;
    };
    self.var_ref_count.insert(name, n + 1);
    Some(n)
  }

  fn declare_var(&mut self, name: String, value_type: PrimitiveTypes) -> String {
    if self.scopes.is_empty() {
      panic!()
    }

    if let Some(n) = self.advance_ref_counter(name.clone()) {
      let new_name = format!("{}_{}", name, n);
      self.scopes.last_mut().unwrap().insert(name, (new_name.clone(), value_type.clone()));
      self.var_types.insert(new_name.clone(), value_type);
      new_name
    }
    else {
      self.scopes.last_mut().unwrap().insert(name.clone(), (name.clone(), value_type.clone()));
      self.var_types.insert(name.clone(), value_type);
      name
    }
  }

  fn declare_parameters(&mut self, parameters: &mut Vec<(String, PrimitiveTypes)>) {
    if self.scopes.is_empty() {
      panic!()
    }
    if !self.scopes.last().unwrap().is_empty() {
      panic!("Expected last scope to be emoty for parameters");
    }
    if parameters.is_empty() {
      return;
    }

    let mut dup: Vec<String> = Vec::new();
    // check for duplicate names
    for (name, value_type) in parameters {
      if dup.contains(name) {
        panic!("Duplicate parameter name '{}'", name);
      }
      let new_name = self.declare_var(name.clone(), value_type.clone());
      let old_name = replace(name, new_name);
      dup.push(old_name);
    }
  }

  fn get_var(&self, name: &String) -> Option<(String, PrimitiveTypes)> {
    if self.scopes.is_empty() {
      panic!()
    }
    for scope in self.scopes.iter().rev() {
      if let Some(var) = scope.get(name) {
        return Some(var.clone());
      }
    }
    None
  }

  fn get_var_type(&self, name: &String) -> Option<PrimitiveTypes> {
    if let Some(value_type) = self.var_types.get(name) {
      return Some(value_type.clone());
    }
    None
  }

  fn register_functions(&mut self, ast: &Vec<ASTNode>) {
    for node in ast {
      if let ASTNodeType::FunctionDef(ref name, ref args, ref return_type, _) = node.node_type {
        if self.functions.contains_key(name) {
          panic!("double function delcaration '{name}'")
        }
        let mut arg_types = Vec::new();
        if let Some(args) = args {
          for (_, ref arg_type) in args {
            arg_types.push(arg_type.clone());
          }
        };
        self.functions.insert(name.clone(), (arg_types, return_type.clone()));
      }
    }
  }

  fn rename_global_variables(&mut self, ast: &mut Vec<ASTNode>) {
    self.rename_global_variables_statements(ast);
  }

  fn rename_global_variables_statements(&mut self, ast: &mut Vec<ASTNode>) {
    self.scopes.push(HashMap::new());
    for node in ast {
      match node.node_type {
        ASTNodeType::FunctionDef(_, ref mut args, _, ref mut body) => {
          self.scopes.push(HashMap::new());
          if let Some(args) = args {
            self.declare_parameters(args);
          }
          self.rename_global_variables_statements(body);
          self.scopes.pop();
        }
        ASTNodeType::Assignment(ref mut name, ref mut value) => {
          self.rename_global_variables_expression(value);
          let Some((new_name, _)) = self.get_var(name) else {
            panic!("Var '{}' was not declared", name);
          };
          let _ = mem::replace(name, new_name);
        },
        ASTNodeType::SExpression(ref mut expr) => self.rename_global_variables_expression(expr),
        ASTNodeType::BuiltinFunction(_, ref mut expr) => {
          // println!("WARNING: BuiltIn function are pure statements atm. this will change!");
          // println!("WARNING: BuiltIn function arguments are perceived as on expression, not actual arguments!");
          self.rename_global_variables_expression(expr);
        },
        ASTNodeType::Declaration(ref mut name, ref value_type, ref mut value) => {
          if let Some(value) = value {
            self.rename_global_variables_expression(value);
          }
          let new_name = self.declare_var(name.clone(), value_type.clone());
          let _ = mem::replace(name, new_name);
        },
        ASTNodeType::Const(ref mut name, ref const_type, _) => {
          let new_name = self.declare_var(name.clone(), const_type.clone());
          let _ = mem::replace(name, new_name);
        }
        ASTNodeType::If(ref mut cond, ref mut then, ref mut els) => {
          self.rename_global_variables_expression(cond);
          self.rename_global_variables_statements(then);
          if let Some(els) = els {
            self.rename_global_variables_statements(els);
          }
        },
        ASTNodeType::While(ref mut cond, ref mut body) => {
          self.rename_global_variables_expression(cond);
          self.rename_global_variables_statements(body);
        },
        ASTNodeType::Return(Some(ref mut expr)) => {
          self.rename_global_variables_expression(expr);
        }
        ASTNodeType::Return(None) => {}

        ASTNodeType::FunctionCall(_, _, _) |
        ASTNodeType::Literal(_, _) |
        ASTNodeType::Identifier(_, _) |
        ASTNodeType::BinaryOp(_, _, _, _) => {
          panic!("Unexpected Expression as Statement!");
        },
      }
    }
    self.scopes.pop();
  }

  fn rename_global_variables_expression(&mut self, node: &mut ASTNode) {
    match node.node_type {
      ASTNodeType::BinaryOp(ref mut left, _, ref mut right, _) => {
        self.rename_global_variables_expression(left);
        self.rename_global_variables_expression(right);
      },
      ASTNodeType::Literal(_, _) => {},
      ASTNodeType::Identifier(ref mut name, ref mut value_type) => {
        let Some((new_name, new_value_type)) = self.get_var(name) else {
          panic!("Var '{}' was not declared", name);
        };
        let _ = mem::replace(name, new_name);
        let _ = mem::replace(value_type, new_value_type);
      },
      ASTNodeType::FunctionCall(_, ref mut args, _) => {
        for expr in args {
          self.rename_global_variables_expression(expr);
        }
      },

      ASTNodeType::Return(_) |
      ASTNodeType::FunctionDef(_, _, _, _) |
      ASTNodeType::Assignment(_, _) |
      ASTNodeType::BuiltinFunction(_, _) |
      ASTNodeType::Declaration(_, _, _) |
      ASTNodeType::Const(_, _, _) |
      ASTNodeType::If(_, _, _) |
      ASTNodeType::While(_, _) |
      ASTNodeType::SExpression(_) => {
        panic!("Unexpected statement while renaming vars in expressions {:?}", node);
      }
    }
  }

  fn resolve_types(&mut self, ast: &mut Vec<ASTNode>) {
    self.resolve_types_statements(ast);
  }

  fn resolve_types_statements(&mut self, ast: &mut Vec<ASTNode>) {
    for node in ast {
      match node.node_type {
        ASTNodeType::FunctionDef(_, _, ref return_type, ref mut body) => {
          self.current_function_return_type = return_type.clone();
          self.resolve_types_statements(body);
          self.current_function_return_type = None;
        }
        ASTNodeType::Assignment(ref name, ref mut value) => {
          let new_type = self.resolve_types_expression(value);
          let Some(var_type) = self.get_var_type(name) else {
            panic!("Var '{name}' was not declared but tried to assign to.")
          };
          let dominant_type = TypeChecker::get_dominant_type(&var_type, &new_type);
          self.set_type_for_expression(value, dominant_type);
        }
        ASTNodeType::BuiltinFunction(_, ref mut expr) => {
          let expected_type = PrimitiveTypes::U64;
          let found_type = self.resolve_types_expression(expr);
          let dominant_type = TypeChecker::get_dominant_type(&expected_type, &found_type);
          self.set_type_for_expression(expr, dominant_type);
        }
        ASTNodeType::Declaration(_, ref value_type, ref mut value) => {
          if let Some(value) = value {
            let expr_type = self.resolve_types_expression(value);
            let dominant_type = TypeChecker::get_dominant_type(value_type, &expr_type);
            if value_type != dominant_type {
              panic!("mismatch in types during Declaration {:#?} {:#?}", value_type, expr_type);
            }
            self.set_type_for_expression(value, dominant_type);
          }
        }
        ASTNodeType::Const(_, ref const_type, ref mut value) => {
          match (const_type, value) {
            // valid combinations
            (PrimitiveTypes::F64, ConstLiteral::Float(_)) |
            (PrimitiveTypes::U64, ConstLiteral::Integer(_)) |
            (PrimitiveTypes::Bool, ConstLiteral::Bool(_)) => {}

            // valid const type but mismatch of types
            (PrimitiveTypes::U64, _) |
            (PrimitiveTypes::F64, _) |
            (PrimitiveTypes::Bool, _) => panic!(),

            // ambiguous/invalid types for a const
            (PrimitiveTypes::Number, _) |
            (PrimitiveTypes::Float, _) |
            (PrimitiveTypes::Integer, _) |
            (PrimitiveTypes::Void, _) |
            (PrimitiveTypes::COUNT, _) => panic!(),
          }
        }
        ASTNodeType::If(ref mut cond, ref mut then, ref mut els) => {
          let expected_type = PrimitiveTypes::U64;
          let found_type = self.resolve_types_expression(cond);
          let dominant_type = TypeChecker::get_dominant_type(&expected_type, &found_type);
          self.set_type_for_expression(cond, dominant_type);
          self.resolve_types_statements(then);
          if let Some(els) = els {
            self.resolve_types_statements(els);
          }
        }
        ASTNodeType::While(ref mut cond, ref mut body) => {
          // let dominant_type = TypeChecker::get_dominant_type(&expected_type, &found_type);
          self.set_type_for_expression(cond, &PrimitiveTypes::Bool);
          self.resolve_types_statements(body);
        }
        ASTNodeType::SExpression(ref mut expr) => {
          let _ = self.resolve_types_expression(expr);
        }
        ASTNodeType::Return(ref mut expr) => {
          match (expr, self.current_function_return_type.clone()) {
            (None, None) => {}, // fine
            (Some(ref mut expr), Some(return_type)) => {
              let found_type = self.resolve_types_expression(expr.as_mut());
              if found_type != return_type {
                panic!()
              }
            },

            (Some(_), None) |
            (None, Some(_)) => panic!("{:?}", node),
          }
        }

        ASTNodeType::FunctionCall(_, _, _) |
        ASTNodeType::BinaryOp(_, _, _, _) |
        ASTNodeType::Literal(_, _) |
        ASTNodeType::Identifier(_, _) => {
          panic!()
        },
      }
    }
  }

  fn resolve_types_expression(&mut self, node: &mut ASTNode) -> PrimitiveTypes{
    let found_type = self.find_operant_type(node);
    let new_type: PrimitiveTypes = match node.node_type {
        ASTNodeType::BinaryOp(ref mut left, Operator::Equal, ref mut right, ref mut op_type) |
        ASTNodeType::BinaryOp(ref mut left, Operator::Less, ref mut right, ref mut op_type) |
        ASTNodeType::BinaryOp(ref mut left, Operator::Greater, ref mut right, ref mut op_type) => {
          self.set_type_for_expression(left, &found_type);
          self.set_type_for_expression(right, &found_type);
          let _ = replace(op_type, PrimitiveTypes::Bool);
          return found_type
        }
        ASTNodeType::BinaryOp(_, _, _, _) => found_type,
        ASTNodeType::Literal(ref typ, _) => typ.clone(),
        ASTNodeType::Identifier(_, ref value_type) => {
          let dominant_type = TypeChecker::get_dominant_type(value_type, &found_type);
          dominant_type.clone()
        }
        ASTNodeType::FunctionCall(ref name, ref mut args, _) => {
          if !args.is_empty() {
            // print!("WARNING: All return values of all functions are none atm.");
            // print!("WARNING: All functions calls are treated as statements atm.");
            if !self.functions.contains_key(name) {
              panic!("Function '{}' was not defined", name);
            };
            if args.len() != self.functions.get(name).unwrap().0.len() {
              panic!("number of arguments and parameter does not match up for '{}'", name);
            }

            for (i, arg) in args.iter_mut().enumerate() {
              let found_type = self.resolve_types_expression(arg);
              let exprected_type = &self.functions.get(name).unwrap().0[i];
              let dominant_type = TypeChecker::get_dominant_type(exprected_type, &found_type);
              self.set_type_for_expression(arg, dominant_type);
            }
          }
          let Some(func) = self.functions.get(name) else {
            panic!()
          };
          let Some(ref return_type) = func.1 else {
            panic!()
          };
          return_type.clone()
        }

        ASTNodeType::Return(_) |
        ASTNodeType::FunctionDef(_, _, _, _) |
        ASTNodeType::Assignment(_, _) |
        ASTNodeType::BuiltinFunction(_, _) |
        ASTNodeType::Declaration(_, _, _) |
        ASTNodeType::Const(_, _, _) |
        ASTNodeType::If(_, _, _) |
        ASTNodeType::While(_, _) |
        ASTNodeType::SExpression(_) => {
          panic!()
        },
    };
    self.set_type_for_expression(node, &new_type);
    new_type
  }

  fn set_type_for_expression(&self, expr: &mut ASTNode, new_type: &PrimitiveTypes) {
    match expr.node_type {
      ASTNodeType::Literal(ref mut typ, _) => {
        let _ = mem::replace(typ, new_type.clone());
      }
      ASTNodeType::Identifier(_, ref typ) => {
        if typ != new_type {
          panic!();
        }
      }
      ASTNodeType::BinaryOp(ref mut left, ref op, ref mut right, ref mut typ) => {
        match op {
          Operator::Plus |
          Operator::Minus |
          Operator::Mul |
          Operator::Div => {
            let _ = mem::replace(typ, new_type.clone());
            self.set_type_for_expression(left, new_type);
            self.set_type_for_expression(right, new_type);
          }
          Operator::Equal |
          Operator::Greater |
          Operator::Less => {
            if new_type != &PrimitiveTypes::Bool {
              panic!("Exprected type '{:#?}', but operands '==', '>', '<' are always returning bool", new_type)
            }
            let _ = mem::replace(typ, PrimitiveTypes::Bool);
            let left_t = self.find_operant_type(left);
            let right_t = self.find_operant_type(right);
            let dominant_type = TypeChecker::get_dominant_type(&left_t, &right_t);
            self.set_type_for_expression(left, dominant_type);
            self.set_type_for_expression(right, dominant_type);
          }

          Operator::And |
          Operator::Or => {
            if new_type != &PrimitiveTypes::Bool {
              panic!("The 'and' and 'or' operators can only operate on 'bool' oprants")
            }
            let _ = mem::replace(typ, PrimitiveTypes::Bool);
            self.set_type_for_expression(left, new_type);
            self.set_type_for_expression(right, new_type);
          }
          Operator::Assignment |
          Operator::ThinArrow => panic!(),
        }
      }
      ASTNodeType::FunctionCall(ref name, ref mut args, ref mut call_type) => {
        if args.is_empty() {
          return;
        }
        let Some(parameters) = self.functions.get(name) else {
          panic!("Could not find function")
        };
        let Some(return_type) = &parameters.1 else {
          panic!("Expected return type for function")
        };
        let parameters = &parameters.0;
        for (i, arg) in args.iter_mut().enumerate() {
          let parameter_type = parameters[i].clone();
          let arg_type = self.find_operant_type(arg);
          let dominant_type = TypeChecker::get_dominant_type(&parameter_type, &arg_type);
          self.set_type_for_expression(arg, dominant_type);
        }
        if new_type != return_type {
          panic!("function call in expression has type '{:?}', but expected type '{:?}'", return_type, new_type)
        }
        let _ = replace(call_type, new_type.clone());
      }

      ASTNodeType::FunctionDef(_, _, _, _) |
      ASTNodeType::Return(_) |
      ASTNodeType::Assignment(_, _) |
      ASTNodeType::BuiltinFunction(_, _) |
      ASTNodeType::Declaration(_, _, _) |
      ASTNodeType::Const(_, _, _) |
      ASTNodeType::If(_, _, _) |
      ASTNodeType::While(_, _) |
      ASTNodeType::SExpression(_) => {
        panic!()
      },
    }
  }

  fn get_dominant_type<'a>(left_t: &'a PrimitiveTypes, right_t: &'a PrimitiveTypes) -> &'a PrimitiveTypes {
    match left_t {
      PrimitiveTypes::Number => {
        match right_t {
          PrimitiveTypes::Void |
          PrimitiveTypes::Number => left_t,

          PrimitiveTypes::Float |
          PrimitiveTypes::F64 |
          PrimitiveTypes::Integer |
          PrimitiveTypes::U64 => right_t,

          PrimitiveTypes::Bool => panic!("Mismatch of types '{:#?}' and 'bool", left_t),
          PrimitiveTypes::COUNT => panic!("Count is not a valid type"),
        }
      },

      PrimitiveTypes::Float => {
        match right_t {
          PrimitiveTypes::Void |
          PrimitiveTypes::Number |
          PrimitiveTypes::Float => left_t,

          PrimitiveTypes::F64 => right_t,

          PrimitiveTypes::Integer |
          PrimitiveTypes::U64 => panic!("mismatch in types {:#?} {:#?}", left_t, right_t),

          PrimitiveTypes::Bool => panic!("Mismatch of types '{:#?}' and 'bool", left_t),
          PrimitiveTypes::COUNT => panic!("Count is not a valid type"),
        }
      },

      PrimitiveTypes::Integer => {
        match right_t {
          PrimitiveTypes::Void |
          PrimitiveTypes::Number |
          PrimitiveTypes::Integer => left_t,

          PrimitiveTypes::U64 => right_t,

          PrimitiveTypes::Float |
          PrimitiveTypes::F64 => panic!("mismatch in types {:#?} {:#?}", left_t, right_t),

          PrimitiveTypes::Bool => panic!("Mismatch of types '{:#?}' and 'bool", left_t),
          PrimitiveTypes::COUNT => panic!("Count is not a valid type"),
        }
      },

      PrimitiveTypes::Void => {
        match right_t {
          PrimitiveTypes::Void |
          PrimitiveTypes::Number |
          PrimitiveTypes::Integer |
          PrimitiveTypes::U64 |
          PrimitiveTypes::Float |
          PrimitiveTypes::F64 |
          PrimitiveTypes::Bool => right_t,

          PrimitiveTypes::COUNT => panic!("Count is not a valid type"),
        }
      },

      PrimitiveTypes::U64 => {
        match right_t {
          PrimitiveTypes::Void |
          PrimitiveTypes::Number |
          PrimitiveTypes::Integer |
          PrimitiveTypes::U64 => left_t,

          PrimitiveTypes::F64 |
          PrimitiveTypes::Float => panic!("mismatch in types {:#?} {:#?}", left_t, right_t),

          PrimitiveTypes::Bool => panic!("Mismatch of types '{:#?}' and 'bool", left_t),
          PrimitiveTypes::COUNT => panic!("Count is not a valid type"),
        }
      },
      PrimitiveTypes::F64 => {
        match right_t {
          PrimitiveTypes::Void |
          PrimitiveTypes::Number |
          PrimitiveTypes::Float |
          PrimitiveTypes::F64 => left_t,

          PrimitiveTypes::Integer |
          PrimitiveTypes::U64 => panic!("mismatch in types {:#?} {:#?}", left_t, right_t),

          PrimitiveTypes::Bool => panic!("Mismatch of types '{:#?}' and 'bool", left_t),
          PrimitiveTypes::COUNT => panic!("Count is not a valid type"),
        }
      },
      PrimitiveTypes::Bool => {
        match right_t {
          PrimitiveTypes::Bool => left_t,

          PrimitiveTypes::Number |
          PrimitiveTypes::Float |
          PrimitiveTypes::Integer |
          PrimitiveTypes::Void |
          PrimitiveTypes::U64 |
          PrimitiveTypes::F64 => panic!("Mismatch of types 'bool' and '{:#?}", right_t),

          PrimitiveTypes::COUNT => panic!("Count is not a valid type"),
        }
      }
      PrimitiveTypes::COUNT => panic!("Count is not a valid type"),
    }

  }

  fn find_operant_type(&self, expr: &ASTNode) -> PrimitiveTypes {
    match expr.node_type {
        ASTNodeType::Identifier(_, ref typ) => typ.clone(),
        ASTNodeType::Literal(ref typ, _) => typ.clone(),
        ASTNodeType::FunctionCall(ref name, _, _) => {
          let Some(funciton) = self.functions.get(name) else {
            panic!("Could not find function '{}' while type checking function call", name);
          };
          let Some(ref return_type) = funciton.1 else {
            panic!("Function '{}' does not return anything!", name)
          };
          return_type.clone()
        }
        ASTNodeType::BinaryOp(ref left, _, ref right, _) => {
          let left_t = self.find_operant_type(left);
          let right_t = self.find_operant_type(right);
          TypeChecker::get_dominant_type(&left_t, &right_t).clone()
        },

        _ => panic!("Error while evaluating expr type. Unexpected ASTNode")
    }
  }

}
