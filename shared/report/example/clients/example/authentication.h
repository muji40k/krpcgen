#ifndef _CLIENTS_EXAMPLE_AUTHENTICATION_H_
#define _CLIENTS_EXAMPLE_AUTHENTICATION_H_

#include <linux/sunrpc/clnt.h>
#include <linux/sunrpc/xdr.h>

#define AUTH_HANDLE_SIZE 0

// General handle, edit as you like
void auth_handle_encode(struct rpc_rqst *rqstp, struct xdr_stream *xdr, const void *handle);

#endif
