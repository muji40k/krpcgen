
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Literal {
    Integer(i64),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Bracket {
    Left,
    Right,
    LeftCurly,
    RightCurly,
    LeftSquare,
    RightSquare,
    LeftTriangle,
    RightTriangle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Type {
    Void,

    Unsigned,
    Integer, // 32 bit
    Hyper,   // 64 bit
    Float,
    Double,
    Boolean,
    Quadruple,

    String,
    Opaque,

    Pointer, // ~Optional

    Enum,
    Struct,
    Union,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Operator {
    Assign,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Separator {
    Semicolon,
    Colon,
    Comma,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Keyword {
    Const,
    Case,
    Switch,
    Default,
    Typedef,
    Program,
    Version,
    Procedure,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    Keyword(Keyword),
    Bracket(Bracket),
    Type(Type),
    Separator(Separator),
    Literal(Literal),
    Operator(Operator),
    Comment(String),
}

impl std::fmt::Display for Bracket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bracket::Left => write!(f, "("),
            Bracket::Right => write!(f, ")"),
            Bracket::LeftCurly => write!(f, "{{"),
            Bracket::RightCurly => write!(f, "}}"),
            Bracket::LeftSquare => write!(f, "["),
            Bracket::RightSquare => write!(f, "]"),
            Bracket::LeftTriangle => write!(f, "<"),
            Bracket::RightTriangle => write!(f, ">"),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Void => write!(f, "void"),
            Type::Unsigned => write!(f, "unsigned"),
            Type::Integer => write!(f, "int"),
            Type::Hyper => write!(f, "hyper"),
            Type::Float => write!(f, "float"),
            Type::Double => write!(f, "double"),
            Type::Quadruple => write!(f, "quadruple"),
            Type::Boolean => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Opaque => write!(f, "opaque"),
            Type::Pointer => write!(f, "*"),
            Type::Enum => write!(f, "enum"),
            Type::Struct => write!(f, "struct"),
            Type::Union => write!(f, "union"),
        }
    }
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::Assign => write!(f, "="),
        }
    }
}

impl std::fmt::Display for Separator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Separator::Semicolon => write!(f, ";"),
            Separator::Colon => write!(f, ":"),
            Separator::Comma => write!(f, ","),
        }
    }
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Keyword::Const => write!(f, "const"),
            Keyword::Case => write!(f, "case"),
            Keyword::Switch => write!(f, "switch"),
            Keyword::Default => write!(f, "default"),
            Keyword::Typedef => write!(f, "typedef"),
            Keyword::Program => write!(f, "program"),
            Keyword::Version => write!(f, "version"),
            Keyword::Procedure => write!(f, "procedure"),
        }
    }
}

