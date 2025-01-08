use super::*;

macro_rules! assert_program {
    ( $progr: expr, #$number: expr => $prog_name: ident [$size: expr] { $($rest: tt)* }) => {
        {
            let (v, progr) = $progr;
            assert_eq!(&rpc::Value::Number($number), v);
            assert_eq!(stringify!{$prog_name}, progr.name);
            assert_eq!($size, progr.versions.len());
            assert_versions!(progr.versions, $($rest)*);
        }
    };
    ( $progr: expr, $number: ident => $prog_name: ident [$size: expr] { $($rest: tt)* }) => {
        {
            let (v, progr) = $progr;
            assert_eq!(&rpc::Value::Identifier(String::from(stringify!{$number})), v);
            assert_eq!(stringify!{$prog_name}, progr.name);
            assert_eq!($size, progr.versions.len());
            assert_versions!(progr.versions, $($rest)*);
        }
    };
}

macro_rules! assert_versions{
    ( $vers: expr, ) => {};
    ( $vers: expr, #$number: expr => $version_name: ident [$size: expr] { $($procs: tt)* }, $($rest: tt)* ) => {
        {
            let v = rpc::Value::Number($number);
            match $vers.get(&v) {
                None => panic!("No version with value {v:?} found"),
                Some(ver) => {
                    assert_eq!(stringify!{$version_name}, ver.name);
                    assert_eq!($size, ver.procedures.len());
                    assert_procedures!(ver.procedures, $($procs)*);
                }
            }
        }
        assert_versions!($vers, $($rest)*);
    };
    ( $vers: expr, $number: ident => $version_name: ident [$size: expr] { $($procs: tt)* }, $($rest: tt)* ) => {
        {
            let v = rpc::Value::Identifier(String::from(stringify!{$number}));
            match $progr.get(&v) {
                None => panic!("No version with value {v:?} found"),
                Some(ver) => {
                    assert_eq!(stringify!{$version_name}, ver.name);
                    assert_eq!($size, ver.procedures.len());
                    assert_procedures!(ver.procedures, $($procs)*);
                }
            }
        }
        assert_versions!($vers, $($rest)*);
    };
}

macro_rules! assert_procedures{
    ( $proc: expr, ) => {};
    ( $proc: expr, #$number: expr => $proc_name: ident ($($type: expr),+) => $rtype: expr, $($rest: tt)* ) => {
        {
            let v = rpc::Value::Number($number);
            match $proc.get(&v) {
                None => panic!("No procedure with value {v:?} found"),
                Some(proc) => {
                    assert_eq!(stringify!{$proc_name}, proc.name);
                    assert_eq!(vec![$($type),+], proc.arguments);
                    assert_eq!($rtype, proc.return_type);
                }
            }
        }
        assert_procedures!($proc, $($rest)*);
    };
    ( $proc: expr, $number: ident => $proc_name: ident ($($type: expr),+) => $rtype: expr, $($rest: tt)* ) => {
        {
            let v = rpc::Value::Identifier(String::from(stringify!{$number}));
            match $proc.get(&v) {
                None => panic!("No procedure with value {v:?} found"),
                Some(proc) => {
                    assert_eq!(stringify!{$proc_name}, proc.name);
                    assert_eq!(vec![$($type),+], proc.arguments);
                    assert_eq!($rtype, proc.return_type);
                }
            }
        }
        assert_procedures!($proc, $($rest)*);
    };
}

macro_rules! check_struct {
    ( $struct: expr, $name: ident [$size: expr] => { $($defs: tt)* } ) => {
        {
            let (name, st) = $struct;
            assert_eq!(stringify!{$name}, name.as_str());
            assert_eq!($size, st.len());
            check_struct_field!(st, $($defs)*);
        }
    };
}

macro_rules! check_struct_field {
    ( $struct: expr, ) => {};
    ( $struct: expr, $field: ident: $type: expr; $($rest: tt)* ) => {
        {
            match $struct.get(stringify!{$field}) {
                None => panic!("No field \"{}\" found", stringify!{$field}),
                Some(tp) => if *tp != $type {
                    panic!("type {:?} for field \"{}\" expected, got {tp:?}", $type, stringify!{$field});
                }
            }
        }
        check_struct_field!($struct, $($rest)*);
    };

}

fn ping_progr() -> [token::Token; 55] { [
    token::Token::Comment("\n * Simple ping program\n ".to_string()),
    token::Token::Keyword(token::Keyword::Program), token::Token::Identifier("PING_PROG".to_string()), token::Token::Bracket(token::Bracket::LeftCurly),
        token::Token::Keyword(token::Keyword::Version), token::Token::Identifier("PING_VERS_PINGBACK".to_string()), token::Token::Bracket(token::Bracket::LeftCurly),
            token::Token::Type(token::Type::Void), token::Token::Identifier("PINGPROC_NULL".to_string()), token::Token::Bracket(token::Bracket::Left), token::Token::Type(token::Type::Void), token::Token::Bracket(token::Bracket::Right),
            token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(0)), token::Token::Separator(token::Separator::Semicolon),
            token::Token::Comment(String::from(
                 "
                 * ping the caller, return the round-trip time
                 * in milliseconds. Return a minus one (-1) if
                 * operation times-out
                 "
            )),
            token::Token::Type(token::Type::Integer), token::Token::Identifier("PINGPROC_PINGBACK".to_string()), token::Token::Bracket(token::Bracket::Left), token::Token::Type(token::Type::Void), token::Token::Bracket(token::Bracket::Right),
            token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(1)), token::Token::Separator(token::Separator::Semicolon),
            token::Token::Comment(String::from(
                " void - above is an argument to the call "
            )),
        token::Token::Bracket(token::Bracket::RightCurly),
        token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(2)), token::Token::Separator(token::Separator::Semicolon),
        token::Token::Comment(String::from("\n * Original version\n ")),
        token::Token::Keyword(token::Keyword::Version), token::Token::Identifier("PING_VERS_ORIG".to_string()), token::Token::Bracket(token::Bracket::LeftCurly),
            token::Token::Type(token::Type::Void), token::Token::Identifier("PINGPROC_NULL".to_string()), token::Token::Bracket(token::Bracket::Left), token::Token::Type(token::Type::Void), token::Token::Bracket(token::Bracket::Right),
            token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(0)), token::Token::Separator(token::Separator::Semicolon),
        token::Token::Bracket(token::Bracket::RightCurly),
        token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(1)), token::Token::Separator(token::Separator::Semicolon),
    token::Token::Bracket(token::Bracket::RightCurly), token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(200000)), token::Token::Separator(token::Separator::Semicolon),
    token::Token::Keyword(token::Keyword::Const), token::Token::Identifier("PING_VERS".to_string()), token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(2)), token::Token::Separator(token::Separator::Semicolon),
    token::Token::Comment(String::from(" latest version ")),
] }

#[test]
fn ping() {
    let module = parse(ping_progr().into_iter()).unwrap();
    let mut defs = module.definitions.iter();

    match defs.next() {
        Some(rpc::Definition::Program(v, progr)) => assert_program!((v, progr),
            #200000 => PING_PROG[2] {
                #2 => PING_VERS_PINGBACK[2] {
                    #0 => PINGPROC_NULL(rpc::Type::Void) => rpc::Type::Void,
                    #1 => PINGPROC_PINGBACK(rpc::Type::Void) => rpc::Type::Integer(rpc::Integer::Integer),
                },
                #1 => PING_VERS_ORIG[1] {
                    #0 => PINGPROC_NULL(rpc::Type::Void) => rpc::Type::Void,
                },
            }
        ),
        _ => panic!("Program expected"),
    }

    match defs.next() {
        Some(rpc::Definition::Const(id, v)) => {
            assert_eq!("PING_VERS", id.as_str());
            assert_eq!(&rpc::Value::Number(2), v);
        },
        _ => panic!("Const expected"),
    }

    if let Some(_) = defs.next() {
        panic!("End expected")
    }
}

fn bakery_progr() -> [token::Token; 121] { [
    token::Token::Keyword(token::Keyword::Const), token::Token::Identifier("REGISTER".to_string()), token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(0)), token::Token::Separator(token::Separator::Semicolon),
    token::Token::Keyword(token::Keyword::Const), token::Token::Identifier("ACCESS".to_string()), token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(1)), token::Token::Separator(token::Separator::Semicolon),
    token::Token::Keyword(token::Keyword::Const), token::Token::Identifier("GET".to_string()), token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(2)), token::Token::Separator(token::Separator::Semicolon),
    token::Token::Keyword(token::Keyword::Const), token::Token::Identifier("STATUS".to_string()), token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(3)), token::Token::Separator(token::Separator::Semicolon),
    token::Token::Keyword(token::Keyword::Const), token::Token::Identifier("OP_MAX".to_string()), token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(4)), token::Token::Separator(token::Separator::Semicolon),
    token::Token::Keyword(token::Keyword::Const), token::Token::Identifier("STATUS_FREE".to_string()), token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(0)), token::Token::Separator(token::Separator::Semicolon),
    token::Token::Keyword(token::Keyword::Const), token::Token::Identifier("STATUS_REGISTERED".to_string()), token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(1)), token::Token::Separator(token::Separator::Semicolon),
    token::Token::Keyword(token::Keyword::Const), token::Token::Identifier("STATUS_ACCESSING".to_string()), token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(2)), token::Token::Separator(token::Separator::Semicolon),
    token::Token::Keyword(token::Keyword::Const), token::Token::Identifier("STATUS_READY_FOR_CR".to_string()), token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(3)), token::Token::Separator(token::Separator::Semicolon),
    token::Token::Keyword(token::Keyword::Const), token::Token::Identifier("ERROR_WRONG_ID_RPC".to_string()), token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(-1)), token::Token::Separator(token::Separator::Semicolon),
    token::Token::Keyword(token::Keyword::Const), token::Token::Identifier("ERROR_INCOMPATIBLE_HANLE_RPC".to_string()), token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(-2)), token::Token::Separator(token::Separator::Semicolon),
    token::Token::Keyword(token::Keyword::Const), token::Token::Identifier("ERROR_WRONG_STATUS_RPC".to_string()), token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(-3)), token::Token::Separator(token::Separator::Semicolon),
    token::Token::Keyword(token::Keyword::Const), token::Token::Identifier("ERROR_REJECT_ACCESS_RPC".to_string()), token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(-4)), token::Token::Separator(token::Separator::Semicolon),
    token::Token::Keyword(token::Keyword::Const), token::Token::Identifier("ERROR_WRONG_OP_RPC".to_string()), token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(-5)), token::Token::Separator(token::Separator::Semicolon),
    token::Token::Type(token::Type::Struct), token::Token::Identifier("BAKERY".to_string()), token::Token::Bracket(token::Bracket::LeftCurly),
        token::Token::Type(token::Type::Integer), token::Token::Identifier("op".to_string()), token::Token::Separator(token::Separator::Semicolon),
        token::Token::Type(token::Type::Integer), token::Token::Identifier("id".to_string()), token::Token::Separator(token::Separator::Semicolon), token::Token::Type(token::Type::Integer), token::Token::Identifier("num".to_string()), token::Token::Separator(token::Separator::Semicolon),
        token::Token::Type(token::Type::Integer), token::Token::Identifier("result".to_string()), token::Token::Separator(token::Separator::Semicolon),
        token::Token::Type(token::Type::Struct), token::Token::Identifier("BAKERY".to_string()), token::Token::Type(token::Type::Pointer), token::Token::Identifier("not_supposed_2_be_here".to_string()), token::Token::Separator(token::Separator::Semicolon),
    token::Token::Bracket(token::Bracket::RightCurly), token::Token::Separator(token::Separator::Semicolon),
    token::Token::Keyword(token::Keyword::Typedef), token::Token::Type(token::Type::Struct), token::Token::Identifier("BAKERY".to_string()), token::Token::Identifier("BAKERY".to_string()), token::Token::Separator(token::Separator::Semicolon),
    token::Token::Keyword(token::Keyword::Program), token::Token::Identifier("BAKERY_PROG".to_string()), token::Token::Bracket(token::Bracket::LeftCurly),
        token::Token::Keyword(token::Keyword::Version), token::Token::Identifier("BAKERY_VER".to_string()), token::Token::Bracket(token::Bracket::LeftCurly),
            token::Token::Type(token::Type::Struct), token::Token::Identifier("BAKERY".to_string()), token::Token::Identifier("BAKERY_PROC".to_string()), token::Token::Bracket(token::Bracket::Left), token::Token::Type(token::Type::Struct), token::Token::Identifier("BAKERY".to_string()), token::Token::Bracket(token::Bracket::Right),
            token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(1)), token::Token::Separator(token::Separator::Semicolon),
        token::Token::Bracket(token::Bracket::RightCurly),
        token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(1)), token::Token::Separator(token::Separator::Semicolon),
    token::Token::Bracket(token::Bracket::RightCurly),
    token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(0x20000001)), token::Token::Separator(token::Separator::Semicolon),
] }

#[test]
fn bakery() {
    let module = parse(bakery_progr().into_iter()).unwrap();
    let mut defs = module.definitions.iter();

    match defs.next() {
        Some(rpc::Definition::Const(id, v)) => {
            assert_eq!("REGISTER", id.as_str());
            assert_eq!(&rpc::Value::Number(0), v);
        },
        _ => panic!("Const expected"),
    }
    match defs.next() {
        Some(rpc::Definition::Const(id, v)) => {
            assert_eq!("ACCESS", id.as_str());
            assert_eq!(&rpc::Value::Number(1), v);
        },
        _ => panic!("Const expected"),
    }
    match defs.next() {
        Some(rpc::Definition::Const(id, v)) => {
            assert_eq!("GET", id.as_str());
            assert_eq!(&rpc::Value::Number(2), v);
        },
        _ => panic!("Const expected"),
    }
    match defs.next() {
        Some(rpc::Definition::Const(id, v)) => {
            assert_eq!("STATUS", id.as_str());
            assert_eq!(&rpc::Value::Number(3), v);
        },
        _ => panic!("Const expected"),
    }
    match defs.next() {
        Some(rpc::Definition::Const(id, v)) => {
            assert_eq!("OP_MAX", id.as_str());
            assert_eq!(&rpc::Value::Number(4), v);
        },
        _ => panic!("Const expected"),
    }

    match defs.next() {
        Some(rpc::Definition::Const(id, v)) => {
            assert_eq!("STATUS_FREE", id.as_str());
            assert_eq!(&rpc::Value::Number(0), v);
        },
        _ => panic!("Const expected"),
    }
    match defs.next() {
        Some(rpc::Definition::Const(id, v)) => {
            assert_eq!("STATUS_REGISTERED", id.as_str());
            assert_eq!(&rpc::Value::Number(1), v);
        },
        _ => panic!("Const expected"),
    }
    match defs.next() {
        Some(rpc::Definition::Const(id, v)) => {
            assert_eq!("STATUS_ACCESSING", id.as_str());
            assert_eq!(&rpc::Value::Number(2), v);
        },
        _ => panic!("Const expected"),
    }
    match defs.next() {
        Some(rpc::Definition::Const(id, v)) => {
            assert_eq!("STATUS_READY_FOR_CR", id.as_str());
            assert_eq!(&rpc::Value::Number(3), v);
        },
        _ => panic!("Const expected"),
    }

    match defs.next() {
        Some(rpc::Definition::Const(id, v)) => {
            assert_eq!("ERROR_WRONG_ID_RPC", id.as_str());
            assert_eq!(&rpc::Value::Number(-1), v);
        },
        _ => panic!("Const expected"),
    }
    match defs.next() {
        Some(rpc::Definition::Const(id, v)) => {
            assert_eq!("ERROR_INCOMPATIBLE_HANLE_RPC", id.as_str());
            assert_eq!(&rpc::Value::Number(-2), v);
        },
        _ => panic!("Const expected"),
    }
    match defs.next() {
        Some(rpc::Definition::Const(id, v)) => {
            assert_eq!("ERROR_WRONG_STATUS_RPC", id.as_str());
            assert_eq!(&rpc::Value::Number(-3), v);
        },
        _ => panic!("Const expected"),
    }
    match defs.next() {
        Some(rpc::Definition::Const(id, v)) => {
            assert_eq!("ERROR_REJECT_ACCESS_RPC", id.as_str());
            assert_eq!(&rpc::Value::Number(-4), v);
        },
        _ => panic!("Const expected"),
    }
    match defs.next() {
        Some(rpc::Definition::Const(id, v)) => {
            assert_eq!("ERROR_WRONG_OP_RPC", id.as_str());
            assert_eq!(&rpc::Value::Number(-5), v);
        },
        _ => panic!("Const expected"),
    }

    match defs.next() {
        Some(rpc::Definition::Struct(id, st)) => {
            check_struct!((id, st),
                BAKERY[5] => {
                    op: rpc::Type::Integer(rpc::Integer::Integer);
                    id: rpc::Type::Integer(rpc::Integer::Integer);
                    num: rpc::Type::Integer(rpc::Integer::Integer);
                    result: rpc::Type::Integer(rpc::Integer::Integer);
                    not_supposed_2_be_here: rpc::Type::Pointer(Box::new(rpc::Type::Named(rpc::NamedType::Struct(String::from("BAKERY")))));
                }
            );
        },
        _ => panic!("Const expected"),
    }

    match defs.next() {
        Some(rpc::Definition::Typedef(id, tp)) => {
            assert_eq!("BAKERY", id.as_str());
            assert_eq!(&rpc::Type::Named(rpc::NamedType::Struct(String::from("BAKERY"))), tp);
        }
        _ => panic!("Typedef expected"),
    }

    match defs.next() {
        Some(rpc::Definition::Program(v, progr)) => assert_program!((v, progr),
            #0x20000001 => BAKERY_PROG[1] {
                #1 => BAKERY_VER[1] {
                    #1 => BAKERY_PROC(
                        rpc::Type::Named(rpc::NamedType::Struct(String::from("BAKERY")))
                    ) => rpc::Type::Named(rpc::NamedType::Struct(String::from("BAKERY"))),
                },
            }
        ),
        _ => panic!("Program expected"),
    }


    if let Some(_) = defs.next() {
        panic!("End expected")
    }
}

fn enum_test_p() -> [token::Token; 16] { [
    token::Token::Type(token::Type::Enum), token::Token::Identifier("Test".to_string()), token::Token::Bracket(token::Bracket::LeftCurly),
        token::Token::Identifier("A".to_string()), token::Token::Separator(token::Separator::Comma),
        token::Token::Identifier("B".to_string()), token::Token::Operator(token::Operator::Assign), token::Token::Literal(token::Literal::Integer(3)), token::Token::Separator(token::Separator::Comma),
        token::Token::Identifier("C".to_string()), token::Token::Operator(token::Operator::Assign), token::Token::Identifier("A".to_string()), token::Token::Separator(token::Separator::Comma),
        token::Token::Identifier("D".to_string()),
    token::Token::Bracket(token::Bracket::RightCurly), token::Token::Separator(token::Separator::Semicolon),
] }

#[test]
fn enum_test() {
    let module = parse(enum_test_p().into_iter()).unwrap();
    let mut defs = module.definitions.iter();

    match defs.next() {
        Some(rpc::Definition::Enum(id, en)) => {
            assert_eq!("Test", id.as_str());
            assert_eq!(&vec![
                ("A".to_string(), None),
                ("B".to_string(), Some(rpc::Value::Number(3))),
                ("C".to_string(), Some(rpc::Value::Identifier("A".to_string()))),
                ("D".to_string(), None),
            ], en);
        },
        _ => panic!("Enum expected"),
    }

    if let Some(_) = defs.next() {
        panic!("End expected")
    }
}


