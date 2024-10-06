use phf::phf_map;

#[derive(Debug, PartialEq, Clone, PartialOrd)]
pub enum Token {
    Keyword(Keyword),
    Builtin(String),
    Identifier(String),
    Integer(String),
    Float(String),
    Operator(String),
    Assignment,
    LParen,
    RParen,
    Comma,
    Newline,
    Indent(usize),
    Colon,
    EOF,
}

pub type LocToken = ((usize, usize), Token);

#[derive(Debug, PartialEq, Clone, PartialOrd)]
pub enum Precedences {
    P0,
    P1,
    P2,
    Count,
}

impl Precedences {
    pub fn increment(&self) -> Precedences {
        match self {
            Precedences::P0 => Precedences::P1,
            Precedences::P1 => Precedences::P2,
            Precedences::P2 => Precedences::Count,
            Precedences::Count => panic!("Tried to increment Precedences over max!"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Plus,
    Minus,
    Mul,
    Div,
    Equal,
    Greater,
    Less,
}
pub const OPERATOR_SYMBOLS: [char; 8] = ['!','*','+','-','/','=','<','>'];
pub const OPERATOR_MAP: phf::Map<&str, Operator> = phf_map! {
    "==" => Operator::Equal,
    ">" => Operator::Greater,
    "<" => Operator::Less,
    "+" => Operator::Plus,
    "-" => Operator::Minus,
    "*" => Operator::Mul,
    "/" => Operator::Div,
};
pub const OPERATOR_PRECEDENCES: phf::Map<&str, Precedences> = phf_map! {
    "==" => Precedences::P0,
    ">" => Precedences::P0,
    "<" => Precedences::P0,
    "+" => Precedences::P1,
    "-" => Precedences::P1,
    "*" => Precedences::P2,
    "/" => Precedences::P2,
};

#[derive(Debug, PartialEq, Clone, PartialOrd)]
pub enum Keyword {
    Def,
    Var,
    If,
    Else,
    While,
}

pub fn match_keywords(s: &str) -> Option<Keyword> {
    match s {
        "def" => Some(Keyword::Def),
        "var" => Some(Keyword::Var),
        "if" => Some(Keyword::If),
        "else" => Some(Keyword::Else),
        "while" => Some(Keyword::While),
        _ => None,
    }
}

pub fn match_builtin_functions(s: &str) -> Option<String> {
    match s {
        "print_int" => Some(String::from("print_int")),
        _ => None,
    }
}

// Soon variant_count will be available. This is incredibly useful
// when adding variants. you can assert a certain amount of variants
// and so ensure to remeber where to implement the new variant.
//
// assert_eq!(std::mem::variant_count::<Token>(), 11);