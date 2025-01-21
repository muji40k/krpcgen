
use crate::{
    handle,
    types,
    file::{
        File,
        Printable,
        IteratorPrinter,
    },
};

pub fn generate_version_definition(_: &handle::Handle, file: &mut dyn File, ver: &rpc::Version) {
    format!("extern const struct svc_version {}_version;", ver.name).print(file)
}

pub fn generate_version_declaraion(handle: &handle::Handle, file: &mut dyn File, ver: &rpc::Version) {
    let argsize = ver.procedures.values().map(|proc| {
        proc.arguments.iter().map(|tp| {
            types::generate_xdr_size(handle, tp)
        }).filter(|v| "0" != v)
            .reduce(|a, b| a + "+" + &b)
            .unwrap_or_else(|| String::from("0"))
    }).filter(|v| "0" != v)
        .reduce(|a, b| format!("STATIC_MAX(({a}), ({b}))"))
        .unwrap_or_else(|| String::from("0"));
    IteratorPrinter::from([
        format!("static unsigned long {}_call_count = 0;", ver.name),
        format!("const struct svc_version {}_version = {{", ver.name),
        format!("    .vs_vers = {},", ver.name),
        format!("    .vs_nproc = ARRAY_SIZE({}_procedures),", ver.name),
        format!("    .vs_proc = {}_procedures,", ver.name),
        format!("    .vs_count = &{}_call_count,", ver.name),
        format!("    .vs_dispatch = dispatch,"),
        format!("    .vs_xdrsize = AUTH_HANDLE_SIZE+{argsize},"),
        format!("    .vs_hidden = false,"),
        format!("    .vs_rpcb_optnl = false,"),
        format!("    .vs_need_cong_ctrl = false"),
        format!("}};")
    ]).print(file)
}

pub fn generate_version_procedures_array(handle: &handle::Handle, file: &mut dyn File, ver: &rpc::Version) {
    format!("static const struct svc_procedure {}_procedures[] = {{", ver.name).print(file);
    ver.procedures.values().for_each(|proc|
        super::procedure::generate_procedure_declaration(handle, file, proc, &ver.name)
    );
    "};".print(file);
}

pub fn generate_version_constants(_: &handle::Handle, file: &mut dyn File, ver: &rpc::Version) {
    IteratorPrinter::from(
        ver.procedures.iter().map(|(value, proc)| (&proc.name, value))
    ).print(file)
}

