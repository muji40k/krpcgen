# RPC main stuff

## Основные структуры

``` c
/*
 * List of RPC programs on the same transport endpoint
 */
struct svc_program {
    struct svc_program *       pg_next;    /* other programs (same xprt) */
    u32                        pg_prog;    /* program number */
    unsigned int               pg_lovers;  /* lowest version */
    unsigned int               pg_hivers;  /* highest version */
    unsigned int               pg_nvers;   /* number of versions */
    const struct svc_version **pg_vers;    /* version array */
    char                      *pg_name;    /* service name */
    char                      *pg_class;   /* class name: services sharing authentication */
    struct svc_stat           *pg_stats;   /* rpc statistics */
    enum svc_auth_status     (*pg_authenticate)(struct svc_rqst *rqstp);
    __be32                   (*pg_init_request)(struct svc_rqst *,
                                                const struct svc_program *,
                                                struct svc_process_info *);
    int                      (*pg_rpcbind_set)(struct net *net,
                                               const struct svc_program *,
                                               u32 version, int family,
                                               unsigned short proto,
                                               unsigned short port);
};

/*
 * RPC program version
 */
struct svc_version {
    u32                         vs_vers;    /* version number */
    u32                         vs_nproc;   /* number of procedures */
    const struct svc_procedure *vs_proc;    /* per-procedure info */
    unsigned long __percpu     *vs_count;   /* call counts */
    u32                         vs_xdrsize; /* xdrsize needed for this version */

    /* Don't register with rpcbind */
    bool  vs_hidden;

    /* Don't care if the rpcbind registration fails */
    bool  vs_rpcb_optnl;

    /* Need xprt with congestion control */
    bool  vs_need_cong_ctrl;

    /* Dispatch function */
    int (*vs_dispatch)(struct svc_rqst *rqstp);
};

/*
 * RPC procedure info
 */
struct svc_procedure {
    /* process the request: */
    __be32       (*pc_func)(struct svc_rqst *);
    /* XDR decode args: */
    bool         (*pc_decode)(struct svc_rqst *rqstp, struct xdr_stream *xdr);
    /* XDR encode result: */
    bool         (*pc_encode)(struct svc_rqst *rqstp, struct xdr_stream *xdr);
    /* XDR free result: */
    void         (*pc_release)(struct svc_rqst *);
    unsigned int   pc_argsize;    /* argument struct size */
    unsigned int   pc_argzero;    /* how much of argument to clear */
    unsigned int   pc_ressize;    /* result struct size */
    unsigned int   pc_cachetype;  /* cache info (NFS) */
    unsigned int   pc_xdrressize; /* maximum size of XDR reply */
    const char    *pc_name;       /* for display */
};
```

