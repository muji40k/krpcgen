#ifndef _SERVERS_EXAMPLE_AUTHENTICATION_H_
#define _SERVERS_EXAMPLE_AUTHENTICATION_H_

#include <linux/sunrpc/svcauth.h>

#define AUTH_HANDLE_SIZE 0

enum svc_auth_status authenticate(struct svc_rqst *rqstp);

#endif
