#ifndef _SERVERS_COMMON_H_
#define _SERVERS_COMMON_H_

#include <linux/sunrpc/svc.h>

int threadfn(void *data);
int dispatch(struct svc_rqst *rqstp);

#endif
