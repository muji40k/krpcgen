
#[cfg(test)]
mod test;

use rpc::{self, token};

#[derive(Debug)]
pub enum Error {
    UnknownDefinition(token::Token),
    UnexpectedToken(String, token::Token),
    ExpressionNotClosed(String, token::Token),
    UnexpectedEOF(String),
    UndefinedType(String),
    UndefinedValue(String),
    NonPositiveArraySize(i64),
    TypeRedefined(String),
    IdentifierRedefined(String),
    StructureFieldRedefined(String),
    NotSwitchingType(rpc::Type),
    UnionArmRedefined(UnionArm),
    UseOfPendingType(String),
    ProgramNumberReassigned(rpc::Value),
    VersionNumberReassigned(rpc::Value),
    ProcedureNumberReassigned(rpc::Value),
}

#[derive(Debug)]
pub enum UnionArm {
    Regular(rpc::Value),
    Default,
}

impl Error {
    fn unknown_definition<T>(t: token::Token) -> Result<T> {
        Err(Self::UnknownDefinition(t))
    }

    fn unexpected_token<T>(msg: String, t: token::Token) -> Result<T> {
        Err(Self::UnexpectedToken(msg, t))
    }

    fn expression_not_closed<T>(msg: String, t: token::Token) -> Result<T> {
        Err(Self::ExpressionNotClosed(msg, t))
    }

    fn unexpected_eof<T>(msg: String) -> Result<T> {
        Err(Self::UnexpectedEOF(msg))
    }

    fn undefined_type<T>(msg: String) -> Result<T> {
        Err(Self::UndefinedType(msg))
    }

    fn undefined_value<T>(msg: String) -> Result<T> {
        Err(Self::UndefinedValue(msg))
    }

    fn non_positive_array_size<T>(size: i64) -> Result<T> {
        Err(Self::NonPositiveArraySize(size))
    }

    fn type_redefined<T>(msg: String) -> Result<T> {
        Err(Self::TypeRedefined(msg))
    }

    fn identifier_redefined<T>(msg: String) -> Result<T> {
        Err(Self::IdentifierRedefined(msg))
    }

    fn structure_field_redefined<T>(msg: String) -> Result<T> {
        Err(Self::StructureFieldRedefined(msg))
    }

    fn not_switching_type<T>(tp: rpc::Type) -> Result<T> {
        Err(Self::NotSwitchingType(tp))
    }

    fn union_arm_redefined<T>(v: rpc::Value) -> Result<T> {
        Err(Self::UnionArmRedefined(UnionArm::Regular(v)))
    }

    fn union_default_redefined<T>() -> Result<T> {
        Err(Self::UnionArmRedefined(UnionArm::Default))
    }

    fn use_of_pending_type<T>(msg: String) -> Result<T> {
        Err(Self::UseOfPendingType(msg))
    }

    fn program_number_reassigned<T>(v: rpc::Value) -> Result<T> {
        Err(Self::ProgramNumberReassigned(v))
    }

    fn version_number_reassigned<T>(v: rpc::Value) -> Result<T> {
        Err(Self::VersionNumberReassigned(v))
    }

    fn procedure_number_reassigned<T>(v: rpc::Value) -> Result<T> {
        Err(Self::ProcedureNumberReassigned(v))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

struct PickIterator<I: Iterator<Item=token::Token>> {
    picked: Option<token::Token>,
    iter: I,
}

impl<I: Iterator<Item=token::Token>> PickIterator<I> {
    fn new(iter: I) -> Self {
        Self {
            picked: None,
            iter,
        }
    }

    fn push_back(self: &mut Self, t: token::Token) {
        match self.picked {
            Some(_) => panic!("Supposed to pick only one token"),
            None => self.picked = Some(t),
        }
    }
}

impl<I: Iterator<Item=token::Token>> Iterator for PickIterator<I> {
    type Item = token::Token;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(t) = self.picked.take() {
            Some(t)
        } else {
            self.iter.next()
        }
    }
}

struct PendingTypes {
    structs: Option<String>,
    unions: Option<String>,
}

struct DefinedTypes {
    typedefs: std::collections::HashSet<String>,
    enums: std::collections::HashSet<String>,
    structs: std::collections::HashSet<String>,
    unions: std::collections::HashSet<String>,
}

struct Handle<I: Iterator<Item=token::Token>> {
    tokens: PickIterator<I>,
    namespace: std::collections::HashSet<String>,
    values: std::collections::HashSet<String>,
    pending_types: PendingTypes,
    defined_types: DefinedTypes,
    assigned_numbers: std::collections::HashSet<rpc::Value>,
}

pub fn parse(tokens: impl Iterator<Item=token::Token>) -> Result<rpc::Module> {
    let mut module = rpc::new_module();
    let mut handle = Handle {
        tokens: PickIterator::new(tokens.filter(|t| match t {
            token::Token::Comment(_) => false,
            _ => true,
        })),
        namespace: std::collections::HashSet::new(),
        values: std::collections::HashSet::new(),
        pending_types: PendingTypes {
            structs: None,
            unions: None,
        },
        defined_types: DefinedTypes {
            typedefs: std::collections::HashSet::new(),
            enums: std::collections::HashSet::new(),
            structs: std::collections::HashSet::new(),
            unions: std::collections::HashSet::new(),
        },
        assigned_numbers: std::collections::HashSet::new(),
    };
    let mut err = None;

    while let (None, Some(t)) = (&err, handle.tokens.next()) {
        handle.tokens.push_back(t);
        match parse_definition(&mut handle) {
            Ok(def) => module.definitions.push(def),
            Err(error) => err = Some(error),
        }
    }

    match err {
        Some(err) => Err(err),
        None => Ok(module),
    }
}

fn parse_definition(
    handle: &mut Handle<impl Iterator<Item=token::Token>>,
) -> Result<rpc::Definition> {
    match handle.tokens.next() {
        None => Error::unexpected_eof("Definition expected".to_string()),
        Some(token::Token::Type(token::Type::Enum)) =>
            parse_enum_definition(handle).map(|(id, en)| rpc::Definition::Enum(id, en)),
        Some(token::Token::Keyword(token::Keyword::Const)) =>
            parse_const_definition(handle).map(|(id, v)| rpc::Definition::Const(id, v)),
        Some(token::Token::Keyword(token::Keyword::Typedef)) =>
            parse_typedef_definition(handle).map(|(id, tp)| rpc::Definition::Typedef(id, tp)),
        Some(token::Token::Type(token::Type::Struct)) =>
            parse_struct_definition(handle).map(|(id, st)| rpc::Definition::Struct(id, st)),
        Some(token::Token::Type(token::Type::Union)) =>
            parse_union_definition(handle).map(|(id, un)| rpc::Definition::Union(id, un)),
        Some(token::Token::Keyword(token::Keyword::Program)) =>
            parse_program_definition(handle).map(|(v, pr)| rpc::Definition::Program(v, pr)),
        Some(t) => Error::unknown_definition(t),
    }
}

fn parse_value(
    handle: &mut Handle<impl Iterator<Item=token::Token>>,
) -> Result<rpc::Value> {
    parse_value_condition(handle, |num| Ok(num))
}

fn parse_value_condition<F: FnOnce(i64) -> Result<i64>>(
    handle: &mut Handle<impl Iterator<Item=token::Token>>,
    cond: F,
) -> Result<rpc::Value> {
    match handle.tokens.next() {
        None => Error::unexpected_eof("Expected value".to_string()),
        Some(token::Token::Literal(token::Literal::Integer(num))) => cond(num).map(|num| rpc::Value::Number(num)),
        Some(token::Token::Identifier(id)) => match handle.values.get(&id) {
            Some(_) => Ok(rpc::Value::Identifier(id)),
            None => Error::undefined_value(id),
        },
        Some(t) => Error::unexpected_token("Expected value".to_string(), t),
    }
}

fn parse_type_identifier(
    handle: &mut Handle<impl Iterator<Item=token::Token>>,
) -> Result<rpc::Type> {
    match handle.tokens.next() {
        None => Error::unexpected_eof("Expected declaration type".to_string()),
        Some(token::Token::Type(t)) => match t {
            token::Type::Void => Ok(rpc::Type::Void),
            token::Type::Unsigned => match handle.tokens.next() {
                None => Error::unexpected_eof("Expected integer type".to_string()),
                Some(token::Token::Type(token::Type::Integer)) => Ok(rpc::Type::Unsigned(rpc::Integer::Integer)),
                Some(token::Token::Type(token::Type::Hyper)) => Ok(rpc::Type::Unsigned(rpc::Integer::Hyper)),
                Some(t) => Error::unexpected_token("Expected integer type".to_string(), t),
            },
            token::Type::Integer => Ok(rpc::Type::Integer(rpc::Integer::Integer)),
            token::Type::Hyper => Ok(rpc::Type::Integer(rpc::Integer::Hyper)),
            token::Type::Float => Ok(rpc::Type::Float(rpc::Float::Single)),
            token::Type::Double => Ok(rpc::Type::Float(rpc::Float::Double)),
            token::Type::Quadruple => Ok(rpc::Type::Float(rpc::Float::Quadruple)),
            token::Type::Boolean => Ok(rpc::Type::Boolean),
            token::Type::String => Ok(rpc::Type::String),
            token::Type::Opaque => Ok(rpc::Type::Opaque),
            token::Type::Enum => match handle.tokens.next() {
                None => Error::unexpected_eof("No identifier for enum".to_string()),
                Some(token::Token::Identifier(id)) => match handle.defined_types.enums.get(&id) {
                    None => Error::undefined_type(format!{"Unknown enum with identifier {id}"}),
                    _ => Ok(rpc::Type::Named(rpc::NamedType::Enum(id)))
                },
                Some(t) => Error::unexpected_token("No identifier for enum".to_string(), t),
            },
            token::Type::Struct => match handle.tokens.next() {
                None => Error::unexpected_eof("No identifier for struct".to_string()),
                Some(token::Token::Identifier(id)) => match (handle.defined_types.structs.get(&id), &handle.pending_types.structs) {
                    (None, None) =>
                        Error::undefined_type(format!{"Unknown struct with identifier {id}"}),
                    (None, Some(pid)) if *pid != id =>
                        Error::undefined_type(format!{"Unknown struct with identifier {id}"}),
                    _ => Ok(rpc::Type::Named(rpc::NamedType::Struct(id)))
                },
                Some(t) => Error::unexpected_token("No identifier for struct".to_string(), t),
            },
            token::Type::Union => match handle.tokens.next() {
                None => Error::unexpected_eof("No identifier for union".to_string()),
                Some(token::Token::Identifier(id)) => match (handle.defined_types.unions.get(&id), &handle.pending_types.unions) {
                    (None, None) =>
                        Error::undefined_type(format!{"Unknown union with identifier {id}"}),
                    (None, Some(pid)) if *pid != id =>
                        Error::undefined_type(format!{"Unknown union with identifier {id}"}),
                    _ => Ok(rpc::Type::Named(rpc::NamedType::Union(id)))
                },
                Some(t) => Error::unexpected_token("No identifier for union".to_string(), t),
            },
            token::Type::Pointer => Error::undefined_type("No type for pointer".to_string()),
        },
        Some(token::Token::Identifier(id)) => match handle.defined_types.typedefs.get(&id) {
            None => Error::undefined_type(format!{"Unknown type identifier {id}"}),
            _ => Ok(rpc::Type::Named(rpc::NamedType::Typedef(id)))
        },
        Some(t) => Error::unexpected_token("Expected declaration type".to_string(), t),
    }.and_then(|tp| Ok(match handle.tokens.next() {
        Some(token::Token::Type(token::Type::Pointer)) =>
            rpc::Type::Pointer(Box::new(tp)),
        None => tp,
        Some(t) => {
            handle.tokens.push_back(t);
            tp
        }
    })).and_then(|tp| match tp { // Check not to use pending types directly
        rpc::Type::Named(nm) => match nm {
            rpc::NamedType::Struct(st) => match &handle.pending_types.structs {
                Some(rst) if st == *rst => Error::use_of_pending_type(format!(
                    "Can't use struct \"{st}\" that is being defined directly"
                )),
                _ => Ok(rpc::Type::Named(rpc::NamedType::Struct(st)))
            },
            rpc::NamedType::Union(un) => match &handle.pending_types.unions {
                Some(run) if un == *run => Error::use_of_pending_type(format!(
                    "Can't use union \"{un}\" that is being defined directly"
                )),
                _ => Ok(rpc::Type::Named(rpc::NamedType::Union(un)))
            },
            _ => Ok(rpc::Type::Named(nm)),
        },
        _ => Ok(tp),
    })
}

fn parse_array_type(handle: &mut Handle<impl Iterator<Item=token::Token>>, tp: rpc::Type) -> Result<rpc::Type> {
    match handle.tokens.next() {
        Some(token::Token::Bracket(br)) => match br {
            token::Bracket::LeftTriangle => match handle.tokens.next() {
                None => Error::unexpected_eof("Expected variadic array closing bracket or size hint".to_string()),
                Some(token::Token::Bracket(token::Bracket::RightTriangle)) =>
                    Ok(rpc::Type::VArray(Box::new(tp), None)),
                Some(t) => {
                    handle.tokens.push_back(t);
                    parse_value_condition(handle, |num| if 0 >= num {
                        Error::non_positive_array_size(num)
                    } else {
                        Ok(num)
                    }).and_then(|v| match handle.tokens.next() {
                        None => Error::unexpected_eof("Expected variadic array closing bracket".to_string()),
                        Some(token::Token::Bracket(token::Bracket::RightTriangle)) =>
                            Ok(rpc::Type::VArray(Box::new(tp), Some(v))),
                        Some(t) => Error::unexpected_token("Expected variadic array closing bracket".to_string(), t),
                    })
                },
            },
            token::Bracket::LeftSquare => parse_value_condition(handle, |num| if 0 >= num {
                Error::non_positive_array_size(num)
            } else {
                Ok(num)
            }).and_then(|v| match handle.tokens.next() {
                None => Error::unexpected_eof("Expected array closing bracket".to_string()),
                Some(token::Token::Bracket(token::Bracket::RightSquare)) =>
                    Ok(rpc::Type::Array(Box::new(tp), v)),
                Some(t) => Error::unexpected_token("Expected array closing bracket".to_string(), t),
            }),
            _ => {
                handle.tokens.push_back(token::Token::Bracket(br));
                Ok(tp)
            }
        },
        None => Ok(tp),
        Some(t) => {
            handle.tokens.push_back(t);
            Ok(tp)
        }
    }
}

fn parse_declaration(handle: &mut Handle<impl Iterator<Item=token::Token>>) -> Result<(String, rpc::Type)> {
    parse_type_identifier(handle)
        .and_then(|tp| match handle.tokens.next() {
            None => Error::unexpected_eof("Expected declaration identifier".to_string()),
            Some(token::Token::Identifier(name)) => Ok((name, tp)),
            Some(t) => Error::unexpected_token("Expected declaration identifier".to_string(), t),
        }).and_then(|(name, tp)| {
            parse_array_type(handle, tp).map(|tp| (name, tp))
        })
}

fn parse_type(handle: &mut Handle<impl Iterator<Item=token::Token>>) -> Result<rpc::Type> {
    parse_type_identifier(handle).and_then(|tp| parse_array_type(handle, tp))
}

fn parse_enum_definition(
    handle: &mut Handle<impl Iterator<Item=token::Token>>,
) -> Result<(String, rpc::Enum)> {
    match handle.tokens.next() {                                 // Identifier
        None => Error::unexpected_eof("No enum identifier was provided".to_owned()),
        Some(token::Token::Identifier(id)) => match handle.defined_types.enums.get(&id) {
            Some(_) => Error::type_redefined(format!{"Enum with id \"{id}\" already exists"}),
            None => Ok(id),
        },
        Some(t) => Error::unexpected_token("Expected enum identifier".to_owned(), t),
    }.and_then(|id| match handle.tokens.next() {                 // {
        None => Error::unexpected_eof("No enum body".to_owned()),
        Some(token::Token::Bracket(token::Bracket::LeftCurly)) => Ok(id),
        Some(t) => Error::unexpected_token("Expected enum body \"{\"".to_owned(), t),
    }).and_then(|id| parse_enum_body(handle).map(|en| (id, en))) // Body
    .and_then(|pass| match handle.tokens.next() {                // }
        None => Error::unexpected_eof("Enum definition wasn't finished".to_owned()),
        Some(token::Token::Bracket(token::Bracket::RightCurly)) => Ok(pass),
        Some(t) => Error::unexpected_token("Enum definition wasn't finished".to_owned(), t),
    }).and_then(|pass| match handle.tokens.next() {              // ;
        None => Error::unexpected_eof("Enum definition wasn't finished".to_owned()),
        Some(token::Token::Separator(token::Separator::Semicolon)) => Ok(pass),
        Some(t) => Error::expression_not_closed("Enum definition wasn't finished".to_owned(), t),
    }).and_then(|(id, en)| {
        handle.defined_types.enums.insert(id.clone());
        Ok((id, en))
    })
}

fn parse_enum_item(
    handle: &mut Handle<impl Iterator<Item=token::Token>>,
) -> Result<(String, Option<rpc::Value>)> {
    match handle.tokens.next() {                        // Identifier
        None => Error::unexpected_eof("Expected enum item identifier".to_owned()),
        Some(token::Token::Identifier(id)) => match handle.namespace.get(&id) {
            Some(_) => Error::identifier_redefined(format!("Enum identifier \"{id}\" already exists")),
            None => Ok(id),
        },
        Some(t) => Error::expression_not_closed("Expected enum item identifier".to_owned(), t),
    }.and_then(|id| match handle.tokens.next() {        // [=]
        None => Ok((id, None)),
        Some(token::Token::Operator(token::Operator::Assign)) =>
            parse_value(handle).map(|v| (id, Some(v))), // [Value]
        Some(t) => {
            handle.tokens.push_back(t);
            Ok((id, None))
        }
    }).and_then(|(id, v)| {
        handle.namespace.insert(id.clone());
        handle.values.insert(id.clone());
        Ok((id, v))
    })
}

fn parse_enum_body(
    handle: &mut Handle<impl Iterator<Item=token::Token>>,
) -> Result<rpc::Enum> {
    let mut en = rpc::new_enum();
    let mut error: Option<Error> = None;

    while match parse_enum_item(handle) { // Item
        Ok(item) => {
            en.push(item);

            match handle.tokens.next() {  // [,]
                None => false,
                Some(token::Token::Separator(token::Separator::Comma)) => true,
                Some(t) => {
                    handle.tokens.push_back(t);
                    false
                }
            }
        },
        Err(err) => {
            error = Some(err);
            false
        },
    } {}

    match error {
        None => Ok(en),
        Some(err) => Err(err),
    }
}


fn parse_const_definition(
    handle: &mut Handle<impl Iterator<Item=token::Token>>,
) -> Result<(String, rpc::Value)> {
    match handle.tokens.next() {                           // Identifier
        None => Error::unexpected_eof("Expected const identifier".to_owned()),
        Some(token::Token::Identifier(id)) => match handle.namespace.get(&id) {
            Some(_) => Error::identifier_redefined(format!("Constant identifier \"{id}\" already exists")),
            None => Ok(id),
        },
        Some(t) => Error::expression_not_closed("Expected const identifier".to_owned(), t),
    }.and_then(|pass| match handle.tokens.next() {         // =
        None => Error::unexpected_eof("Expected assign sign".to_string()),
        Some(token::Token::Operator(token::Operator::Assign)) => Ok(pass),
        Some(t) => Error::unexpected_token("Expected assign sign".to_string(), t),
    }).and_then(|id| parse_value(handle).map(|v| (id, v))) // Value
    .and_then(|pass| match handle.tokens.next() {          // ;
        None => Error::unexpected_eof("Const definition wasn't finished".to_owned()),
        Some(token::Token::Separator(token::Separator::Semicolon)) => Ok(pass),
        Some(t) => Error::expression_not_closed("Const definition wasn't finished".to_owned(), t),
    }).and_then(|(id, v)| {
        handle.namespace.insert(id.clone());
        handle.values.insert(id.clone());
        Ok((id, v))
    })
}

fn parse_typedef_definition(
    handle: &mut Handle<impl Iterator<Item=token::Token>>,
) -> Result<(String, rpc::Type)> {
    parse_declaration(handle)                           // Declaraion
        .and_then(|(id, tp)| match handle.namespace.get(&id) {
            Some(_) => Error::type_redefined(format!("Type with identifier {id} already exists")),
            None => Ok((id, tp)),
        }).and_then(|pass| match handle.tokens.next() { // ;
            None => Error::unexpected_eof("Const definition wasn't finished".to_owned()),
            Some(token::Token::Separator(token::Separator::Semicolon)) => Ok(pass),
            Some(t) => Error::expression_not_closed("Const definition wasn't finished".to_owned(), t),
        }).and_then(|(id, tp)| {
            handle.namespace.insert(id.clone());
            handle.defined_types.typedefs.insert(id.clone());
            Ok((id, tp))
        })
}

fn parse_struct_definition(
    handle: &mut Handle<impl Iterator<Item=token::Token>>,
) -> Result<(String, rpc::Struct)> {
    let out = match handle.tokens.next() {                       // Identifier
        None => Error::unexpected_eof("No struct identifier was provided".to_owned()),
        Some(token::Token::Identifier(id)) => match handle.defined_types.structs.get(&id) {
            Some(_) => Error::type_redefined(format!{"Struct with id \"{id}\" already exists"}),
            None => {
                handle.pending_types.structs = Some(id.clone());
                Ok(id)
            },
        },
        Some(t) => Error::unexpected_token("Expected struct identifier".to_owned(), t),
    }.and_then(|id| match handle.tokens.next() {                 // {
        None => Error::unexpected_eof("No struct body".to_owned()),
        Some(token::Token::Bracket(token::Bracket::LeftCurly)) => Ok(id),
        Some(t) => Error::unexpected_token("Expected struct body \"{\"".to_owned(), t),
    }).and_then(|id| parse_struct_body(handle).map(|en| (id, en))) // Body
    .and_then(|pass| match handle.tokens.next() {                // }
        None => Error::unexpected_eof("Struct definition wasn't finished".to_owned()),
        Some(token::Token::Bracket(token::Bracket::RightCurly)) => Ok(pass),
        Some(t) => Error::unexpected_token("Struct definition wasn't finished".to_owned(), t),
    }).and_then(|pass| match handle.tokens.next() {              // ;
        None => Error::unexpected_eof("Struct definition wasn't finished".to_owned()),
        Some(token::Token::Separator(token::Separator::Semicolon)) => Ok(pass),
        Some(t) => Error::expression_not_closed("Struct definition wasn't finished".to_owned(), t),
    }).and_then(|(id, en)| {
        handle.defined_types.structs.insert(id.clone());
        Ok((id, en))
    });

    handle.pending_types.structs = None;

    out
}

fn parse_struct_body(
    handle: &mut Handle<impl Iterator<Item=token::Token>>,
) -> Result<rpc::Struct> {
    let mut st = rpc::new_struct();
    let mut error: Option<Error> = None;

    while match parse_declaration(handle)    // Item
        .and_then(|(id, tp)| match st.get(&id) {
            Some(_) => Error::structure_field_redefined(
                format!("Field with identifier \"{id}\" already exists")
            ),
            None => {
                st.insert(id, tp);

                match handle.tokens.next() { // ;
                    None => Error::unexpected_eof(
                        "Structure field declaraion wasn't finished".to_string(),
                    ),
                    Some(token::Token::Separator(token::Separator::Semicolon)) => Ok(()),
                    Some(t) => Error::unexpected_token(
                        "Structure field declaraion wasn't finished".to_string(),
                        t,
                    ),
                }.and_then(|_| match handle.tokens.next() {
                    None => Ok(false),
                    Some(token::Token::Bracket(token::Bracket::RightCurly)) => {
                        handle.tokens.push_back(token::Token::Bracket(token::Bracket::RightCurly));
                        Ok(false)
                    },
                    Some(t) => {
                        handle.tokens.push_back(t);
                        Ok(true)
                    },
                })
            },
        }) {
        Ok(next) => next,
        Err(err) => {
            error = Some(err);
            false
        },
    } {}

    match error {
        None => Ok(st),
        Some(err) => Err(err),
    }
}

fn parse_union_definition(
    handle: &mut Handle<impl Iterator<Item=token::Token>>,
) -> Result<(String, rpc::Union)> {
    let out = match handle.tokens.next() {                                 // Identifier
        None => Error::unexpected_eof("No union identifier was provided".to_owned()),
        Some(token::Token::Identifier(id)) => match handle.defined_types.unions.get(&id) {
            Some(_) => Error::type_redefined(format!{"Union with id \"{id}\" already exists"}),
            None => {
                handle.pending_types.unions = Some(id.clone());
                Ok(id)
            },
        },
        Some(t) => Error::unexpected_token("Expected union identifier".to_owned(), t),
    }.and_then(|pass| match handle.tokens.next() {                         // switch
        None => Error::unexpected_eof("Keyword \"case\" expected".to_owned()),
        Some(token::Token::Keyword(token::Keyword::Switch)) => Ok(pass),
        Some(t) => Error::unexpected_token("Keyword \"case\" expected".to_owned(), t),
    }).and_then(|pass| match handle.tokens.next() {                        // (
        None => Error::unexpected_eof("Expected \"(\"".to_owned()),
        Some(token::Token::Bracket(token::Bracket::Left)) => Ok(pass),
        Some(t) => Error::unexpected_token("Expected \"(\"".to_owned(), t),
    }).and_then(|id| parse_declaration(handle).and_then(|(sid, tp)|        // Declaration
        match tp {
            rpc::Type::Integer(i) => Ok(rpc::SwitchingType::Integer(i)),
            rpc::Type::Unsigned(u) => Ok(rpc::SwitchingType::Unsigned(u)),
            rpc::Type::Named(rpc::NamedType::Enum(en)) => Ok(rpc::SwitchingType::Enum(en)),
            tp => Error::not_switching_type(tp),
        }.map(|stp| (id, (sid, stp)))
    )).and_then(|pass| match handle.tokens.next() {                        // )
        None => Error::unexpected_eof("Expected \"(\"".to_owned()),
        Some(token::Token::Bracket(token::Bracket::Right)) => Ok(pass),
        Some(t) => Error::unexpected_token("Expected \"(\"".to_owned(), t),
    }).and_then(|pass| match handle.tokens.next() {                        // {
        None => Error::unexpected_eof("No union body".to_owned()),
        Some(token::Token::Bracket(token::Bracket::LeftCurly)) => Ok(pass),
        Some(t) => Error::unexpected_token("Expected union body \"{\"".to_owned(), t),
    }).and_then(|(id, (sid, stp))| parse_union_body(handle).map(|mut un| { // Body
        un.value = sid;
        un.switch_type = stp;
        (id, un)
    })).and_then(|pass| match handle.tokens.next() {                       // }
        None => Error::unexpected_eof("Union definition wasn't finished".to_owned()),
        Some(token::Token::Bracket(token::Bracket::RightCurly)) => Ok(pass),
        Some(t) => Error::unexpected_token("Union definition wasn't finished".to_owned(), t),
    }).and_then(|pass| match handle.tokens.next() {                        // ;
        None => Error::unexpected_eof("Union definition wasn't finished".to_owned()),
        Some(token::Token::Separator(token::Separator::Semicolon)) => Ok(pass),
        Some(t) => Error::expression_not_closed("Union definition wasn't finished".to_owned(), t),
    }).and_then(|(id, en)| {
        handle.defined_types.unions.insert(id.clone());
        Ok((id, en))
    });

    handle.pending_types.unions = None;

    out
}

enum UnionItem {
    Regular(rpc::Value, (String, rpc::Type)),
    Default(String, rpc::Type),
}

fn parse_union_item(
    handle: &mut Handle<impl Iterator<Item=token::Token>>,
) -> Result<UnionItem> {
    match handle.tokens.next() {
        None => Error::unexpected_eof("Matching value expected".to_owned()),
        Some(token::Token::Keyword(token::Keyword::Case)) =>      // Case
            parse_value(handle)                                   // Value
                .and_then(|pass| match handle.tokens.next() {     // :
                    None => Error::unexpected_eof("Colon expected".to_owned()),
                    Some(token::Token::Separator(token::Separator::Colon)) => Ok(pass),
                    Some(t) => Error::unexpected_token("Colon expected".to_owned(), t),
                }).and_then(|v| {
                    parse_declaration(handle)                     // Declaration
                        .map(|decl| UnionItem::Regular(v, decl))
                }),
        Some(token::Token::Keyword(token::Keyword::Default)) => { // Default
            match handle.tokens.next() {                          // :
                None => Error::unexpected_eof("Colon expected".to_owned()),
                Some(token::Token::Separator(token::Separator::Colon)) => Ok(()),
                Some(t) => Error::unexpected_token("Colon expected".to_owned(), t),
            }.and_then(|_| {
                parse_declaration(handle)                         // Declaration
                    .map(|(id, tp)| UnionItem::Default(id, tp))
            })
        },
        Some(t) => Error::unexpected_token("Matching value expected".to_owned(), t),
    }
}

fn parse_union_body(
    handle: &mut Handle<impl Iterator<Item=token::Token>>,
) -> Result<rpc::Union> {
    let mut un = rpc::new_union();
    let mut error: Option<Error> = None;

    while match parse_union_item(handle).and_then(|item| // Item
        match item {
            UnionItem::Regular(v, decl) => match un.arms.get(&v) {
                Some(_) => Error::union_arm_redefined(v),
                None => {
                    un.arms.insert(v, decl);
                    Ok(true)
                },
            },
            UnionItem::Default(id, tp) => match &un.default {
                Some(_) => Error::union_default_redefined(),
                None => {
                    un.default = Some((id, tp));
                    Ok(false)
                },
            },
        }.and_then(|next| match handle.tokens.next() {   // ;
            None => Error::unexpected_eof(
                "Union arm declaraion wasn't finished".to_string(),
            ),
            Some(token::Token::Separator(token::Separator::Semicolon)) => Ok(next),
            Some(t) => Error::unexpected_token(
                "Union arm declaraion wasn't finished".to_string(),
                t,
            ),
        }).and_then(|next| match next {
            false => Ok(false),
            true => match handle.tokens.next() {
                None => Ok(false),
                Some(token::Token::Bracket(token::Bracket::RightCurly)) => {
                    handle.tokens.push_back(token::Token::Bracket(token::Bracket::RightCurly));
                    Ok(false)
                },
                Some(t) => {
                    handle.tokens.push_back(t);
                    Ok(true)
                },
            }
        })
    ) {
        Ok(next) => next,
        Err(err) => {
            error = Some(err);
            false
        },
    } {}

    match error {
        None => Ok(un),
        Some(err) => Err(err),
    }
}

fn parse_program_definition(
    handle: &mut Handle<impl Iterator<Item=token::Token>>,
) -> Result<(rpc::Value, rpc::Program)> {
    match handle.tokens.next() {                                   // Identifier
        None => Error::unexpected_eof("Program identifier expected".to_owned()),
        Some(token::Token::Identifier(id)) => match handle.namespace.get(&id) {
            Some(_) => Error::identifier_redefined(format!(
                "Program identifier \"{id}\" already exists"
            )),
            None => Ok(id),
        },
        Some(t) => Error::unexpected_token("Program identifier expected".to_owned(), t),
    }.and_then(|pass| match handle.tokens.next() {                 // {
        None => Error::unexpected_eof("No program body".to_owned()),
        Some(token::Token::Bracket(token::Bracket::LeftCurly)) => Ok(pass),
        Some(t) => Error::unexpected_token("Expected program body \"{\"".to_owned(), t),
    }).and_then(|id| parse_program_versions(handle).map(|mut pr| { // Body
        pr.name = id;
        pr
    })).and_then(|pass| match handle.tokens.next() {               // }
        None => Error::unexpected_eof("Program body not closed".to_owned()),
        Some(token::Token::Bracket(token::Bracket::RightCurly)) => Ok(pass),
        Some(t) => Error::unexpected_token("Program body not closed".to_owned(), t),
    }).and_then(|pass| match handle.tokens.next() {                // =
        None => Error::unexpected_eof("Number not assigned to program".to_owned()),
        Some(token::Token::Operator(token::Operator::Assign)) => Ok(pass),
        Some(t) => Error::unexpected_token("Number not assigned to program".to_owned(), t),
    }).and_then(|pr| parse_value(handle).map(|v| (v, pr)))         // Value
    .and_then(|(v, pr)| match handle.assigned_numbers.get(&v) {
        Some(_) => Error::program_number_reassigned(v),
        None => Ok((v, pr)),
    }).and_then(|pass| match handle.tokens.next() {                // ;
        None => Error::unexpected_eof("Program definition not closed".to_owned()),
        Some(token::Token::Separator(token::Separator::Semicolon)) => Ok(pass),
        Some(t) => Error::unexpected_token("Program definition not closed".to_owned(), t),
    }).and_then(|(v, pr)| {
        handle.namespace.insert(pr.name.clone());
        handle.assigned_numbers.insert(v.clone());
        Ok((v, pr))
    })
}

fn parse_program_versions(
    handle: &mut Handle<impl Iterator<Item=token::Token>>,
) -> Result<rpc::Program> {
    let mut out = rpc::new_program();
    let mut error: Option<Error> = None;
    let mut version_names = std::collections::HashSet::<String>::new();
    let mut version_values = std::collections::HashSet::<rpc::Value>::new();

    while match parse_version(handle)
        .and_then(|(v, proc)| match version_values.get(&v) {
            Some(_) => Error::version_number_reassigned(v),
            None => match version_names.get(&proc.name) {
                Some(_) => Error::identifier_redefined(format!(
                    "Version with identifier \"{}\" already exists in \
                     current program", proc.name,
                )),
                None => {
                    version_values.insert(v.clone());
                    version_names.insert(proc.name.clone());
                    out.versions.insert(v, proc);
                    Ok(())
                }
            },
        })
        .and_then(|_| Ok(match handle.tokens.next() {
            None => false,
            Some(token::Token::Bracket(token::Bracket::RightCurly)) => {
                handle.tokens.push_back(token::Token::Bracket(token::Bracket::RightCurly));
                false
            },
            Some(t) => {
                handle.tokens.push_back(t);
                true
            },
        })) {
        Ok(next) => next,
        Err(err) => {
            error = Some(err);
            false
        }
    } {}

    match error {
        Some(error) => Err(error),
        None => Ok(out),
    }
}

fn parse_version(
    handle: &mut Handle<impl Iterator<Item=token::Token>>,
) -> Result<(rpc::Value, rpc::Version)> {
    match handle.tokens.next() {                                      // Version
        None => Error::unexpected_eof("Version identifier expected".to_owned()),
        Some(token::Token::Keyword(token::Keyword::Version)) => Ok(()),
        Some(t) => Error::unexpected_token("Version identifier expected".to_owned(), t),
    }.and_then(|_| match handle.tokens.next() {                       // Identifier
        None => Error::unexpected_eof("Version identifier expected".to_owned()),
        Some(token::Token::Identifier(id)) => Ok(id),
        Some(t) => Error::unexpected_token("Version identifier expected".to_owned(), t),
    }).and_then(|pass| match handle.tokens.next() {                    // {
        None => Error::unexpected_eof("No version body".to_owned()),
        Some(token::Token::Bracket(token::Bracket::LeftCurly)) => Ok(pass),
        Some(t) => Error::unexpected_token("Expected version body \"{\"".to_owned(), t),
    }).and_then(|id| parse_version_procedures(handle).map(|mut ver| { // Body
        ver.name = id;
        ver
    })).and_then(|pass| match handle.tokens.next() {                  // }
        None => Error::unexpected_eof("Version body not closed".to_owned()),
        Some(token::Token::Bracket(token::Bracket::RightCurly)) => Ok(pass),
        Some(t) => Error::unexpected_token("Version body not closed".to_owned(), t),
    }).and_then(|pass| match handle.tokens.next() {                   // =
        None => Error::unexpected_eof("Number not assigned to version".to_owned()),
        Some(token::Token::Operator(token::Operator::Assign)) => Ok(pass),
        Some(t) => Error::unexpected_token("Number not assigned to version".to_owned(), t),
    }).and_then(|ver| parse_value(handle).map(|v| (v, ver)))          // Value
    .and_then(|pass| match handle.tokens.next() {                     // ;
        None => Error::unexpected_eof("Version definition not closed".to_owned()),
        Some(token::Token::Separator(token::Separator::Semicolon)) => Ok(pass),
        Some(t) => Error::unexpected_token("Version definition not closed".to_owned(), t),
    })
}

fn parse_version_procedures(
    handle: &mut Handle<impl Iterator<Item=token::Token>>,
) -> Result<rpc::Version> {
    let mut out = rpc::new_version();
    let mut error: Option<Error> = None;
    let mut procedure_names = std::collections::HashSet::<String>::new();
    let mut procedure_values = std::collections::HashSet::<rpc::Value>::new();

    while match parse_procedure(handle)
        .and_then(|(v, proc)| match procedure_values.get(&v) {
            Some(_) => Error::procedure_number_reassigned(v),
            None => match procedure_names.get(&proc.name) {
                Some(_) => Error::identifier_redefined(format!(
                    "Procedure with identifier \"{}\" already exists in \
                     current version", proc.name,
                )),
                None => {
                    procedure_values.insert(v.clone());
                    procedure_names.insert(proc.name.clone());
                    out.procedures.insert(v, proc);
                    Ok(())
                }
            },
        })
        .and_then(|_| Ok(match handle.tokens.next() {
            None => false,
            Some(token::Token::Bracket(token::Bracket::RightCurly)) => {
                handle.tokens.push_back(token::Token::Bracket(token::Bracket::RightCurly));
                false
            },
            Some(t) => {
                handle.tokens.push_back(t);
                true
            },
        })) {
        Ok(next) => next,
        Err(err) => {
            error = Some(err);
            false
        }
    } {}

    match error {
        Some(error) => Err(error),
        None => Ok(out),
    }
}

fn parse_procedure(
    handle: &mut Handle<impl Iterator<Item=token::Token>>,
) -> Result<(rpc::Value, rpc::Procedure)> {
    parse_type(handle).and_then(|tp| match handle.tokens.next() {        // Type + Identifier
        None => Error::unexpected_eof("Procedure identifier expected".to_owned()),
        Some(token::Token::Identifier(id)) => Ok((tp, id)),
        Some(t) => Error::unexpected_token("Procedure identifier expected".to_owned(), t),
    }).and_then(|pass| match handle.tokens.next() {                      // (
        None => Error::unexpected_eof("No procedure body".to_owned()),
        Some(token::Token::Bracket(token::Bracket::Left)) => Ok(pass),
        Some(t) => Error::unexpected_token("Expected procedure body \"{\"".to_owned(), t),
    }).and_then(|(tp, id)| parse_procedure_args(handle).map(|mut proc| { // Args
        proc.name = id;
        proc.return_type = tp;
        proc
    })).and_then(|pass| match handle.tokens.next() {                     // )
        None => Error::unexpected_eof("Procedure body not closed".to_owned()),
        Some(token::Token::Bracket(token::Bracket::Right)) => Ok(pass),
        Some(t) => Error::unexpected_token("Procedure body not closed".to_owned(), t),
    }).and_then(|pass| match handle.tokens.next() {                      // =
        None => Error::unexpected_eof("Number not assigned to procedure".to_owned()),
        Some(token::Token::Operator(token::Operator::Assign)) => Ok(pass),
        Some(t) => Error::unexpected_token("Number not assigned to procedure".to_owned(), t),
    }).and_then(|proc| parse_value(handle).map(|v| (v, proc)))           // Value
    .and_then(|pass| match handle.tokens.next() {                        // ;
        None => Error::unexpected_eof("Procedure definition not closed".to_owned()),
        Some(token::Token::Separator(token::Separator::Semicolon)) => Ok(pass),
        Some(t) => Error::unexpected_token("Procedure definition not closed".to_owned(), t),
    })
}

fn parse_procedure_args(
    handle: &mut Handle<impl Iterator<Item=token::Token>>,
) -> Result<rpc::Procedure> {
    let mut out = rpc::new_procedure();
    let mut error: Option<Error> = None;

    while match parse_type(handle)              // Type
        .and_then(|tp| match tp {
            rpc::Type::Void if 0 == out.arguments.len() => Ok(false),
            _ => {
                out.arguments.push(tp);
                Ok(match handle.tokens.next() { // [,]
                    None => false,
                    Some(token::Token::Separator(token::Separator::Comma)) => true,
                    Some(t) => {
                        handle.tokens.push_back(t);
                        false
                    },
                })
            }
        }) {
        Ok(next) => next,
        Err(err) => {
            error = Some(err);
            false
        }
    } {}

    match error {
        Some(error) => Err(error),
        None => Ok(out),
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownDefinition(token) => write!(f,
                "Expected definition, got: {token:?}"
            ),
            Error::UnexpectedToken(msg, token) => write!(f,
                "Unexpected token: {token:?}\n{msg}"
            ),
            Error::ExpressionNotClosed(msg, token) => write!(f,
                "Expression not closed, got token: {token:?}\n{msg}"
            ),
            Error::UnexpectedEOF(msg) => write!(f, "Unexpected EOF\n{msg}"),
            Error::UndefinedType(msg) => write!(f, "Undefined type\n{msg}"),
            Error::UndefinedValue(msg) => write!(f, "Undefined value: {msg}"),
            Error::NonPositiveArraySize(size) => write!(f,
                "Array size must be greater than 0, got {size}"
            ),
            Error::TypeRedefined(msg) => write!(f, "Type redefined\n{msg}"),
            Error::IdentifierRedefined(msg) => write!(f, "Identifier redefined\n{msg}"),
            Error::StructureFieldRedefined(msg) => write!(f, "Structure field redefined\n{msg}"),
            Error::NotSwitchingType(tp) => write!(f,
                "Only integer type can be used for union identifier, got {tp:?}"
            ),
            Error::UnionArmRedefined(arm) => match arm {
                UnionArm::Regular(v) => write!(f, "Union arm for value {v:?} redefined"),
                UnionArm::Default => write!(f, "Default union arm redefined"),
            },
            Error::UseOfPendingType(msg) => write!(f,
                "Use of type being defined\n{msg}"
            ),
            Error::ProgramNumberReassigned(value) => write!(f,
                "Program with number {value:?} redefined"
            ),
            Error::VersionNumberReassigned(value) => write!(f,
                "Version with number {value:?} redefined"
            ),
            Error::ProcedureNumberReassigned(value) => write!(f,
                "Procedure with number {value:?} redefined"
            ),
        }
    }
}

