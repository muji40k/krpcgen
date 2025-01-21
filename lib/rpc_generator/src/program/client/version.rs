
use crate::{
    handle,
    file::{
        File,
        Printable,
        IteratorPrinter,
    },
};

pub fn generate_version_definition(_: &handle::Handle, file: &mut dyn File, ver: &rpc::Version) {
    format!("extern const struct rpc_version {}_version;", ver.name).print(file)
}

pub fn generate_version_declaraion(_: &handle::Handle, file: &mut dyn File, ver: &rpc::Version) {
    IteratorPrinter::from([
        format!("static unsigned int {}_call_count = 0;", ver.name),
        format!("const struct rpc_version {}_version = {{", ver.name),
        format!("    .number = {},", ver.name),
        format!("    .nrprocs = ARRAY_SIZE({}_procedures),", ver.name),
        format!("    .procs = {}_procedures,", ver.name),
        format!("    .counts = &{}_call_count,", ver.name),
        format!("}};")
    ]).print(file)
}

pub fn generate_version_procedures_array_definition(_: &handle::Handle, file: &mut dyn File, ver: &rpc::Version) {
    format!("extern const struct rpc_procinfo {}_procedures[];", ver.name).print(file);
}

pub fn generate_version_procedures_array_declaration(handle: &handle::Handle, file: &mut dyn File, ver: &rpc::Version) {
    format!("const struct rpc_procinfo {}_procedures[] = {{", ver.name).print(file);
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

