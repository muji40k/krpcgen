
pub fn value(v: &rpc::Value) -> String {
    match v {
        rpc::Value::Number(num) => num.to_string(),
        rpc::Value::Identifier(id) => id.clone(),
    }
}

fn append_or_new(tp: Option<String>, new: &str) -> String {
    tp.map(|rest| format!("{new}{rest}")).unwrap_or_else(|| new.to_string())
}

fn inner_fulltype(tp: &rpc::Type, buf: (Option<String>, Option<String>)) -> (String, Option<String>) {
    match tp {
        rpc::Type::Void => (format!("void{}", buf.0.expect("Void type occured")), buf.1),
        rpc::Type::Integer(integer) => match integer {
            rpc::Integer::Integer => (append_or_new(buf.0, "s32"), buf.1),
            rpc::Integer::Hyper => (append_or_new(buf.0, "s64"), buf.1),
        },
        rpc::Type::Unsigned(integer) => match integer {
            rpc::Integer::Integer => (append_or_new(buf.0, "u32"), buf.1),
            rpc::Integer::Hyper => (append_or_new(buf.0, "u64"), buf.1),
        },
        rpc::Type::Float(float) => match float {
            rpc::Float::Single => (append_or_new(buf.0, "float"), buf.1),
            rpc::Float::Double => (append_or_new(buf.0, "double"), buf.1),
            rpc::Float::Quadruple => panic!("Haha eventually..."),
        },
        rpc::Type::Boolean => (append_or_new(buf.0, "bool"), buf.1),
        rpc::Type::String => (append_or_new(buf.0, "char"), buf.1),
        rpc::Type::Opaque => (append_or_new(buf.0, "char"), buf.1),
        rpc::Type::Pointer(tp) => inner_fulltype(tp, (
            buf.0.map(|rest| format!("*{rest}"))
                .or_else(|| Some(String::from("*"))),
            buf.1,
        )),
        rpc::Type::Array(tp, sz) => inner_fulltype(tp, (
            buf.0,
            buf.1.map(|rest| format!("{rest}[{}]", value(sz)))
                .or_else(|| Some(format!("[{}]", value(sz)))),
        )),
        rpc::Type::VArray(tp, _) => match tp.as_ref() {
            rpc::Type::String => (append_or_new(buf.0, "string_t"), buf.1),
            _ => (append_or_new(buf.0, &format!("vla({})", typename(&fulltype(tp)))), buf.1),
        },
        rpc::Type::Named(named) => match named {
            rpc::NamedType::Typedef(name) =>
                (append_or_new(buf.0, name), buf.1),
            rpc::NamedType::Enum(name) =>
                (append_or_new(buf.0, &format!("enum {name}")), buf.1),
            rpc::NamedType::Struct(name) | rpc::NamedType::Union(name) =>
                (append_or_new(buf.0, &format!("struct {name}")), buf.1),
        }
    }
}

pub fn fulltype(tp: &rpc::Type) -> (String, Option<String>) {
    inner_fulltype(tp, (None, None))
}

pub fn declaration(name: &str, tp: &(String, Option<String>)) -> String {
    match tp {
        (tname, Some(arr)) => tname.to_owned() + " " + name + &arr,
        (tname, None) => tname.to_owned() + " " + name,
    }
}

pub fn pointer_declaration(name: &str, tp: &(String, Option<String>)) -> String {
    match tp {
        (tname, Some(arr)) => tname.to_string() + " (*" + name + ")" + &arr,
        (tname, None) => tname.to_string() + "* " + name,
    }
}

pub fn typename(tp: &(String, Option<String>)) -> String {
    let (tname, arr) = tp;
    tname.to_string() + arr.as_ref().map(String::as_str).unwrap_or("")
}

pub fn switching_declaraion(name: &str, tp: &rpc::SwitchingType) -> String {
    match tp {
        rpc::SwitchingType::Integer(integer) => match integer {
            rpc::Integer::Integer => format!("s32 {name}"),
            rpc::Integer::Hyper => format!("s64 {name}"),
        }
        rpc::SwitchingType::Unsigned(integer) => match integer {
            rpc::Integer::Integer => format!("u32 {name}"),
            rpc::Integer::Hyper => format!("u64 {name}"),
        },
        rpc::SwitchingType::Enum(ename) => format!("enum {ename} {name}"),
    }
}

