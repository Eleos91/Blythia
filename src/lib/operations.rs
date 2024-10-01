
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

  ParameterIntegerStore(usize),
  ParameterIntegerLoad(usize),
  ArgumentIntegerStore(usize),
  ArgumentIntegerLoad(usize),
  
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


