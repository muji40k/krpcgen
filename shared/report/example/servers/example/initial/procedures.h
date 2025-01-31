#ifndef _SERVERS_EXAMPLE_INITIAL_PROCEDURES_H_
#define _SERVERS_EXAMPLE_INITIAL_PROCEDURES_H_

#include <linux/sunrpc/svc.h>
#include <linux/sunrpc/xdr.h>

#include "../../../types.h"

__be32 initial_get_tasks_handler(struct svc_rqst *rqstp);
bool initial_get_tasks_decode(struct svc_rqst *rqstp, struct xdr_stream *xdr);
bool initial_get_tasks_encode(struct svc_rqst *rqstp, struct xdr_stream *xdr);
void initial_get_tasks_release(struct svc_rqst *rqstp);


#endif
