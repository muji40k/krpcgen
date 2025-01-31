#ifndef _CLIENTS_CLIENT_H_
#define _CLIENTS_CLIENT_H_

#include <linux/sunrpc/clnt.h>

int client_init(const struct rpc_program *program);
struct rpc_clnt *client_get(void);
void client_free(void);

#endif
