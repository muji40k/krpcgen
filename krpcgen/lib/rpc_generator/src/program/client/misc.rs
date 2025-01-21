
use crate::{
    handle,
    file::{
        File,
        Printable,
        IteratorPrinter,
    }
};

pub fn generate_client_misc_definition(_: &handle::Handle, file: &mut dyn File) {
    IteratorPrinter::from([
        "int client_init(const struct rpc_program *program);",
        "struct rpc_clnt *client_get(void);",
        "void client_free(void);",
    ]).print(file)
}

pub fn generate_client_misc_declaration(_: &handle::Handle, file: &mut dyn File) {
    IteratorPrinter::from([
        "static unsigned int version = 1;",
        "module_param(version, uint , 0);",
        "",
        "static unsigned int host_ip_size = 4;",
        "static unsigned char host_ip[4] = {127, 0, 0 ,1};",
        "module_param_array(host_ip, byte, &host_ip_size, 0);",
        "",
        "static unsigned short port = 0;",
        "module_param(port, ushort, 0);",
        "",
        "static __be32 decode_host_ip(void) {",
        "    __u8 decode_buffer[4];",
        "",
        "    for (size_t i = 0; host_ip_size > i; i++) {",
        "        decode_buffer[i] = host_ip[i];",
        "    }",
        "",
        "    for (size_t i = host_ip_size; 4 > i; i++) {",
        "        decode_buffer[i] = 0;",
        "    }",
        "",
        "    return *(__be32*)decode_buffer;",
        "}",
        "",
        "static struct rpc_clnt *client = NULL;",
        "",
        "int client_init(const struct rpc_program *program) {",
        "    if (NULL != client) {",
        "        return 0;",
        "    }",
        "",
        "    struct sockaddr_in sin = {",
        "        .sin_family      = AF_INET,",
        "        .sin_addr.s_addr = decode_host_ip(),",
        "        .sin_addr.s_port = htons(port),",
        "    };",
        "",
        "    struct rpc_create_args args = {",
        "        .net = &init_net,",
        "        .protocol = XPRT_TRANSPORT_TCP,",
        "        .address = (struct sockaddr *)&sin,",
        "        .addrsize = sizeof(sin),",
        "        .program = program,",
        "        .version = version,",
        "        .authflavor = RPC_AUTH_NULL,",
        "        .cred = current_cred(),",
        "        .flags = RPC_CLNT_CREATE_NOPING | RPC_CLNT_CREATE_REUSEPORT,",
        "    };",
        "",
        "    struct rpc_clnt *inner = rpc_create(&args);",
        "    int rc = 0;",
        "",
        "    if (IS_ERR(inner)) {",
        "        rc = PTR_ERR(inner);",
        "    } else {",
        "        client = inner;",
        "    }",
        "",
        "    return rc;",
        "}",
        "",
        "struct rpc_clnt *client_get(void) {",
        "    return client;",
        "}",
        "",
        "void client_free(void) {",
        "    if (client) {",
        "        rpc_shutdown_client(client);",
        "    }",
        "",
        "    client = NULL;",
        "}",
    ]).print(file)
}

