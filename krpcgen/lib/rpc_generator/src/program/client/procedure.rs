
use crate::{
    handle,
    types,
    file::{
        File,
        Printable,
        IteratorPrinter,
    },
};

pub fn generate_procedure_declaration(handle: &handle::Handle, file: &mut dyn File, proc: &rpc::Procedure, ver: &str) {
    let ressize = types::generate_xdr_size(handle, &proc.return_type);
    let argsize = proc.arguments.iter()
        .map(|tp| types::generate_xdr_size(handle, tp))
        .filter(|v| "0" != v)
        .reduce(|a, b| a + "+" + &b)
        .unwrap_or_else(|| String::from("0"));

    IteratorPrinter::from([
        format!("    [{}] = {{", proc.name),
        format!("        .p_proc = {},", proc.name),
        format!("        .p_encode = {ver}_{}_encode,", proc.name),
        format!("        .p_decode = {ver}_{}_decode,", proc.name),
        format!("        .p_arglen = AUTH_HANDLE_SIZE+{argsize},"),
        format!("        .p_replen = {ressize},"),
        format!("        .p_statidx = {},", proc.name),
        format!("        .p_name = \"{}\",", proc.name),
        format!("    }},"),
    ]).print(file)
}

fn expand_arguments(proc: &rpc::Procedure) -> String {
    proc.arguments.iter().enumerate().map(|(i, tp)|
        types::asc::declaration(
            &format!("arg{i}"),
            &types::asc::fulltype(tp)
        )
    ).reduce(|a, b| a + ", " + &b)
        .unwrap_or_else(|| String::from("void"))
}

pub fn generate_procedure_api_definition(_: &handle::Handle, file: &mut dyn File, proc: &rpc::Procedure, ver: &str, prog: &str) {
    "typedef struct {".chain(match &proc.return_type {
        rpc::Type::Void => None,
        _ => Some(format!("    {};",
            types::asc::declaration("value",
                &types::asc::fulltype(&proc.return_type)
            )
        )),
    }).chain(IteratorPrinter::from([
        format!("    int error;"),
        format!("}} {}_result_t;", proc.name),
        format!(""),
        format!("{0}_result_t {prog}_{ver}_{0}({1});", proc.name, expand_arguments(proc)),
    ])).print(file)
}

pub fn generate_procedure_api_declaration(handle: &handle::Handle, file: &mut dyn File, proc: &rpc::Procedure, ver: &str, prog: &str) {
    let wrapped = (1 < proc.arguments.len()).then(|| types::generate_argument_wrap_struct(handle, proc));
    IteratorPrinter::from([
        format!("{0}_result_t {prog}_{ver}_{0}({1}) {{", proc.name, expand_arguments(proc)),
        format!("    {}_result_t res;", proc.name),
        format!("    struct rpc_clnt *client = client_get();"),
        format!(""),
        format!("    if (IS_ERR(client)) {{"),
        format!("        res.error = PTR_ERR(client);"),
        format!("    }} else if (NULL == client) {{"),
        format!("        res.error = -EINVAL;"),
        format!("    }} else {{"),
    ]).chain(wrapped.map(|(name, st)| {
        format!("        struct {name} arg = {{").chain(IteratorPrinter::from(
            st.into_keys().map(|field| format!("            .{field} = {field},"))
        )).chain(IteratorPrinter::from([
            "        };",
            "",
        ]))
    })).chain(IteratorPrinter::from([
        format!("        struct rpc_message msg = {{"),
        format!("            .rpc_proc = &{}_procedures[{}],", ver, proc.name),
    ])).chain(match proc.arguments.len() {
        0 => None,
        1 => Some("            .rpc_argp = &arg0,"),
        _ => Some("            .rpc_argp = &arg,"),
    }).chain(match &proc.return_type {
        rpc::Type::Void => None,
        _ => Some("            .rpc_resp = &res.value,")
    }).chain(IteratorPrinter::from([
        format!("            .rpc_cred = get_current_cred(),"),
        format!("        }};"),
        format!(""),
        format!("        res.error = rpc_call_sync(client, &msg, 0);"),
    ])).chain(IteratorPrinter::from([
        format!("    }}"),
        format!(""),
        format!("    return res;"),
        format!("}}"),
        format!("EXPORT_SYMBOL({prog}_{ver}_{});", proc.name),
    ])).print(file);
}

pub fn generate_procedure_result_decode_definition(_: &handle::Handle, file: &mut dyn File, proc: &rpc::Procedure, ver: &str) {
    format!("int {ver}_{}_decode(struct rpc_rqst *rqstp, struct xdr_stream *xdr, void *data);", proc.name).print(file)
}

pub fn generate_procedure_result_decode_declaration(handle: &handle::Handle, file: &mut dyn File, proc: &rpc::Procedure, ver: &str) {
    format!("int {ver}_{}_decode(struct rpc_rqst *rqstp, struct xdr_stream *xdr, void *data) {{", proc.name)
        .switch(|file| match proc.return_type {
            rpc::Type::Void => format!("    return 0;").print(file),
            _ => {
                let ctype = types::asc::fulltype(&proc.return_type);
                IteratorPrinter::from([
                    format!("    {} = data;",
                        types::asc::pointer_declaration("res", &ctype),
                    ),
                    format!("    int rc = 0;"),
                    format!(""),
                ]).switch(|file|
                    types::generate_decode_statement(handle, file,
                        &proc.return_type, "*res", Some("rc"), Some(4)
                    )
                ).chain(IteratorPrinter::from([
                    "",
                    "    return rc;",
                ])).print(file);
            }
        }).chain(
            "}"
        ).print(file)
}

pub fn generate_procedure_arguments_encode_definition(_: &handle::Handle, file: &mut dyn File, proc: &rpc::Procedure, ver: &str) {
    format!("void {ver}_{}_encode(struct rpc_rqst *rqstp, struct xdr_stream *xdr, const void *data);", proc.name).print(file)
}

pub fn generate_procedure_arguments_encode_declaration(handle: &handle::Handle, file: &mut dyn File, proc: &rpc::Procedure, ver: &str, prog: &str) {
    IteratorPrinter::from([
        format!("void {ver}_{}_encode(struct rpc_rqst *rqstp, struct xdr_stream *xdr, const void *data) {{", proc.name),
        format!("    auth_handle_encode(rqstp, xdr, NULL);")
    ]).switch(|file| match proc.arguments.len() {
        0 => {},
        1 => match proc.arguments.get(0).expect("Was checked") {
            rpc::Type::Void => {},
            _ => {
                let arg = proc.arguments.get(0).expect("Was checked");
                let ctype = types::asc::fulltype(arg);
                IteratorPrinter::from([
                    format!("    const {} = data;",
                        types::asc::pointer_declaration("arg", &ctype),
                    ),
                    format!("    int rc = 0;"),
                    format!(""),
                ]).switch(|file|
                    types::generate_encode_statement(handle, file, arg,
                        "*arg", Some("rc"), Some(4)
                    )
                ).chain(IteratorPrinter::from([
                    format!(""),
                    format!("    if (0 != rc) {{"),
                    format!("        printk(\"[{prog}_{ver}_{}] Error during decode: %pe\", ERR_PTR(rc));", proc.name),
                    format!("    }}"),
                ])).print(file);
            }
        },
        _ => {
            let wrap = types::generate_argument_wrap_struct(handle, proc);
            IteratorPrinter::from([
                format!("    const struct {} *arg = data;", wrap.0),
                format!("    int rc = 0;"),
                format!(""),
            ]).switch(|file| wrap.1.into_iter().for_each(|(field, tp)|
                types::generate_encode_statement(handle, file, &tp,
                    &format!("arg->{field}"), Some("rc"), Some(4)
                )
            )).chain(IteratorPrinter::from([
                format!(""),
                format!("    if (0 != rc) {{"),
                format!("        printk(\"[{prog}_{ver}_{}] Error during decode: %pe\", ERR_PTR(rc));", proc.name),
                format!("    }}"),
            ])).print(file);
        }
    }).chain(
        "}"
    ).print(file)
}

pub fn generate_procedure_release_definition(handle: &handle::Handle, file: &mut dyn File, proc: &rpc::Procedure, ver: &str, prog: &str) -> bool {
    if !types::uses_dynamic_memory(handle, &proc.return_type) {
        return false;
    }

    format!("void release_result_{prog}_{ver}_{}({});", proc.name,
        types::asc::pointer_declaration("result",
            &types::asc::fulltype(&proc.return_type)
        ),
    ).print(file);

    true
}

pub fn generate_procedure_release_declaration(handle: &handle::Handle, file: &mut dyn File, proc: &rpc::Procedure, ver: &str, prog: &str) -> bool {
    if !types::uses_dynamic_memory(handle, &proc.return_type) {
        return false;
    }

    IteratorPrinter::from([
        format!("void release_result_{prog}_{ver}_{}({}) {{", proc.name,
            types::asc::pointer_declaration("result",
                &types::asc::fulltype(&proc.return_type),
            ),
        ),
        format!("    if (NULL == result) {{"),
        format!("        return;"),
        format!("    }}"),
        format!(""),
    ]).switch(|file|
        types::generate_release_statement(handle, file, &proc.return_type,
            "*result", Some(4)
        )
    ).chain(IteratorPrinter::from([
        format!("}}"),
        format!("EXPORT_SYMBOL(release_result_{prog}_{ver}_{});", proc.name),
    ])).print(file);

    true
}

