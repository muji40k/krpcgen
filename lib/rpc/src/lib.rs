
pub mod token;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    Number(i64),
    Identifier(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Integer {
    Integer,
    Hyper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Float {
    Single,
    Double,
    Quadruple,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SwitchingType {
    Integer(Integer),
    Unsigned(Integer),
    Enum(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NamedType {
    Typedef(String),
    Enum(String),
    Struct(String),
    Union(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Void,
    Integer(Integer),
    Unsigned(Integer),
    Float(Float),
    Boolean,
    String,
    Opaque,
    Pointer(Box<Type>),
    Array(Box<Type>, Value),
    VArray(Box<Type>, Option<Value>),
    Named(NamedType),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub name: String,
    pub versions: indexmap::IndexMap<Value, Version>,
}

#[derive(Debug, Clone)]
pub struct Version {
    pub name: String,
    pub procedures: indexmap::IndexMap<Value, Procedure>,
}

#[derive(Debug, Clone)]
pub struct Procedure {
    pub name: String,
    pub return_type: Type,
    pub arguments: Vec<Type>,
}

pub type Enum = Vec<(String, Option<Value>)>;
pub type Struct = indexmap::IndexMap<String, Type>;

#[derive(Debug, Clone)]
pub struct Union {
    pub value: String,
    pub switch_type: SwitchingType,
    pub arms: indexmap::IndexMap<Value, (String, Type)>,
    pub default: Option<(String, Type)>,
}

#[derive(Debug, Clone)]
pub enum Definition {
    Const(String, Value),
    Typedef(String, Type),
    Enum(String, Enum),
    Struct(String, Struct),
    Union(String, Union),
    Program(Value, Program),
}

#[derive(Debug, Clone)]
pub struct Module {
    pub definitions: Vec<Definition>,
}

pub fn new_enum() -> Enum {
    Vec::new()
}

pub fn new_struct() -> Struct {
    indexmap::IndexMap::new()
}

pub fn new_union() -> Union {
    Union {
        value: String::new(),
        switch_type: SwitchingType::Integer(Integer::Integer),
        arms: indexmap::IndexMap::new(),
        default: None,
    }
}

pub fn new_program() -> Program {
    Program {
        name: String::new(),
        versions: indexmap::IndexMap::new(),
    }
}

pub fn new_version() -> Version {
    Version {
        name: String::new(),
        procedures: indexmap::IndexMap::new(),
    }
}

pub fn new_procedure() -> Procedure {
    Procedure {
        name: String::new(),
        return_type: Type::Void,
        arguments: Vec::new(),
    }
}

pub fn new_module() -> Module {
    Module {
        definitions: Vec::new(),
    }
}

