use std::{collections::HashMap, mem};
use std::mem::replace;

use crate::ast::{ASTNode, ASTNodeType, PrimitiveTypes};

pub struct TypeChecker {
  scopes: Vec<HashMap<String, (String, PrimitiveTypes)>>,
  var_types: HashMap<String, PrimitiveTypes>,
  var_ref_count: HashMap<String, usize>,
  functions: HashMap<String, (Vec<PrimitiveTypes>, Option<PrimitiveTypes>)>,
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
    }
  }

  pub fn prepare_ast(&mut self, ast: &mut Vec<ASTNode>) {
    self.register_functions(ast);
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
      if let ASTNodeType::FunctionDef(ref name, ref args, _) = node.node_type {
        if self.functions.contains_key(name) {
          panic!("double function delcaration '{name}'")
        }
        let mut arg_types = Vec::new();
        if let Some(args) = args {
          for (_, ref arg_type) in args {
            arg_types.push(arg_type.clone());
          }
        };
        self.functions.insert(name.clone(), (arg_types, None));
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
        ASTNodeType::FunctionDef(_, ref mut args, ref mut body) => {
          self.scopes.push(HashMap::new());
          if let Some(args) = args {
            self.declare_parameters(args);
          }
          self.rename_global_variables_statements(body);
          self.scopes.pop();
        }
        ASTNodeType::FunctionCall(_, ref mut args) => {
          for expr in args {
            self.rename_global_variables_expression(expr);
          }
        },
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

      ASTNodeType::FunctionDef(_, _, _) |
      ASTNodeType::FunctionCall(_, _) |
      ASTNodeType::Assignment(_, _) |
      ASTNodeType::BuiltinFunction(_, _) |
      ASTNodeType::Declaration(_, _, _) |
      ASTNodeType::If(_, _, _) |
      ASTNodeType::While(_, _) |
      ASTNodeType::SExpression(_) => {
        panic!("Unexpected statement while renaming vars in expressions");
      }
    }
  }

  fn resolve_types(&mut self, ast: &mut Vec<ASTNode>) {
    self.resolve_types_statements(ast);
  }

  fn resolve_types_statements(&mut self, ast: &mut Vec<ASTNode>) {
    for node in ast {
      match node.node_type {
        ASTNodeType::FunctionDef(_, _, ref mut body) => {
          self.resolve_types_statements(body);
        },
        ASTNodeType::FunctionCall(ref name, ref mut args) => {
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
              TypeChecker::set_type_for_expression(arg, dominant_type);
            }
          }
        },
        ASTNodeType::Assignment(ref name, ref mut value) => {
          let new_type = self.resolve_types_expression(value);
          let Some(var_type) = self.get_var_type(name) else {
            panic!("Var '{name}' was not declared but tried to assign to.")
          };
          let dominant_type = TypeChecker::get_dominant_type(&var_type, &new_type);
          TypeChecker::set_type_for_expression(value, dominant_type);
        },
        ASTNodeType::BuiltinFunction(_, ref mut expr) => {
          let expected_type = PrimitiveTypes::U64;
          let found_type = self.resolve_types_expression(expr);
          let dominant_type = TypeChecker::get_dominant_type(&expected_type, &found_type);
          TypeChecker::set_type_for_expression(expr, dominant_type);
        }
        ASTNodeType::Declaration(_, ref value_type, ref mut value) => {
          if let Some(value) = value {
            let expr_type = self.resolve_types_expression(value);
            let dominant_type = TypeChecker::get_dominant_type(value_type, &expr_type);
            if value_type != dominant_type {
              panic!("mismatch in types during Declaration {:#?} {:#?}", value_type, expr_type);
            }
            TypeChecker::set_type_for_expression(value, dominant_type);
          }
        }
        ASTNodeType::If(ref mut cond, ref mut then, ref mut els) => {
          let expected_type = PrimitiveTypes::U64;
          let found_type = self.resolve_types_expression(cond);
          let dominant_type = TypeChecker::get_dominant_type(&expected_type, &found_type);
          TypeChecker::set_type_for_expression(cond, dominant_type);
          self.resolve_types_statements(then);
          if let Some(els) = els {
            self.resolve_types_statements(els);
          }
        },
        ASTNodeType::While(ref mut cond, ref mut body) => {
          let expected_type = PrimitiveTypes::U64;
          let found_type = self.resolve_types_expression(cond);
          let dominant_type = TypeChecker::get_dominant_type(&expected_type, &found_type);
          TypeChecker::set_type_for_expression(cond, dominant_type);
          self.resolve_types_statements(body);
        },
        ASTNodeType::SExpression(ref mut expr) => {
          let _ = self.resolve_types_expression(expr);
        },

        ASTNodeType::BinaryOp(_, _, _, _) |
        ASTNodeType::Literal(_, _) |
        ASTNodeType::Identifier(_, _) => {
          panic!()
        },
      }
    }
  }

  fn resolve_types_expression(&mut self, node: &mut ASTNode) -> PrimitiveTypes{
    let found_type = TypeChecker::find_operant_type(node);
    let new_type: PrimitiveTypes = match node.node_type {
        ASTNodeType::BinaryOp(_, _, _, _) => found_type,
        ASTNodeType::Literal(ref typ, _) => typ.clone(),
        ASTNodeType::Identifier(_, ref value_type) => {
          let dominant_type = TypeChecker::get_dominant_type(value_type, &found_type);
          dominant_type.clone()
        }

        ASTNodeType::FunctionDef(_, _, _) |
        ASTNodeType::FunctionCall(_, _) |
        ASTNodeType::Assignment(_, _) |
        ASTNodeType::BuiltinFunction(_, _) |
        ASTNodeType::Declaration(_, _, _) |
        ASTNodeType::If(_, _, _) |
        ASTNodeType::While(_, _) |
        ASTNodeType::SExpression(_) => {
          panic!()
        },
    };
    TypeChecker::set_type_for_expression(node, &new_type);
    new_type.clone()
  }

  fn set_type_for_expression(expr: &mut ASTNode, new_type: &PrimitiveTypes) {
    match expr.node_type {
      ASTNodeType::Literal(ref mut typ, _) => {
        let _ = mem::replace(typ, new_type.clone());
      }
      ASTNodeType::Identifier(_, ref typ) => {
        if typ != new_type {
          panic!();
        }
      }
      ASTNodeType::BinaryOp(ref mut left, _, ref mut right, ref mut typ) => {
        let _ = mem::replace(typ, new_type.clone());
        TypeChecker::set_type_for_expression(left, new_type);
        TypeChecker::set_type_for_expression(right, new_type);
      }

      ASTNodeType::FunctionDef(_, _, _) |
      ASTNodeType::FunctionCall(_, _) |
      ASTNodeType::Assignment(_, _) |
      ASTNodeType::BuiltinFunction(_, _) |
      ASTNodeType::Declaration(_, _, _) |
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
          PrimitiveTypes::F64 => right_t,

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

          PrimitiveTypes::COUNT => panic!("Count is not a valid type"),
        }
      },

      PrimitiveTypes::COUNT => panic!("Count is not a valid type"),
    }

  }

  fn find_operant_type(expr: &ASTNode) -> PrimitiveTypes {
    match expr.node_type {
        ASTNodeType::Identifier(_, ref typ) => typ.clone(),
        ASTNodeType::Literal(ref typ, _) => typ.clone(),
        ASTNodeType::BinaryOp(ref left, _, ref right, _) => {
          let left_t = TypeChecker::find_operant_type(left);
          let right_t = TypeChecker::find_operant_type(right);
          TypeChecker::get_dominant_type(&left_t, &right_t).clone()
        },

        _ => panic!("Error while evaluating expr type. Unexpected ASTNode")
    }
  }

}
