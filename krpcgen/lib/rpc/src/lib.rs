
pub mod token;

pub enum Value {
    Number(i64),
    Identifier(String),
}

pub enum Integer {
    Integer,
    Hyper,
}

pub enum Float {
    Single,
    Double,
    Quadruple,
}

pub enum SwitchingType {
    Integer(Integer),
    Unsigned(Integer),
    Enum(String),
}

pub type Enum = std::collections::HashMap<String, Option<Value>>;
pub type Struct = std::collections::HashMap<String, Type>;
pub struct Union {
    pub switch_type: SwitchingType,
    pub arms: std::collections::HashMap<Value, (String, Type)>,
    pub default: Option<Box<(String, Type)>>,
}

pub enum NamedType {
    Typedef(Box<Type>),
    Enum(Enum),
    Struct(Struct),
    Union(Union),
}

pub enum Type {
    Void,
    Integer(Integer),
    Unsigned(Integer),
    Float(Float),
    Boolean,
    String,
    Opaque,
    Pointer(Box<Type>),
    Array(Box<Type>, usize),
    VArray(Box<Type>, Option<usize>),
    Named(String, NamedType),
}

pub struct Program {
    pub name: String,
    pub versions: std::collections::HashMap<Value, Version>,
}

pub struct Version {
    pub name: String,
    pub procedures: std::collections::HashMap<Value, Procedure>,
}

pub struct Procedure {
    pub name: String,
    pub return_type: Type,
    pub arguments: Vec<Type>,
}

pub struct Module {
    pub namespace: std::collections::HashSet<String>,
    pub constants: std::collections::HashMap<String, Value>,
    pub defined_types: std::collections::HashMap<String, NamedType>,
    pub programs: std::collections::HashMap<Value, Program>,
}

