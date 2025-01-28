
pub mod version;
pub mod procedure;
pub mod misc;

use crate::{
    handle,
    file::{
        File,
        Printable,
        IteratorPrinter,
    }
};

pub fn generate_program_auth_handle_size(_: &handle::Handle, file: &mut dyn File, _: &rpc::Program) {
    "#define AUTH_HANDLE_SIZE 0".print(file);
}

pub fn generate_program_auth_encode_definition(_: &handle::Handle, file: &mut dyn File, _: &rpc::Program) {
    IteratorPrinter::from([
        "// General handle, edit as you like",
        "void auth_handle_encode(struct rpc_rqst *rqstp, struct xdr_stream *xdr, const void *handle);",
    ]).print(file)
}

pub fn generate_program_auth_encode_declaration(_: &handle::Handle, file: &mut dyn File, _: &rpc::Program) {
    IteratorPrinter::from([
        "void auth_handle_encode(struct rpc_rqst *rqstp, struct xdr_stream *xdr, const void *handle) {",
        "}",
    ]).print(file)
}

pub fn generate_program_declaraion(_: &handle::Handle, file: &mut dyn File, progr: &rpc::Program) {
    IteratorPrinter::from([
        format!("static struct rpc_stat stats = {{}};"),
        format!("static const struct rpc_program {}_program = {{", progr.name),
        format!("    .name = \"{}\",", progr.name),
        format!("    .number = {},", progr.name),
        format!("    .nrvers = ARRAY_SIZE({}_versions),", progr.name),
        format!("    .version = {}_versions,", progr.name),
        format!("    .stats = &stats,"),
        format!("}};"),
    ]).print(file)
}

pub fn generate_program_entrypoint(_: &handle::Handle, file: &mut dyn File, progr: &rpc::Program) {
    IteratorPrinter::from([
        format!("static int __init init_md(void) {{"),
        format!("    int rc = client_init(&{}_program);", progr.name),
        format!(""),
        format!("    if (0 == rc) {{"),
        format!("        printk(\"[{}] Client side api loaded\\n\");", progr.name),
        format!("    }} else {{"),
        format!("        printk(\"[{}] Client initialization error: %pe\\n\", ERR_PTR(rc));", progr.name),
        format!("    }}"),
        format!(""),
        format!("    return rc;"),
        format!("}}"),
        format!(""),
        format!("static void __exit exit_md(void) {{"),
        format!("    client_free();"),
        format!("    printk(\"[{}] RPC client side unloaded\\n\");", progr.name),
        format!("}}"),
        format!(""),
        format!("module_init(init_md);"),
        format!("module_exit(exit_md);"),
    ]).print(file)
}

pub fn generate_program_version_array(_: &handle::Handle, file: &mut dyn File, program: &rpc::Program) {
    format!("static const struct rpc_version *{}_versions[] = {{", program.name)
        .chain(IteratorPrinter::from(program.versions.values().map(|ver|
            format!("    [{0}] = &{0}_version,", ver.name)
        ))).chain(
            "};"
        ).print(file);
}

pub fn generate_program_constants(_: &handle::Handle, file: &mut dyn File, program: &rpc::Program) {
    IteratorPrinter::from(
        program.versions.iter().map(|(value, version)| (&version.name, value))
    ).print(file)
}

