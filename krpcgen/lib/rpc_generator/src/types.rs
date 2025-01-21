
mod printables;
pub mod asc;

use crate::{
    handle,
    config,
    file::{ File, Printable, IteratorPrinter },
};

pub fn uses_dynamic_memory(handle: &handle::Handle, tp: &rpc::Type) -> bool {
    match tp {
        rpc::Type::Void => false,
        rpc::Type::Integer(_) => false,
        rpc::Type::Unsigned(_) => false,
        rpc::Type::Float(_) => false,
        rpc::Type::Boolean => false,
        rpc::Type::String => false,
        rpc::Type::Opaque => false,
        rpc::Type::Pointer(_) => true,
        rpc::Type::Array(tp, _) => uses_dynamic_memory(handle, tp),
        rpc::Type::VArray(_, _) => true,
        rpc::Type::Named(named) => match named {
            rpc::NamedType::Typedef(name) =>
                uses_dynamic_memory(handle,
                    handle.module.types.typedefs.get(name).expect("Was added")
                ),
            rpc::NamedType::Enum(_) => false,
            rpc::NamedType::Struct(name) => handle.module.types.structs.get(name).expect("Was added")
                .values()
                .any(|tp| uses_dynamic_memory(handle, tp)),
            rpc::NamedType::Union(name) => {
                let un = handle.module.types.unions.get(name).expect("Was added");
                un.arms.values().chain(un.default.iter())
                    .any(|(_, tp)| uses_dynamic_memory(handle, tp))
            },
        }
    }
}

fn append_or_self(s: Option<String>, current: String) -> String {
    s.map(|out| out + "+" + &current).unwrap_or(current)
}

fn generate_xdr_size_inner(handle: &handle::Handle, tp: &rpc::Type, out: Option<String>) -> String {
    match tp {
        rpc::Type::Void => append_or_self(out, String::from("0")),
        rpc::Type::Pointer(tp) => generate_xdr_size_inner(handle, tp,
            Some(append_or_self(out, format!("sizeof(u32)")))
        ),
        rpc::Type::Array(tp, sz) => append_or_self(out,
            format!("{}*({})", asc::value(sz), generate_xdr_size(handle, tp))
        ),
        rpc::Type::VArray(tp, sz) => {
            append_or_self(out,
                format!("sizeof(u32)+{}*({})",
                    sz.as_ref().map_or_else(
                        || String::from("VLA_LIMIT"),
                        asc::value,
                    ),
                    generate_xdr_size(handle, tp)
                )
            )
        }
        rpc::Type::Named(named) => match named {
            rpc::NamedType::Typedef(name) => generate_xdr_size_inner(
                handle,
                handle.module.types.typedefs.get(name).expect("Was added"),
                out
            ),
            rpc::NamedType::Enum(name) =>
                append_or_self(out, format!("sizeof(enum {name})")),
            rpc::NamedType::Struct(name) => handle.module.types.structs.get(name)
                .expect("Was added").values()
                .fold(out, |out, tp| {
                    Some(append_or_self(out, generate_xdr_size(handle, tp)))
                }).expect("At least one field in struct"),
            rpc::NamedType::Union(name) => {
                let un = handle.module.types.unions.get(name).expect("Was added");

                append_or_self(out, format!("{}+{}",
                    generate_switch_xdr_size(handle, &un.switch_type),
                    un.arms.values().chain(un.default.iter()).map(|(_, v)| v)
                        .map(|current| generate_xdr_size(handle, current))
                        .reduce(|prev, next| format!("STATIC_MAX(({prev}),({next}))"))
                        .expect("At least one field in union")
                ))
            },
        },
        _ => append_or_self(out, format!("sizeof({})", asc::typename(&asc::fulltype(tp)))),
    }
}

pub fn generate_xdr_size(handle: &handle::Handle, tp: &rpc::Type) -> String {
    generate_xdr_size_inner(handle, tp, None)
}

fn generate_switch_xdr_size(handle: &handle::Handle, tp: &rpc::SwitchingType) -> String {
    match tp.clone() {
        rpc::SwitchingType::Integer(integer) =>
            generate_xdr_size(handle, &rpc::Type::Integer(integer)),
        rpc::SwitchingType::Unsigned(integer) =>
            generate_xdr_size(handle, &rpc::Type::Unsigned(integer)),
        rpc::SwitchingType::Enum(name) =>
            generate_xdr_size(handle, &rpc::Type::Named(rpc::NamedType::Enum(name))),
    }
}

pub fn generate_release_statement(
    handle: &handle::Handle,
    file: &mut dyn File,
    tp: &rpc::Type,
    access: &str,
    offset: Option<usize>,
) {
    if !uses_dynamic_memory(handle, tp) {
        return;
    }

    let offset = offset.unwrap_or(0);
    let soffset = (0..offset).map(|_| ' ').collect::<String>();

    match tp {
        rpc::Type::Pointer(inner) => {
            format!("{soffset}if (NULL != {access}) {{").print(file);
            generate_release_statement(handle, file, inner, &format!("*({access})"), Some(offset + 4));
            format!("{soffset}    kfree({access});").print(file);
            format!("{soffset}    {access} = NULL;").print(file);
            format!("{soffset}}}").print(file);
        },
        rpc::Type::Array(tp, sz) => {
            format!("{soffset}for (size_t i = 0; {} > i; i++) {{", asc::value(sz)).print(file);
            generate_release_statement(handle, file, tp, &format!("({access})[i]"), Some(offset + 4));
            format!("{soffset}}}").print(file);
        },
        rpc::Type::VArray(tp, _) => {
            format!("{soffset}if (NULL != ({access}).data) {{").print(file);
            if uses_dynamic_memory(handle, tp) {
                let ctype = asc::fulltype(tp);
                format!("{soffset}    {} = ({})(({access}).data);",
                    asc::pointer_declaration("_base", &ctype),
                    asc::pointer_declaration("", &ctype),
                ).print(file);
                format!("{soffset}    {} = _base;",
                    asc::pointer_declaration("base", &ctype),
                ).print(file);
                format!("{soffset}    for (size_t i = 0; ({access}).size > i; i++) {{").print(file);
                generate_release_statement(handle, file, tp, "base[i]", Some(offset + 8));
                format!("{soffset}    }}").print(file);
            }
            format!("{soffset}    kfree(({access}).data);").print(file);
            format!("{soffset}    ({access}).data = NULL;").print(file);
            format!("{soffset}    ({access}).size = 0;").print(file);
            format!("{soffset}}}").print(file);
        },
        rpc::Type::Named(named) => match named {
            rpc::NamedType::Struct(name) => {
                let st = handle.module.types.structs.get(name).expect("Was added");
                format!("{soffset}{{ // struct {name}").print(file);
                st.iter().for_each(|(field, tp)| generate_release_statement(
                    handle, file, tp,
                    &format!("({access}).{field}"),
                    Some(offset + 4),
                ));
                format!("{soffset}}}").print(file);
            },
            rpc::NamedType::Union(name) => {
                let un = handle.module.types.unions.get(name).expect("Was added");
                format!("{soffset}{{ // union {name}").print(file);
                format!("{soffset}    switch (({access}).{}) {{", un.value).print(file);
                un.arms.iter().for_each(|(v, (field, tp))| {
                    format!("{soffset}    case ({}):", asc::value(v)).print(file);
                    generate_release_statement(handle, file, tp,
                        &format!("({access}).{name}_u.{field}"),
                        Some(offset + 8),
                    );
                    format!("{soffset}        break;").print(file);
                });
                if let Some((field, tp)) = &un.default {
                    format!("{soffset}    default:").print(file);
                    generate_release_statement(handle, file, tp,
                        &format!("({access}).{name}_u.{field}"),
                        Some(offset + 8),
                    );
                    format!("{soffset}        break;").print(file);
                }
                format!("{soffset}    }}").print(file);
                format!("{soffset}}}").print(file);
            },
            rpc::NamedType::Typedef(name) => {
                generate_release_statement(handle, file,
                    handle.module.types.typedefs.get(name).expect("Was added"),
                    access, Some(offset)
                );
            },
            _ => {},
        },
        _ => {},
    }
}

fn generate_switch_decode_statement(
    handle: &handle::Handle,
    file: &mut dyn File,
    tp: &rpc::SwitchingType,
    access: &str,
    rc: Option<&str>,
    offset: Option<usize>,
) {
    match tp.clone() {
        rpc::SwitchingType::Integer(integer) => generate_decode_statement(
            handle, file, &rpc::Type::Integer(integer), access, rc, offset,
        ),
        rpc::SwitchingType::Unsigned(integer) => generate_decode_statement(
            handle, file, &rpc::Type::Unsigned(integer), access, rc, offset,
        ),
        rpc::SwitchingType::Enum(name) => generate_decode_statement(
            handle, file, &rpc::Type::Named(rpc::NamedType::Enum(name)),
            access, rc, offset,
        )
    }
}

pub fn generate_decode_statement(
    handle: &handle::Handle,
    file: &mut dyn File,
    tp: &rpc::Type,
    access: &str,
    rc: Option<&str>,
    offset: Option<usize>,
) {
    let rc = rc.unwrap_or("rc");
    let offset = offset.unwrap_or(0);
    let soffset = (0..offset).map(|_| ' ').collect::<String>();

    match tp {
        rpc::Type::Void => {},
        rpc::Type::Integer(rpc::Integer::Integer)
        | rpc::Type::Unsigned(rpc::Integer::Integer) => IteratorPrinter::from([
            format!("{soffset}{{"),
            format!("{soffset}    int _rc = 0;"),
            format!("{soffset}    if (0 == {rc}"),
            format!("{soffset}        && 0 > (_rc = xdr_stream_decode_u32(xdr, &({access})))) {{"),
            format!("{soffset}        {rc} = _rc;"),
            format!("{soffset}    }}"),
            format!("{soffset}}}"),
        ]).print(file),
        rpc::Type::Integer(rpc::Integer::Hyper)
        | rpc::Type::Unsigned(rpc::Integer::Hyper) => IteratorPrinter::from([
            format!("{soffset}{{"),
            format!("{soffset}    int _rc = 0;"),
            format!("{soffset}    if (0 == {rc}"),
            format!("{soffset}        && 0 > (_rc = xdr_stream_decode_u64(xdr, &({access})))) {{"),
            format!("{soffset}        {rc} = _rc;"),
            format!("{soffset}    }}"),
            format!("{soffset}}}"),
        ]).print(file),
        rpc::Type::Float(float) => match float {
            rpc::Float::Single => IteratorPrinter::from([
                format!("{soffset}{{"),
                format!("{soffset}    int _rc = 0;"),
                format!("{soffset}    if (0 == {rc}"),
                format!("{soffset}        && 0 > (_rc = xdr_stream_decode_opaque(xdr, &({access}), sizeof(float)))) {{"),
                format!("{soffset}        {rc} = _rc;"),
                format!("{soffset}    }}"),
                format!("{soffset}}}"),
            ]).print(file),
            rpc::Float::Double => IteratorPrinter::from([
                format!("{soffset}{{"),
                format!("{soffset}    int _rc = 0;"),
                format!("{soffset}    if (0 == {rc}"),
                format!("{soffset}        && 0 > (_rc = xdr_stream_decode_opaque(xdr, &({access}), sizeof(double)))) {{"),
                format!("{soffset}        {rc} = _rc;"),
                format!("{soffset}    }}"),
                format!("{soffset}}}"),
            ]).print(file),
            rpc::Float::Quadruple => panic!("Not today"), // ToDo?
        },
        rpc::Type::Boolean => IteratorPrinter::from([
            format!("{soffset}{{"),
            format!("{soffset}    int _rc = 0;"),
            format!("{soffset}    if (0 == {rc}"),
            format!("{soffset}        && 0 > (_rc = xdr_stream_decode_bool(xdr, &({access})))) {{"),
            format!("{soffset}        {rc} = _rc;"),
            format!("{soffset}    }}"),
            format!("{soffset}}}"),
        ]).print(file),
        rpc::Type::Pointer(tp) => {
            let tname = asc::typename(&asc::fulltype(tp));
            IteratorPrinter::from([
                format!("{soffset}{{"),
                format!("{soffset}    int _rc = 0;"),
                format!("{soffset}    u32 size = 0;"),
                format!("{soffset}    if (0 == {rc}"),
                format!("{soffset}        && 0 > (_rc = xdr_stream_decode_u32(xdr, &size))) {{"),
                format!("{soffset}        {rc} = _rc;"),
                format!("{soffset}    }}"),
                format!("{soffset}    if (0 == {rc} && 1 < size) {{"),
                format!("{soffset}        {rc} = -EMSGSIZE;"),
                format!("{soffset}    }} else if (0 == {rc} && 1 == size) {{"),
                format!("{soffset}        {access} = kmalloc(sizeof({tname}), GFP_KERNEL);"),
                format!("{soffset}        if (NULL == {access}) {{"),
                format!("{soffset}            {rc} = -ENOMEM;"),
                format!("{soffset}        }} else {{"),
            ]).print(file);
            generate_decode_statement(handle, file, tp,
                &format!("*({access})"), Some(rc), Some(offset + 12)
            );
            IteratorPrinter::from([
                format!("{soffset}        }}"),
                format!("{soffset}    }} else {{"),
                format!("{soffset}        {access} = NULL;"),
                format!("{soffset}    }}"),
                format!("{soffset}}}"),
            ]).print(file);
        },
        rpc::Type::Array(tp, sz) => match tp.as_ref() { // ToDo: Not following xdr protocol, but will work
            rpc::Type::Opaque => IteratorPrinter::from([
                format!("{soffset}{{"),
                format!("{soffset}    int _rc = 0;"),
                format!("{soffset}    if (0 == {rc}"),
                format!("{soffset}        && 0 > (_rc = xdr_stream_decode_opaque(xdr, {access}, {}))) {{", asc::value(sz)),
                format!("{soffset}        {rc} = _rc;"),
                format!("{soffset}    }}"),
                format!("{soffset}}}"),
            ]).print(file),
            _ => {
                format!("{soffset}for (size_t i = 0; 0 == {rc} && {} > i; i++) {{", asc::value(sz)).print(file);
                generate_decode_statement(handle, file, tp,
                    &format!("({access})[i]"), Some(rc), Some(offset + 4)
                );
                format!("{soffset}}}").print(file);
            },
        },
        rpc::Type::VArray(tp, sz) => {
            let ctype = asc::fulltype(tp);
            let name = asc::typename(&ctype);
            let sz = sz.as_ref().map(asc::value)
                .unwrap_or_else(|| String::from("VLA_LIMIT"));
            IteratorPrinter::from([
                format!("{soffset}{{"),
                format!("{soffset}    int _rc = 0;"),
                format!("{soffset}    if (0 == {rc}"),
                format!("{soffset}        && 0 > (_rc = xdr_stream_decode_u32(xdr, &({access}).size))) {{"),
                format!("{soffset}        {rc} = _rc;"),
                format!("{soffset}    }}"),
                format!("{soffset}    if (0 == {rc} && {sz} < ({access}).size) {{"),
                format!("{soffset}        {rc} = -EMSGSIZE;"),
                format!("{soffset}    }} else if (0 == {rc} && 0 != ({access}).size) {{"),
                format!("{soffset}        ({access}).data = kmalloc(sizeof({name}) * ({access}).size, GFP_KERNEL);"),
                format!("{soffset}        if (NULL == ({access}).data) {{"),
                format!("{soffset}            {rc} = -ENOMEM;"),
                format!("{soffset}        }} else {{"),
            ]).print(file);

            match tp.as_ref() {
                rpc::Type::Opaque => IteratorPrinter::from([
                    format!("{soffset}            int _rc = 0;"),
                    format!("{soffset}            if (0 == {rc}"),
                    format!("{soffset}                && 0 > (_rc = xdr_stream_decode_opaque(xdr, ({access}).data, ({access}).size))) {{"),
                    format!("{soffset}                {rc} = _rc;"),
                    format!("{soffset}            }}"),
                ]).print(file),
                rpc::Type::String => IteratorPrinter::from([
                    format!("{soffset}            int _rc = 0;"),
                    format!("{soffset}            if (0 == {rc}"),
                    format!("{soffset}                && 0 > (_rc = xdr_stream_decode_string(xdr, ({access}).data, ({access}).size))) {{"),
                    format!("{soffset}                {rc} = _rc;"),
                    format!("{soffset}            }}"),
                ]).print(file),
                _ => {
                    IteratorPrinter::from([
                        format!("{soffset}            {} = ({})(({access}).data);",
                            asc::pointer_declaration("_base", &ctype),
                            asc::pointer_declaration("", &ctype),
                        ),
                        format!("{soffset}            {} = _base;",
                            asc::pointer_declaration("base", &ctype),
                        ),
                        format!("{soffset}            for (size_t i = 0; 0 == {rc} && ({access}).size > i; i++) {{"),
                    ]).print(file);
                    generate_decode_statement(handle, file, tp,
                        &format!("base[i]"), Some(rc), Some(offset + 16)
                    );
                    format!("{soffset}            }}").print(file)
                }
            }

            IteratorPrinter::from([
                format!("{soffset}        }}"),
                format!("{soffset}    }} else {{"),
                format!("{soffset}        ({access}).data = NULL;"),
                format!("{soffset}    }}"),
                format!("{soffset}}}"),
            ]).print(file);
        },
        rpc::Type::Named(named) => match named {
            rpc::NamedType::Typedef(name) =>
                generate_decode_statement(handle, file,
                    handle.module.types.typedefs.get(name).expect("Was added"),
                    access, Some(rc), Some(offset)
                ),
            rpc::NamedType::Enum(_) => IteratorPrinter::from([
                format!("{soffset}{{"),
                format!("{soffset}    int _rc = 0;"),
                format!("{soffset}    s32 num = 0;"),
                format!("{soffset}    if (0 == {rc}"),
                format!("{soffset}        && 0 > (_rc = xdr_stream_decode_u32(xdr, &num))) {{"),
                format!("{soffset}        {rc} = _rc;"),
                format!("{soffset}    }}"),
                format!("{soffset}    if (0 == {rc}) {{"),
                format!("{soffset}        {access} = num;"),
                format!("{soffset}    }}"),
                format!("{soffset}}}"),
            ]).print(file),
            rpc::NamedType::Struct(name) => {
                let st = handle.module.types.structs.get(name).expect("Was added");
                format!("{soffset}{{ // struct {name}").print(file);
                st.iter().for_each(|(field, tp)| generate_decode_statement(
                    handle, file, tp,
                    &format!("({access}).{field}"),
                    Some(rc),
                    Some(offset + 4),
                ));
                format!("{soffset}}}").print(file);
            }
            rpc::NamedType::Union(name) => {
                let un = handle.module.types.unions.get(name).expect("Was added");
                format!("{soffset}{{ // union {name}").print(file);
                generate_switch_decode_statement(handle, file, &un.switch_type,
                    &format!("({access}).{}", un.value),
                    Some(rc), Some(offset + 4),
                );
                format!("{soffset}    switch (({access}).{}) {{", un.value).print(file);
                un.arms.iter().for_each(|(v, (field, tp))| {
                    format!("{soffset}    case ({}):", asc::value(v)).print(file);
                    generate_decode_statement(handle, file, tp,
                        &format!("({access}).{name}_u.{field}"),
                        Some(rc), Some(offset + 8),
                    );
                    format!("{soffset}        break;").print(file);
                });
                if let Some((field, tp)) = &un.default {
                    format!("{soffset}    default:").print(file);
                    generate_decode_statement(handle, file, tp,
                        &format!("({access}).{name}_u.{field}"),
                        Some(rc), Some(offset + 8),
                    );
                    format!("{soffset}        break;").print(file);
                }
                format!("{soffset}    }}").print(file);
                format!("{soffset}}}").print(file);
            },
        },
        rpc::Type::String | rpc::Type::Opaque => panic!("Unexpected unit type: {tp:?}"),
    }
}

fn generate_switch_encode_statement(
    handle: &handle::Handle,
    file: &mut dyn File,
    tp: &rpc::SwitchingType,
    access: &str,
    rc: Option<&str>,
    offset: Option<usize>,
) {
    match tp.clone() {
        rpc::SwitchingType::Integer(integer) => generate_encode_statement(
            handle, file, &rpc::Type::Integer(integer), access, rc, offset,
        ),
        rpc::SwitchingType::Unsigned(integer) => generate_encode_statement(
            handle, file, &rpc::Type::Unsigned(integer), access, rc, offset,
        ),
        rpc::SwitchingType::Enum(name) => generate_encode_statement(
            handle, file, &rpc::Type::Named(rpc::NamedType::Enum(name)),
            access, rc, offset,
        )
    }
}

pub fn generate_encode_statement(
    handle: &handle::Handle,
    file: &mut dyn File,
    tp: &rpc::Type,
    access: &str,
    rc: Option<&str>,
    offset: Option<usize>,
) {
    let rc = rc.unwrap_or("rc");
    let offset = offset.unwrap_or(0);
    let soffset = (0..offset).map(|_| ' ').collect::<String>();

    match tp {
        rpc::Type::Void => {},
        rpc::Type::Integer(rpc::Integer::Integer)
        | rpc::Type::Unsigned(rpc::Integer::Integer) => IteratorPrinter::from([
            format!("{soffset}{{"),
            format!("{soffset}    int _rc = 0;"),
            format!("{soffset}    if (0 == {rc}"),
            format!("{soffset}        && 0 > (_rc = xdr_stream_encode_u32(xdr, {access}))) {{"),
            format!("{soffset}        {rc} = _rc;"),
            format!("{soffset}    }}"),
            format!("{soffset}}}"),
        ]).print(file),
        rpc::Type::Integer(rpc::Integer::Hyper)
        | rpc::Type::Unsigned(rpc::Integer::Hyper) => IteratorPrinter::from([
            format!("{soffset}{{"),
            format!("{soffset}    int _rc = 0;"),
            format!("{soffset}    if (0 == {rc}"),
            format!("{soffset}        && 0 > (_rc = xdr_stream_encode_u64(xdr, {access}))) {{"),
            format!("{soffset}        {rc} = _rc;"),
            format!("{soffset}    }}"),
            format!("{soffset}}}"),
        ]).print(file),
        rpc::Type::Float(float) => match float {
            rpc::Float::Single => IteratorPrinter::from([
                format!("{soffset}{{"),
                format!("{soffset}    int _rc = 0;"),
                format!("{soffset}    if (0 == {rc}"),
                format!("{soffset}        && 0 > (_rc = xdr_stream_encode_opaque(xdr, &({access}), sizeof(float)))) {{"),
                format!("{soffset}        {rc} = _rc;"),
                format!("{soffset}    }}"),
                format!("{soffset}}}"),
            ]).print(file),
            rpc::Float::Double => IteratorPrinter::from([
                format!("{soffset}{{"),
                format!("{soffset}    int _rc = 0;"),
                format!("{soffset}    if (0 == {rc}"),
                format!("{soffset}        && 0 > (_rc = xdr_stream_encode_opaque(xdr, &({access}), sizeof(double)))) {{"),
                format!("{soffset}        {rc} = _rc;"),
                format!("{soffset}    }}"),
                format!("{soffset}}}"),
            ]).print(file),
            rpc::Float::Quadruple => panic!("Not today"), // ToDo?
        },
        rpc::Type::Boolean => IteratorPrinter::from([
            format!("{soffset}{{"),
            format!("{soffset}    int _rc = 0;"),
            format!("{soffset}    if (0 == {rc}"),
            format!("{soffset}        && 0 > (_rc = xdr_stream_encode_bool(xdr, {access}))) {{"),
            format!("{soffset}        {rc} = _rc;"),
            format!("{soffset}    }}"),
            format!("{soffset}}}"),
        ]).print(file),
        rpc::Type::Pointer(tp) => {
            IteratorPrinter::from([
                format!("{soffset}{{"),
                format!("{soffset}    int _rc = 0;"),
                format!("{soffset}    u32 size = (NULL == {access}) ? 0 : 1;"),
                format!("{soffset}    if (0 == {rc}"),
                format!("{soffset}        && 0 > (_rc = xdr_stream_encode_u32(xdr, size))) {{"),
                format!("{soffset}        {rc} = _rc;"),
                format!("{soffset}    }}"),
                format!("{soffset}    if (0 == {rc} && 1 == size) {{"),
            ]).print(file);
            generate_encode_statement(handle, file, tp,
                &format!("*({access})"), Some(rc), Some(offset + 8)
            );
            IteratorPrinter::from([
                format!("{soffset}    }}"),
                format!("{soffset}}}"),
            ]).print(file);
        },
        rpc::Type::Array(tp, sz) => {
            IteratorPrinter::from([
                format!("{soffset}{{"),
                format!("{soffset}    if (0 == {rc} && NULL == {access}) {{"),
                format!("{soffset}        {rc} = -EINVAL;"),
                format!("{soffset}    }}"),
            ]).print(file);
            match tp.as_ref() { // ToDo: Not following xdr protocol, but will work
                rpc::Type::Opaque => IteratorPrinter::from([
                    format!("{soffset}    int _rc = 0;"),
                    format!("{soffset}    if (0 == {rc}"),
                    format!("{soffset}        && 0 > (_rc = xdr_stream_encode_opaque(xdr, {access}, {}))) {{", asc::value(sz)),
                    format!("{soffset}        {rc} = _rc;"),
                    format!("{soffset}    }}"),
                ]).print(file),
                _ => {
                    format!("{soffset}    for (size_t i = 0; 0 == {rc} && {} > i; i++) {{", asc::value(sz)).print(file);
                    generate_encode_statement(handle, file, tp,
                        &format!("({access})[i]"), Some(rc), Some(offset + 8)
                    );
                    format!("{soffset}    }}").print(file);
                },
            }
            format!("{soffset}}}").print(file);
        },
        rpc::Type::VArray(tp, sz) => {
            let sz = sz.as_ref().map(asc::value)
                .unwrap_or_else(|| String::from("VLA_LIMIT"));
            IteratorPrinter::from([
                format!("{soffset}{{"),
                format!("{soffset}    int _rc = 0;"),
                format!("{soffset}    if (0 == {rc}"),
                format!("{soffset}        && 0 != ({access}).size"),
                format!("{soffset}        && NULL == ({access}).data) {{"),
                format!("{soffset}        {rc} = -EINVAL;"),
                format!("{soffset}    }}"),
                format!("{soffset}    if (0 == {rc} && {sz} < ({access}).size) {{"),
                format!("{soffset}        {rc} = -EMSGSIZE;"),
                format!("{soffset}    }}"),
                format!("{soffset}    if (0 == {rc}"),
                format!("{soffset}        && 0 > (_rc = xdr_stream_encode_u32(xdr, ({access}).size))) {{"),
                format!("{soffset}        {rc} = _rc;"),
                format!("{soffset}    }}"),
                format!("{soffset}    if (0 == {rc} && 0 != ({access}).size) {{"),
            ]).print(file);

            match tp.as_ref() {
                rpc::Type::Opaque | rpc::Type::String => IteratorPrinter::from([
                    format!("{soffset}        int _rc = 0;"),
                    format!("{soffset}        if (0 == {rc}"),
                    format!("{soffset}            && 0 > (_rc = xdr_stream_encode_opaque(xdr, ({access}).data, ({access}).size))) {{"),
                    format!("{soffset}            {rc} = _rc;"),
                    format!("{soffset}        }}"),
                ]).print(file),
                _ => {
                    let ctype = asc::fulltype(tp);
                    IteratorPrinter::from([
                        format!("{soffset}        {} = ({})(({access}).data);",
                            asc::pointer_declaration("_base", &ctype),
                            asc::pointer_declaration("", &ctype),
                        ),
                        format!("{soffset}        {} = _base;",
                            asc::pointer_declaration("base", &ctype),
                        ),
                        format!("{soffset}        for (size_t i = 0; 0 == {rc} && ({access}).size > i; i++) {{"),
                    ]).print(file);
                    generate_encode_statement(handle, file, tp,
                        &format!("base[i]"), Some(rc), Some(offset + 12)
                    );
                    format!("{soffset}        }}").print(file)
                }
            }

            IteratorPrinter::from([
                format!("{soffset}    }}"),
                format!("{soffset}}}"),
            ]).print(file);
        },
        rpc::Type::Named(named) => match named {
            rpc::NamedType::Typedef(name) =>
                generate_encode_statement(handle, file,
                    handle.module.types.typedefs.get(name).expect("Was added"),
                    access, Some(rc), Some(offset)
                ),
            rpc::NamedType::Enum(_) => IteratorPrinter::from([
                format!("{soffset}{{"),
                format!("{soffset}    int _rc = 0;"),
                format!("{soffset}    s32 num = {access};"),
                format!("{soffset}    if (0 == {rc}"),
                format!("{soffset}        && 0 > (_rc = xdr_stream_decode_u32(xdr, num))) {{"),
                format!("{soffset}        {rc} = _rc;"),
                format!("{soffset}    }}"),
                format!("{soffset}}}"),
            ]).print(file),
            rpc::NamedType::Struct(name) => {
                let st = handle.module.types.structs.get(name).expect("Was added");
                format!("{soffset}{{ // struct {name}").print(file);
                st.iter().for_each(|(field, tp)| generate_encode_statement(
                    handle, file, tp,
                    &format!("({access}).{field}"),
                    Some(rc),
                    Some(offset + 4),
                ));
                format!("{soffset}}}").print(file);
            }
            rpc::NamedType::Union(name) => {
                let un = handle.module.types.unions.get(name).expect("Was added");
                format!("{soffset}{{ // union {name}").print(file);
                generate_switch_encode_statement(handle, file, &un.switch_type,
                    &format!("({access}).{}", un.value),
                    Some(rc), Some(offset + 4),
                );
                format!("{soffset}    switch (({access}).{}) {{", un.value).print(file);
                un.arms.iter().for_each(|(v, (field, tp))| {
                    format!("{soffset}    case ({}):", asc::value(v)).print(file);
                    generate_encode_statement(handle, file, tp,
                        &format!("({access}).{name}_u.{field}"),
                        Some(rc), Some(offset + 8),
                    );
                    format!("{soffset}        break;").print(file);
                });
                if let Some((field, tp)) = &un.default {
                    format!("{soffset}    default:").print(file);
                    generate_encode_statement(handle, file, tp,
                        &format!("({access}).{name}_u.{field}"),
                        Some(rc), Some(offset + 8),
                    );
                    format!("{soffset}        break;").print(file);
                }
                format!("{soffset}    }}").print(file);
                format!("{soffset}}}").print(file);
            },
        },
        rpc::Type::String | rpc::Type::Opaque => panic!("Unexpected unit type: {tp:?}"),
    }
}

pub fn generate_argument_wrap_struct(_: &handle::Handle, proc: &rpc::Procedure) -> (String, rpc::Struct) {
    (
        format!("{}_argument_wrap", proc.name),
        proc.arguments.iter()
            .enumerate()
            .map(|(i, tp)| (format!("arg{i}"), tp.clone()))
            .collect()
    )
}

pub fn generate_argument_wrap(handle: &handle::Handle, file: &mut dyn File, proc: &rpc::Procedure) -> bool {
    if 1 >= proc.arguments.len() {
        false
    } else {
        let st = generate_argument_wrap_struct(handle, proc);
        (&st.0, &st.1).print(file);
        true
    }

}

pub fn misc_types(file: &mut dyn File) {
    IteratorPrinter::from([
        "#define STATIC_MAX(a, b) (((a) > (b)) ? (a) : (b))",
        "",
        "struct _vla {",
        "    u32 size;   // Amount of elements (For more information see the specification)",
        "    void *data;",
        "};",
        "typedef struct _vla vla_t;",
        "typedef struct _vla string_t;",
        "#define vla(type) vla_t",
    ]).print(file);
}

pub struct Constants {
    vla_limit: usize
}

impl Constants {
    pub fn new(cfg: &Option<config::Config>) -> Self {
        Self {
            vla_limit: config::vla_limit(cfg),
        }
    }
}

pub fn misc_constants(file: &mut dyn File, cfg: Constants) {
    IteratorPrinter::from([
        format!("#define VLA_LIMIT {}", cfg.vla_limit),
    ]).print(file);
}

