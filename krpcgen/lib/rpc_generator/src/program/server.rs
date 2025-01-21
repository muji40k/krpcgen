
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

pub fn generate_program_auth_definition(_: &handle::Handle, file: &mut dyn File, _: &rpc::Program) {
    "enum svc_auth_status authenticate(struct svc_rqst *rqstp);".print(file)
}

pub fn generate_program_auth_declaraion(_: &handle::Handle, file: &mut dyn File, _: &rpc::Program) {
    IteratorPrinter::from([
        format!("enum svc_auth_status authenticate(struct svc_rqst *rqstp) {{"),
        format!("    //"),
        format!("    // Your authentication logic goes here"),
        format!("    //"),
        format!("    return SVC_OK;"),
        format!("}}"),
    ]).print(file)
}

pub fn generate_program_declaraion(handle: &handle::Handle, file: &mut dyn File, progr: &rpc::Program) {
    let (min_v, max_v) = progr.versions.keys()
        .map(|v| crate::misc::unwrap_value(handle, v))
        .fold((-1, -1), |(mut min, mut max), c| {
            if -1 == min || min > c {
                min = c
            }
            if -1 == max || max < c {
                max = c
            }
            (min, max)
        });
    IteratorPrinter::from([
        format!("static struct svc_program {}_program = {{", progr.name),
        format!("    .pg_prog = {},", progr.name),
        format!("    .pg_lovers = {min_v},"),
        format!("    .pg_hivers = {max_v},"),
        format!("    .pg_nvers = ARRAY_SIZE({}_versions),", progr.name),
        format!("    .pg_vers = {}_versions,", progr.name),
        format!("    .pg_name = \"{}\",", progr.name),
        format!("    .pg_class = \"{}\",", progr.name),
        format!("    .pg_authenticate = authenticate,"),
        format!("    .pg_init_request = svc_generic_init_request,"),
        format!("    .pg_rpcbind_set  = svc_generic_rpcbind_set"),
        format!("}};"),
    ]).print(file)
}

pub fn generate_program_entrypoint(_: &handle::Handle, file: &mut dyn File, progr: &rpc::Program) {
    IteratorPrinter::from([
        format!("static struct svc_stat stat;"),
        format!("static struct svc_serv *server = NULL;"),
        format!(""),
        format!("static int __init init_md(void) {{"),
        format!("    stat.program = &{}_program;", progr.name),
        format!("    server = svc_create(&{}_program, 0, threadfn);", progr.name),
        format!(""),
        format!("    int rc = 0;"),
        format!("    int cport = port;"),
        format!(""),
        format!("    if (NULL == server) {{"),
        format!("        rc = -EINVAL;"),
        format!("    }}"),
        format!(""),
        format!("    if (0 == rc) {{"),
        format!("        rc = svc_bind(server, &init_net);"),
        format!("    }}"),
        format!(""),
        format!("    if (0 == rc) {{"),
        format!("        rc = svc_xprt_create(server, \"tcp\", &init_net, AF_INET, cport, 0, get_current_cred());"),
        format!("        cport = rc > 0 ? rc : cport;"),
        format!("        rc = rc < 0 ? rc : 0;"),
        format!("    }}"),
        format!(""),
        format!("    if (0 == rc) {{"),
        format!("        rc = svc_set_num_threads(server, NULL, threads);"),
        format!("    }}"),
        format!(""),
        format!("    if (0 == rc) {{"),
        format!("        printk(\"[{}] RPC server started at port: %d\\n\", cport);", progr.name),
        format!("    }} else {{"),
        format!("        printk(\"[{}] RPC server setup error: %pe\\n\", ERR_PTR(rc));", progr.name),
        format!("    }}"),
        format!(""),
        format!("    return rc;"),
        format!("}}"),
        format!(""),
        format!("static void __exit exit_md(void) {{"),
        format!("    if (server) {{"),
        format!("        svc_xprt_destroy_all(server, &init_net);"),
        format!("        svc_set_num_threads(server, NULL, 0);"),
        format!("        svc_rpcb_cleanup(server, &init_net);"),
        format!("        svc_destroy(&server);"),
        format!("    }}"),
        format!(""),
        format!("    server = NULL;"),
        format!("    printk(\"[{}] RPC server stopped\\n\");", progr.name),
        format!("}}"),
        format!(""),
        format!("module_init(init_md);"),
        format!("module_exit(exit_md);"),
    ]).print(file)
}

pub fn generate_program_version_array(_: &handle::Handle, file: &mut dyn File, program: &rpc::Program) {
    format!("static const struct svc_version *{}_versions[] = {{", program.name)
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
