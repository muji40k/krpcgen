#include <linux/module.h>
#include <linux/kernel.h>

#include <linux/sunrpc/clnt.h>

#include "../client.h"
#include "constants.h"

#include "initial/version.h"

MODULE_LICENSE("GPL");

static const struct rpc_version *example_versions[] = {
    [initial] = &initial_version,
};

static struct rpc_stat stats = {};
static const struct rpc_program example_program = {
    .name = "example",
    .number = example,
    .nrvers = ARRAY_SIZE(example_versions),
    .version = example_versions,
    .stats = &stats,
};

static int __init init_md(void) {
    int rc = client_init(&example_program);

    if (0 == rc) {
        printk("[example] Client side api loaded\n");
    } else {
        printk("[example] Client initialization error: %pe\n", ERR_PTR(rc));
    }

    return rc;
}

static void __exit exit_md(void) {
    client_free();
    printk("[example] RPC client side unloaded\n");
}

module_init(init_md);
module_exit(exit_md);
