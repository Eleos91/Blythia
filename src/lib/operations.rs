
use std::{ops::{Deref, DerefMut}, rc::Rc};

#[derive(Debug)]
pub enum Operation {
  PushInt(String),
  AddInt,
  MultInt,
  MinusInt,
  DivInt,
  GreaterInt,
  LessInt,

  PushFloat(String),
  LiteralFloat,
  SwtichRegisterFloat,
  AddFloat,
  MultFloat,
  MinusFloat,
  DivFloat,

  PushBool(String),
  AndBool,
  OrBool,

  PopStack,

  PrintInt,

  EqualInt,

  If(usize),
  Else(usize),
  EndIF(usize),

  While(usize),
  CondWhile(usize),
  EndWhile(usize),

  StoreInt(String),
  LoadInt(String),
  StoreFloat(String),
  LoadFloat(String),


  // System V operations
  SysVIntegerArguemtnPreparation(usize),
  SysVIntegerSaveArgumentAfterCall(usize, usize), // (reg index, stack offset)
  SysVIntegerPrameterLoad(usize),
  SysVIntegerPrameterStore(usize),
  SysVSSEArgumentPreparation(usize),
  SysVSSESaveArgumentAfterCall(usize, usize),
  SysVSSEParameterLoad(usize),
  SysVSSEParameterStore(usize),
  SysVMemoryArgumentPreparation(usize),
  SysVMemoryParameterLoad(usize),
  SysVMemoryParameterStore(usize),

  BeginFunction(String),
  ReserveParameters(usize),
  EndFunction(String),
  FunctionCall(String, usize),
  SysVIntegerReturn,
  SysVSSEReturn,
  SysVPushIntegerReturn,
  SysVPushSSEReturn,
  Return(String),
}

pub enum Operant {
  LiteralFloat(f32),
  LiteralInt(i32),
}

pub struct Operations {
  operations: Vec<Operation>,
}

impl Deref for Operations {
  type Target = Vec<Operation>;

  fn deref(&self) -> &Self::Target {
        &self.operations
    }
}

impl DerefMut for Operations {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.operations
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationsType {
  Function(Rc<String>),
  Main,
}

#[derive(Debug)]
pub struct Program {
  pub function_defs: Vec<Operation>,
  pub main: Vec<Operation>,
  pub vars: Vec<String>,
  pub target: OperationsType,
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

impl Program {
  pub fn new() -> Self {
    Program {
      function_defs: Vec::new(),
      main: Vec::new(),
      vars: Vec::new(),
      target: OperationsType::Main,
    }
  }

  pub fn push(&mut self, op: Operation) {
    match self.target {
      OperationsType::Function(_) => self.function_defs.push(op),
      OperationsType::Main => self.main.push(op),
    }
  }

}

