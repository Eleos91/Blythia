use crate::ast::PrimitiveTypes;
use crate::operations::Operation;

pub mod systemv;

pub trait Parameter {
  fn translate_store(&self, operations: &mut Vec<Operation>);
  fn translate_load(&self, operations: &mut Vec<Operation>);
}

impl Parameter for systemv::Parameter {
  fn translate_store(&self, operations: &mut Vec<Operation>) {
    self.translate_store(operations);
  }

  fn translate_load(&self, operations: &mut Vec<Operation>) {
    self.translate_load(operations);
  }
}

pub trait Parameters {
  fn add_parameters(&mut self, parameters: &[(String, PrimitiveTypes)]);
  fn translate_save_arguments(&self, index: usize, operations: &mut Vec<Operation>);
  fn trnslate_caller_argument(&self, index: usize, operations: &mut Vec<Operation>);
}
