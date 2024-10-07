use ast::PrimitiveTypes;

pub mod lexer;
pub mod parser;
pub mod ast;
pub mod operations;
pub mod token;
pub mod builder;
pub mod compiler;
pub mod scopes;
pub mod type_checker;

pub trait Parameters<T> {
  fn add(&mut self, name: &str, value_type: &PrimitiveTypes);
  fn get(&self, name: String) -> T;
}

// impl Parameters<systemv::Parameter> for systemv::SystemV {
//     fn add(&mut self, name: &str, value_type: &PrimitiveTypes) {
//         self.add(name, value_type)
//     }

//     fn get(&self, _name: String) -> systemv::Parameter {
//         todo!()
//     }
// }
