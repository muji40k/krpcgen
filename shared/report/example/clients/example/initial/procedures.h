#ifndef _CLIENTS_EXAMPLE_INITIAL_PROCEDURES_H_
#define _CLIENTS_EXAMPLE_INITIAL_PROCEDURES_H_

#include <linux/sunrpc/clnt.h>
#include <linux/sunrpc/xdr.h>

void initial_get_tasks_encode(struct rpc_rqst *rqstp, struct xdr_stream *xdr, const void *data);
int initial_get_tasks_decode(struct rpc_rqst *rqstp, struct xdr_stream *xdr, void *data);


#endif
