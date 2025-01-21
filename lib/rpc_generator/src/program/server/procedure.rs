
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
    IteratorPrinter::from([
        format!("    [{}] = {{", proc.name),
        format!("        .pc_func = {ver}_{}_handler,", proc.name),
        format!("        .pc_decode = {ver}_{}_decode,", proc.name),
        format!("        .pc_encode = {ver}_{}_encode,", proc.name),
    ]).chain(match proc.arguments.len() {
        0 => IteratorPrinter::from([
            format!("        .pc_argsize = 0,"),
            format!("        .pc_argzero = 0,"),
        ]),
        1 => {
            let tname = types::asc::typename(&types::asc::fulltype(
                proc.arguments.get(0).expect("Was checked")
            ));
            IteratorPrinter::from([
                format!("        .pc_argsize = sizeof({tname}),"),
                format!("        .pc_argzero = sizeof({tname}),"),
            ])
        }
        _ => IteratorPrinter::from([
            format!("        .pc_argsize = sizeof(struct {}_argument_wrap),", proc.name),
            format!("        .pc_argzero = sizeof(struct {}_argument_wrap),", proc.name),
        ]),
    }).chain(procedure_need_release(handle, proc).then(||
        format!("        .pc_release = {ver}_{}_release,", proc.name)
    )).chain(IteratorPrinter::from([
        format!("        .pc_ressize = sizeof({}),",
            types::asc::typename(&types::asc::fulltype(&proc.return_type))
        ),
        format!("        .pc_xdrressize = {},", types::generate_xdr_size(handle, &proc.return_type)),
        format!("        .pc_name = \"{}\",", proc.name),
        format!("    }},"),
    ])).print(file)
}

pub fn generate_procedure_handler_definition(_: &handle::Handle, file: &mut dyn File, proc: &rpc::Procedure, ver: &str) {
    format!("__be32 {ver}_{}_handler(struct svc_rqst *rqstp);", proc.name).print(file)
}

pub fn generate_procedure_handler_declaration(handle: &handle::Handle, file: &mut dyn File, proc: &rpc::Procedure, ver: &str) {
    format!("__be32 {ver}_{}_handler(struct svc_rqst *rqstp) {{", proc.name)
        .chain(match proc.return_type {
            rpc::Type::Void => None,
            _ => Some({
                let ctype = types::asc::fulltype(&proc.return_type);
                format!("    {} = rqstp->rq_resp;",
                    types::asc::pointer_declaration("res", &ctype),
                )
            }),
        }).chain(match proc.arguments.len() {
            0 => None,
            1 => Some({
                let arg = proc.arguments.get(0).expect("Was checked");
                let ctype = types::asc::fulltype(arg);
                format!("    {} = rqstp->rq_argp;",
                    types::asc::pointer_declaration("arg", &ctype),
                )
            }),
            _ => Some({
                let wrap = types::generate_argument_wrap_struct(handle, proc);
                format!("    struct {} *arg = rqstp->rq_argp;", wrap.0)
            }),
        }).chain(IteratorPrinter::from([
            "    //",
            "    // Place for your logic",
            "    //",
            "    return rpc_success;",
            "}",
        ])).print(file)
}

pub fn generate_procedure_result_encode_definition(_: &handle::Handle, file: &mut dyn File, proc: &rpc::Procedure, ver: &str) {
    format!("bool {ver}_{}_encode(struct svc_rqst *rqstp, struct xdr_stream *xdr);", proc.name).print(file)
}

pub fn generate_procedure_result_encode_declaration(handle: &handle::Handle, file: &mut dyn File, proc: &rpc::Procedure, ver: &str) {
    format!("bool {ver}_{}_encode(struct svc_rqst *rqstp, struct xdr_stream *xdr) {{", proc.name)
        .switch(|file| match proc.return_type {
        rpc::Type::Void => format!("    return true;").print(file),
        _ => {
            let ctype = types::asc::fulltype(&proc.return_type);
            IteratorPrinter::from([
                format!("    {} = rqstp->rq_resp;",
                    types::asc::pointer_declaration("res", &ctype)
                ),
                format!("    int rc = 0;"),
                format!(""),
            ]).print(file);
            types::generate_encode_statement(handle, file, &proc.return_type, "*res", Some("rc"), Some(4));
            IteratorPrinter::from([
                "",
                "    if (0 != rc) {",
                "        return false;",
                "    }",
                "",
                "    return true;",
            ]).print(file);
        }
    }).chain(
        "}"
    ).print(file)
}

pub fn generate_procedure_arguments_decode_definition(_: &handle::Handle, file: &mut dyn File, proc: &rpc::Procedure, ver: &str) {
    format!("bool {ver}_{}_decode(struct svc_rqst *rqstp, struct xdr_stream *xdr);", proc.name).print(file)
}

pub fn generate_procedure_arguments_decode_declaration(handle: &handle::Handle, file: &mut dyn File, proc: &rpc::Procedure, ver: &str) {
    format!("bool {ver}_{}_decode(struct svc_rqst *rqstp, struct xdr_stream *xdr) {{", proc.name).print(file);
    match proc.arguments.len() {
        0 => format!("    return true;").print(file),
        1 => match proc.arguments.get(0).expect("Was checked") {
            rpc::Type::Void => format!("    return true;").print(file),
            _ => {
                let arg = proc.arguments.get(0).expect("Was checked");
                let ctype = types::asc::fulltype(arg);
                IteratorPrinter::from([
                    format!("    {} = rqstp->rq_argp;",
                        types::asc::pointer_declaration("arg", &ctype)
                    ),
                    format!("    int rc = 0;"),
                    format!(""),
                ]).print(file);
                types::generate_decode_statement(handle, file, arg, "*arg", Some("rc"), Some(4));
                IteratorPrinter::from([
                    "",
                    "    if (0 != rc) {",
                    "        return false;",
                    "    }",
                    "",
                    "    return true;",
                ]).print(file);
            }
        },
        _ => {
            let wrap = types::generate_argument_wrap_struct(handle, proc);
            IteratorPrinter::from([
                format!("    struct {} *arg = rqstp->rq_argp;", wrap.0),
                format!("    int rc = 0;"),
                format!(""),
            ]).print(file);
            wrap.1.iter().for_each(|(field, tp)| {
                types::generate_decode_statement(handle, file, tp,
                    &format!("arg->{field}"), Some("rc"), Some(4)
                );
            });
            IteratorPrinter::from([
                "",
                "    if (0 != rc) {",
                "        return false;",
                "    }",
                "",
                "    return true;",
            ]).print(file);
        }
    }
    "}".print(file)
}

pub fn procedure_need_release(handle: &handle::Handle, proc: &rpc::Procedure) -> bool {
    return types::uses_dynamic_memory(handle, &proc.return_type)
        || proc.arguments.iter().any(|tp| types::uses_dynamic_memory(handle, tp))
}

pub fn generate_procedure_release_definition(_: &handle::Handle, file: &mut dyn File, proc: &rpc::Procedure, ver: &str) {
    format!("void {ver}_{}_release(struct svc_rqst *rqstp);", proc.name).print(file)
}

pub fn generate_procedure_release_declaration(handle: &handle::Handle, file: &mut dyn File, proc: &rpc::Procedure, ver: &str) {
    format!("void {ver}_{}_release(struct svc_rqst *rqstp) {{", proc.name).print(file);

    if types::uses_dynamic_memory(handle, &proc.return_type) {
        let ctype = types::asc::fulltype(&proc.return_type);
        IteratorPrinter::from([
            format!("    {} = rqstp->rq_resp;",
                types::asc::pointer_declaration("res", &ctype),
            ),
            format!(""),
        ]).print(file);
        types::generate_release_statement(handle, file, &proc.return_type, "*res", Some(4));
        "".print(file);
    }

    match proc.arguments.len() {
        0 => {},
        1 => {
            let arg = proc.arguments.get(0).expect("Was checked");

            if types::uses_dynamic_memory(handle, arg) {
                let ctype = types::asc::fulltype(arg);
                IteratorPrinter::from([
                    format!("    {} = rqstp->rq_argp;",
                        types::asc::pointer_declaration("arg", &ctype),
                    ),
                    format!(""),
                ]).print(file);
                types::generate_release_statement(handle, file, arg, "*arg", Some(4));
            }
        },
        _ => if proc.arguments.iter().any(|tp| types::uses_dynamic_memory(handle, tp)) {
            let wrap = types::generate_argument_wrap_struct(handle, proc);
            IteratorPrinter::from([
                format!("    struct {} *arg = rqstp->rq_argp;", wrap.0),
                format!(""),
            ]).print(file);
            wrap.1.iter().for_each(|(field, tp)| {
                types::generate_release_statement(handle, file, tp,
                    &format!("arg->{field}"), Some(4)
                );
            });
        },
    }

    "}".print(file)
}

