#include <linux/module.h>
#include <linux/kernel.h>

#include <linux/sunrpc/svc.h>
#include <linux/sunrpc/svc_xprt.h>

#include "../common.h"
#include "constants.h"
#include "authentication.h"

#include "initial/version.h"

MODULE_LICENSE("GPL");

static unsigned short port = 0;
module_param(port, ushort, 0);

static unsigned int threads = 1;
module_param(threads, uint, 0);

static const struct svc_version *example_versions[] = {
    [initial] = &initial_version,
};

static struct svc_program example_program = {
    .pg_prog = example,
    .pg_lovers = 1,
    .pg_hivers = 1,
    .pg_nvers = ARRAY_SIZE(example_versions),
    .pg_vers = example_versions,
    .pg_name = "example",
    .pg_class = "example",
    .pg_authenticate = authenticate,
    .pg_init_request = svc_generic_init_request,
    .pg_rpcbind_set  = svc_generic_rpcbind_set
};

static struct svc_stat stat;
static struct svc_serv *server = NULL;

static int __init init_md(void) {
    stat.program = &example_program;
    server = svc_create(&example_program, 0, threadfn);

    int rc = 0;
    int cport = port;

    if (NULL == server) {
        rc = -EINVAL;
    }

    if (0 == rc) {
        rc = svc_bind(server, &init_net);
    }

    if (0 == rc) {
        rc = svc_xprt_create(server, "tcp", &init_net, AF_INET, cport, 0, get_current_cred());
        cport = rc > 0 ? rc : cport;
        rc = rc < 0 ? rc : 0;
    }

    if (0 == rc) {
        rc = svc_set_num_threads(server, NULL, threads);
    }

    if (0 == rc) {
        printk("[example] RPC server started at port: %d\n", cport);
    } else {
        printk("[example] RPC server setup error: %pe\n", ERR_PTR(rc));
    }

    return rc;
}

static void __exit exit_md(void) {
    if (server) {
        svc_xprt_destroy_all(server, &init_net);
        svc_set_num_threads(server, NULL, 0);
        svc_rpcb_cleanup(server, &init_net);
        svc_destroy(&server);
    }

    server = NULL;
    printk("[example] RPC server stopped\n");
}

module_init(init_md);
module_exit(exit_md);
