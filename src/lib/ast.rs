use crate::token::{Operator, PrimitiveTypes};


#[derive(Debug, PartialEq, Clone)]
pub enum ASTNode {
    FunctionDef { name: String, args: Option<Vec<String>>, body: Vec<ASTNode> },
    FunctionCall(String, Vec<ASTNode>),
    Assignment { name: String, value: Box<ASTNode> },
    BinaryOp { left: Box<ASTNode>, op: Operator, right: Box<ASTNode>, typ: PrimitiveTypes },
    // Number(f64),
    // Integer(i64),
    // Float(f64),
    Literal {typ: PrimitiveTypes, symbols: String},
    Identifier(String),
    BuiltinFunction(String, Box<ASTNode>),
    Declaration(String, Option<Box<ASTNode>>),
    If(Box<ASTNode>, Vec<ASTNode>, Option<Vec<ASTNode>>),
    While(Box<ASTNode>, Vec<ASTNode>),
    SExpression(Box<ASTNode>), // used for standalone expr to clean up stack
}

impl ASTNode {
    pub fn get_type(&self) -> Result<PrimitiveTypes, String> {
        match self {
            ASTNode::BinaryOp { left: _, op: _, right: _, typ } => Ok(typ.clone()),
            ASTNode::Literal { typ, symbols: _ } => Ok(typ.clone()),
            _ => Err(format!("Tried to access type of typeless node: {:#?}", self))
        }
    }
}