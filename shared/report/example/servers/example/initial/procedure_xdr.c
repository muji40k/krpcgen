#include "procedures.h"

bool initial_get_tasks_decode(struct svc_rqst *rqstp, struct xdr_stream *xdr) {
    return true;
}

bool initial_get_tasks_encode(struct svc_rqst *rqstp, struct xdr_stream *xdr) {
    tasks* res = rqstp->rq_resp;
    int rc = 0;

    {
        int _rc = 0;
        if (0 == rc
            && 0 != (*res).size
            && NULL == (*res).data) {
            rc = -EINVAL;
        }
        if (0 == rc && MAX_TASKS < (*res).size) {
            rc = -EMSGSIZE;
        }
        if (0 == rc
            && 0 > (_rc = xdr_stream_encode_u32(xdr, (*res).size))) {
            rc = _rc;
        }
        if (0 == rc && 0 != (*res).size) {
            struct task* _base = (struct task* )((*res).data);
            struct task* base = _base;
            for (size_t i = 0; 0 == rc && (*res).size > i; i++) {
                { // struct task
                    {
                        int _rc = 0;
                        if (0 == rc
                            && 0 != ((base[i]).name).size
                            && NULL == ((base[i]).name).data) {
                            rc = -EINVAL;
                        }
                        if (0 == rc && NAME_LEN < ((base[i]).name).size) {
                            rc = -EMSGSIZE;
                        }
                        if (0 == rc
                            && 0 > (_rc = xdr_stream_encode_u32(xdr, ((base[i]).name).size))) {
                            rc = _rc;
                        }
                        if (0 == rc && 0 != ((base[i]).name).size) {
                            int _rc = 0;
                            if (0 == rc
                                && 0 > (_rc = xdr_stream_encode_opaque(xdr, ((base[i]).name).data, ((base[i]).name).size))) {
                                rc = _rc;
                            }
                        }
                    }
                    {
                        int _rc = 0;
                        if (0 == rc
                            && 0 > (_rc = xdr_stream_encode_u32(xdr, (base[i]).pid))) {
                            rc = _rc;
                        }
                    }
                    {
                        int _rc = 0;
                        if (0 == rc
                            && 0 > (_rc = xdr_stream_encode_u32(xdr, (base[i]).state))) {
                            rc = _rc;
                        }
                    }
                    {
                        int _rc = 0;
                        if (0 == rc
                            && 0 > (_rc = xdr_stream_encode_u32(xdr, (base[i]).flags))) {
                            rc = _rc;
                        }
                    }
                }
            }
        }
    }

    if (0 != rc) {
        return false;
    }

    return true;
}

void initial_get_tasks_release(struct svc_rqst *rqstp) {
    tasks* res = rqstp->rq_resp;

    if (NULL != (*res).data) {
        struct task* _base = (struct task* )((*res).data);
        struct task* base = _base;
        for (size_t i = 0; (*res).size > i; i++) {
            { // struct task
                if (NULL != ((base[i]).name).data) {
                    kfree(((base[i]).name).data);
                    ((base[i]).name).data = NULL;
                    ((base[i]).name).size = 0;
                }
            }
        }
        kfree((*res).data);
        (*res).data = NULL;
        (*res).size = 0;
    }

}

