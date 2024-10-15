use crate::token::Operator;

#[derive(Debug, PartialEq, Clone, PartialOrd)]
pub enum PrimitiveTypes {
    // ambiguous types
    Number,
    Float,
    Integer,
    Void,

    // explicit types
    U64,
    F64,
    Bool,

    // Only temporarely
    COUNT,
}

pub fn match_type(typ: &str) -> Option<PrimitiveTypes> {
    match typ {
        "u64" => Some(PrimitiveTypes::U64),
        "f64" => Some(PrimitiveTypes::F64),
        "bool" => Some(PrimitiveTypes::Bool),
        "void" => Some(PrimitiveTypes::Void),
        _ => None,
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct ASTNode {
    pub node_type: ASTNodeType,
    pub loc: (usize, usize),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ASTNodeType {
    FunctionDef(String, Option<Vec<(String, PrimitiveTypes)>>, Option<PrimitiveTypes>, Vec<ASTNode>),
    FunctionCall(String, Vec<ASTNode>, PrimitiveTypes),
    Assignment(String, Box<ASTNode>),
    BinaryOp(Box<ASTNode>, Operator, Box<ASTNode>, PrimitiveTypes),
    Literal(PrimitiveTypes, String),
    Identifier(String, PrimitiveTypes),
    BuiltinFunction(String, Box<ASTNode>),
    Declaration(String, PrimitiveTypes, Option<Box<ASTNode>>),
    If(Box<ASTNode>, Vec<ASTNode>, Option<Vec<ASTNode>>),
    While(Box<ASTNode>, Vec<ASTNode>),
    SExpression(Box<ASTNode>), // used for standalone expr to clean up stack
    Return(Option<Box<ASTNode>>),
}

impl ASTNode {

    pub fn get_loc(&self) -> &(usize, usize) {
        &self.loc
    }

    pub fn get_type(&self) -> Result<PrimitiveTypes, String> {
        match &self.node_type {
            ASTNodeType::BinaryOp( _, _, _, typ) => Ok(typ.clone()),
            ASTNodeType::Literal(typ, _) => Ok(typ.clone()),
            ASTNodeType::Identifier(_, typ) => Ok(typ.clone()),
            ASTNodeType::FunctionCall(_, _, return_type) => Ok(return_type.clone()),
            _ => Err(format!("Tried to access type of typeless node: {:#?}", self))
        }
    }
}
