
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
  EndFunction(),
  FunctionCall(String, usize),
}

pub enum Operant {
  LiteralFloat(f32),
  LiteralInt(i32),
}

#[derive(Debug)]
pub struct Program {
  pub function_defs: Vec<Operation>,
  pub main: Vec<Operation>,
  pub vars: Vec<String>,
}


