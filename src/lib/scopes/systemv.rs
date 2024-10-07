use crate::{ast::PrimitiveTypes, operations::Operation};

#[derive(Debug, Clone)]
enum ParameterClass {
  Integer(usize),
  Sse(usize),
  // SSEUp,
  // X87,
  // X87Up,
  // ComplexX87,
  // NoClass,
  Memory(usize),
}

#[derive(Debug, Clone)]
pub struct Parameter {
  // name: String,
  // arg_index: usize,
  class: ParameterClass,
  class_index: usize,
  // value_type: PrimitiveTypes,
}

impl Parameter {
  pub fn translate_store(&self, operations: &mut Vec<Operation>) {
    match self.class {
      ParameterClass::Integer(offset) => {
        operations.push(Operation::SysVIntegerPrameterStore(offset));
      }
      ParameterClass::Sse(offset) => {
        operations.push(Operation::SysVSSEParameterStore(offset));
      }
      ParameterClass::Memory(offset) => {
        operations.push(Operation::SysVMemoryParameterStore(offset));
      }
    }
  }

  pub fn translate_load(&self, operations: &mut Vec<Operation>) {
    match self.class {
      ParameterClass::Integer(offset) => {
        operations.push(Operation::SysVIntegerPrameterLoad(offset));
      }
      ParameterClass::Sse(offset) => {
        operations.push(Operation::SysVSSEParameterLoad(offset));
      }
      ParameterClass::Memory(offset) => {
        operations.push(Operation::SysVMemoryParameterLoad(offset));
      }
    }
  }
}

#[derive(Debug, Clone)]
 pub struct SystemV {
  parameters: Vec<Parameter>,
  integer_parameters: Vec<Parameter>,
  sse_parameter: Vec<Parameter>,
  // sse_up_count: usize,
  // x87_count: usize,
  // x87_up_count: usize,
  // complex_x87_count: usize,
  // no_class_count: usize,
  memory_parameters: Vec<Parameter>,
  memory_size: usize,
  stack_reserve_size: usize,
 }

impl Default for SystemV {
    fn default() -> Self {
      SystemV::new()
    }
}

impl SystemV {

  pub fn new() -> Self {
    SystemV {
      parameters: Vec::new(),
      integer_parameters: Vec::new(),
      sse_parameter: Vec::new(),
      // sse_up_count: 0,
      // x87_count: 0,
      // x87_up_count: 0,
      // complex_x87_count: 0,
      // no_class_count: 0,
      memory_parameters: Vec::new(),
      memory_size: 0,
      stack_reserve_size: 0,
    }
  }

  pub fn is_empty(&self) -> bool {
    self.parameters.is_empty()
  }

  pub fn reserved_stack(&self) -> usize {
    self.stack_reserve_size
  }

  fn add(&mut self, value_type: &PrimitiveTypes) {
    let parameter: Parameter;
    match value_type {
      PrimitiveTypes::Bool |
      PrimitiveTypes::U64 => {
        if self.integer_parameters.len() < 6 {
          parameter = Parameter {
            class: ParameterClass::Integer(self.stack_reserve_size),
            class_index: self.integer_parameters.len(),
          };
          self.stack_reserve_size += 8;
          self.integer_parameters.push(parameter.clone());
        }
        else {
          parameter = Parameter {
            class: ParameterClass::Memory(self.memory_size),
            class_index: self.memory_parameters.len(),
          };
          self.memory_parameters.push(parameter.clone());
          self.memory_size += 8;
        };
      }
      PrimitiveTypes::F64 => {
        if self.sse_parameter.len() < 8 {
          parameter = Parameter {
            class: ParameterClass::Sse(self.stack_reserve_size),
            class_index: self.sse_parameter.len(),
          };
          self.stack_reserve_size += 8;
          self.sse_parameter.push(parameter.clone());
        }
        else {
          parameter = Parameter {
            class: ParameterClass::Memory(self.memory_size),
            class_index: self.memory_parameters.len(),
          };
          self.memory_parameters.push(parameter.clone());
          self.memory_size += 8;
        }
      },

      PrimitiveTypes::Number |
      PrimitiveTypes::Float |
      PrimitiveTypes::Integer |
      PrimitiveTypes::Void |
      PrimitiveTypes::COUNT => panic!(),
    }
    self.parameters.push(parameter);
  }

  pub fn len(&self) -> usize {
    self.parameters.len()
  }

  pub fn get(&self, index: usize) -> Option<&Parameter> {
    self.parameters.get(index)
  }

  pub fn add_parameters(&mut self, parameters: &Vec<(String, PrimitiveTypes)>) {
    let mut memory_class: Vec<&(String, PrimitiveTypes)> = Vec::new();
    for p @ (_name, value_type) in parameters {
      match value_type {
        PrimitiveTypes::Bool |
        PrimitiveTypes::U64 => {
          if self.integer_parameters.len() < 6 {
            self.add(value_type);
          }
          else {
            memory_class.push(p);
          }
        }
        PrimitiveTypes::F64 => {
          if self.sse_parameter.len() < 8 {
            self.add(value_type);
          }
          else {
            memory_class.push(p);
          }
        }

        PrimitiveTypes::Number |
        PrimitiveTypes::Float |
        PrimitiveTypes::Integer |
        PrimitiveTypes::Void |
        PrimitiveTypes::COUNT => panic!()
      }
      println!("{}", self.stack_reserve_size);
    }
    for (_name, value_type) in memory_class.iter().rev() {
      self.add(value_type);
    }
  }

  pub fn translate_save_arguments(&self, index: usize, operations: &mut Vec<Operation>) {
    let Some(parameter) = self.parameters.get(index) else {
      panic!("function only has '{}' parameters, but tried to access the '{}'th parameter", self.parameters.len(), index)
    };
    match parameter.class {
      ParameterClass::Integer(offset) => {
        operations.push(Operation::SysVIntegerSaveArgumentAfterCall(parameter.class_index, offset));
      }
      ParameterClass::Sse(offset) => {
        operations.push(Operation::SysVSSESaveArgumentAfterCall(parameter.class_index, offset));
      }
      ParameterClass::Memory(_) => {} // nothing to do. already on stack
    }
  }

  pub fn trnslate_caller_argument(&self, index: usize, operations: &mut Vec<Operation>) {
    let Some(parameter) = self.parameters.get(index) else {
      panic!("function only has '{}' parameters, but tried to access the '{}'th parameter", self.parameters.len(), index)
    };
    match parameter.class {
      ParameterClass::Integer(_) => {
        operations.push(Operation::SysVIntegerArguemtnPreparation(parameter.class_index));
      }
      ParameterClass::Sse(_) => {
        operations.push(Operation::SysVSSEArgumentPreparation(parameter.class_index));
      }
      ParameterClass::Memory(offset) => {
        operations.push(Operation::SysVMemoryArgumentPreparation(offset));
      }
    }
  }

}
