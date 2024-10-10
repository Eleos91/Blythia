use crate::ast::PrimitiveTypes;
use crate::operations::Program;
use std::fmt::Debug;

pub mod systemv;

pub trait Parameter : Debug + Clone {
  fn translate_store(&self, operations: &mut Program);
  fn translate_load(&self, operations: &mut Program);
}

impl Parameter for systemv::Parameter {
  fn translate_store(&self, operations: &mut Program) {
    self.translate_store(operations);
  }

  fn translate_load(&self, operations: &mut Program) {
    self.translate_load(operations);
  }
}

pub trait Parameters<T> : Debug + Clone
where T: Parameter {
  fn add_parameters(&mut self, parameters: &[(String, PrimitiveTypes)]);
  fn translate_save_arguments(&self, index: usize, operations: &mut Program);
  fn trnslate_caller_argument(&self, index: usize, operations: &mut Program);
}

impl Parameters<systemv::Parameter> for systemv::SystemV {

  fn translate_save_arguments(&self, index: usize, operations: &mut Program) {
    self.translate_save_arguments(index, operations);
  }

  fn trnslate_caller_argument(&self, index: usize, operations: &mut Program) {
    self.trnslate_caller_argument(index, operations);
  }

  fn add_parameters(&mut self, parameters: &[(String, PrimitiveTypes)]) {
    self.add_parameters(parameters);
  }
}
