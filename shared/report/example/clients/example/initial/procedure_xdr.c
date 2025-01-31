#include "procedures.h"
#include "../authentication.h"
#include "../../../types.h"

void initial_get_tasks_encode(struct rpc_rqst *rqstp, struct xdr_stream *xdr, const void *data) {
    auth_handle_encode(rqstp, xdr, NULL);
}

int initial_get_tasks_decode(struct rpc_rqst *rqstp, struct xdr_stream *xdr, void *data) {
    tasks* res = data;
    int rc = 0;

    {
        int _rc = 0;
        if (0 == rc
            && 0 > (_rc = xdr_stream_decode_u32(xdr, &(*res).size))) {
            rc = _rc;
        }
        if (0 == rc && MAX_TASKS < (*res).size) {
            rc = -EMSGSIZE;
        } else if (0 == rc && 0 != (*res).size) {
            (*res).data = kmalloc(sizeof(struct task) * (*res).size, GFP_KERNEL);
            if (NULL == (*res).data) {
                rc = -ENOMEM;
            } else {
                struct task* _base = (struct task* )((*res).data);
                struct task* base = _base;
                for (size_t i = 0; 0 == rc && (*res).size > i; i++) {
                    { // struct task
                        {
                            int _rc = 0;
                            if (0 == rc
                                && 0 > (_rc = xdr_stream_decode_u32(xdr, &((base[i]).name).size))) {
                                rc = _rc;
                            }
                            if (0 == rc && NAME_LEN < ((base[i]).name).size) {
                                rc = -EMSGSIZE;
                            } else if (0 == rc && 0 != ((base[i]).name).size) {
                                ((base[i]).name).data = kmalloc(sizeof(char) * ((base[i]).name).size, GFP_KERNEL);
                                if (NULL == ((base[i]).name).data) {
                                    rc = -ENOMEM;
                                } else {
                                    int _rc = 0;
                                    if (0 == rc
                                        && 0 > (_rc = xdr_stream_decode_string(xdr, ((base[i]).name).data, ((base[i]).name).size))) {
                                        rc = _rc;
                                    }
                                }
                            } else {
                                ((base[i]).name).data = NULL;
                            }
                        }
                        {
                            int _rc = 0;
                            if (0 == rc
                                && 0 > (_rc = xdr_stream_decode_u32(xdr, &((base[i]).pid)))) {
                                rc = _rc;
                            }
                        }
                        {
                            int _rc = 0;
                            if (0 == rc
                                && 0 > (_rc = xdr_stream_decode_u32(xdr, &((base[i]).state)))) {
                                rc = _rc;
                            }
                        }
                        {
                            int _rc = 0;
                            if (0 == rc
                                && 0 > (_rc = xdr_stream_decode_u32(xdr, &((base[i]).flags)))) {
                                rc = _rc;
                            }
                        }
                    }
                }
            }
        } else {
            (*res).data = NULL;
        }
    }

    return rc;
}

