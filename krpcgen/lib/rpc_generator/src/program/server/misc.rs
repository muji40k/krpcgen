
use crate::{
    handle,
    file::{
        File,
        Printable,
        IteratorPrinter,
    },
};

pub fn generate_thread_function_definition(_: &handle::Handle, file: &mut dyn File) {
    "int threadfn(void *data);".print(file)
}

pub fn generate_thread_function_declaration(_: &handle::Handle, file: &mut dyn File) {
    IteratorPrinter::from([
        "int threadfn(void *data) {",
        "    struct svc_rqst *rqstp = data;",
        "    svc_thread_init_status(rqstp, 0);",
        "    set_freezable();",
        "",
        "    while (!svc_thread_should_stop(rqstp)) {",
        "        svc_recv(rqstp);",
        "    }",
        "",
        "    svc_exit_thread(rqstp);",
        "",
        "    return 0;",
        "}",
    ]).print(file)
}

pub fn generate_dispatch_definition(_: &handle::Handle, file: &mut dyn File) {
    "int dispatch(struct svc_rqst *rqstp);".print(file)
}

pub fn generate_dispatch_declaration(_: &handle::Handle, file: &mut dyn File) {
    IteratorPrinter::from([
        "int dispatch(struct svc_rqst *rqstp) {",
        "    const struct svc_procedure *proc = rqstp->rq_procinfo;",
        "    __be32 *statp = rqstp->rq_accept_statp;",
        "",
        "    if (!proc->pc_decode(rqstp, &rqstp->rq_arg_stream)) {",
        "        *statp = rpc_garbage_args;",
        "        return 0;",
        "    }",
        "",
        "    *statp = proc->pc_func(rqstp);",
        "",
        "    if (test_bit(RQ_DROPME, &rqstp->rq_flags)) {",
        "        *statp = rpc_success;",
        "        return 0;",
        "    }",
        "",
        "    if (!proc->pc_encode(rqstp, &rqstp->rq_res_stream)) {",
        "        *statp = rpc_system_err;",
        "        return 0;",
        "    }",
        "",
        "    return 1;",
        "}",
    ]).print(file)
}

