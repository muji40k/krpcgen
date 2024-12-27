
# RPC kernel server

## Структура сервера

Один сервер --- одна программа

Возможен пул потоков ядра (создается всегда)

### Основная структура

Функция потока `sv_threadfn` определяется пользователем

``` c
/*
 * RPC service.
 *
 * An RPC service is a ``daemon,'' possibly multithreaded, which
 * receives and processes incoming RPC messages.
 * It has one or more transport sockets associated with it, and maintains
 * a list of idle threads waiting for input.
 *
 * We currently do not support more than one RPC program per daemon.
 */
struct svc_serv {
    struct svc_program *sv_program;     /* RPC program */
    struct svc_stat    *sv_stats;       /* RPC statistics */
    spinlock_t          sv_lock;
    struct kref         sv_refcnt;
    unsigned int        sv_nrthreads;   /* # of server threads */
    unsigned int        sv_maxconn;     /* max connections allowed or
                                         * '0' causing max to be based
                                         * on number of threads. */

    unsigned int        sv_max_payload; /* datagram payload size */
    unsigned int        sv_max_mesg;    /* max_payload + 1 page for overheads */
    unsigned int        sv_xdrsize;     /* XDR buffer size */
    struct list_head    sv_permsocks;   /* all permanent sockets */
    struct list_head    sv_tempsocks;   /* all temporary sockets */
    int                 sv_tmpcnt;      /* count of temporary sockets */
    struct timer_list   sv_temptimer;   /* timer for aging temporary sockets */

    char               *sv_name;        /* service name */

    unsigned int        sv_nrpools;     /* number of thread pools */
    struct svc_pool    *sv_pools;       /* array of thread pools */
    int               (*sv_threadfn)(void *data);

#if defined(CONFIG_SUNRPC_BACKCHANNEL)
    struct lwq          sv_cb_list;     /* queue for callback requests
                                         * that arrive over the same
                                         * connection */
    bool                sv_bc_enabled;  /* service uses backchannel */
#endif /* CONFIG_SUNRPC_BACKCHANNEL */
};
```

### Контекст потока
``` c
/*
 * The context of a single thread, including the request currently being
 * processed.
 */
struct svc_rqst {
    struct list_head    rq_all;      /* all threads list */
    struct llist_node   rq_idle;     /* On the idle list */
    struct rcu_head     rq_rcu_head; /* for RCU deferred kfree */
    struct svc_xprt *   rq_xprt;     /* transport ptr */

    struct sockaddr_storage rq_addr;    /* peer address */
    size_t                  rq_addrlen;
    struct sockaddr_storage rq_daddr;   /* dest addr of request
                                         *  - reply from here */
    size_t                  rq_daddrlen;

    struct svc_serv            *rq_server;    /* RPC service definition */
    struct svc_pool            *rq_pool;      /* thread pool */
    const struct svc_procedure *rq_procinfo;  /* procedure info */
    struct auth_ops            *rq_authop;    /* authentication flavour */
    struct svc_cred             rq_cred;      /* auth info */
    void                       *rq_xprt_ctxt; /* transport specific context ptr */
    struct svc_deferred_req    *rq_deferred;  /* deferred request we are replaying */

    struct xdr_buf      rq_arg;
    struct xdr_stream   rq_arg_stream;
    struct xdr_stream   rq_res_stream;
    struct page        *rq_scratch_page;
    struct xdr_buf      rq_res;
    struct page        *rq_pages[RPCSVC_MAXPAGES + 1];
    struct page       **rq_respages;  /* points into rq_pages */
    struct page       **rq_next_page; /* next reply page to use */
    struct page       **rq_page_end;  /* one past the last page */

    struct folio_batch  rq_fbatch;
    struct kvec         rq_vec[RPCSVC_MAXPAGES];  /* generally useful.. */
    struct bio_vec      rq_bvec[RPCSVC_MAXPAGES];

    __be32           rq_xid;          /* transmission id */
    u32              rq_prog;         /* program number */
    u32              rq_vers;         /* program version */
    u32              rq_proc;         /* procedure number */
    u32              rq_prot;         /* IP protocol */
    int              rq_cachetype;    /* catering to nfsd */
    unsigned long    rq_flags;        /* flags field */
    ktime_t          rq_qtime;        /* enqueue time */

    void            *rq_argp;         /* decoded arguments */
    void            *rq_resp;         /* xdr'd results */
    __be32          *rq_accept_statp;
    void            *rq_auth_data;    /* flavor-specific data */
    __be32           rq_auth_stat;    /* authentication status */
    int              rq_auth_slack;   /* extra space xdr code
                                       * should leave in head
                                       * for krb5i, krb5p. */
    int              rq_reserved;     /* space on socket outq
                                       * reserved for this request */
    ktime_t          rq_stime;        /* start time */

    struct cache_req rq_chandle;      /* handle passed to caches for 
                                       * request delaying */
    /* Catering to nfsd */
    struct auth_domain *rq_client;         /* RPC peer info */
    struct auth_domain *rq_gssclient;      /* "gss/"-style peer info */
    struct task_struct *rq_task;           /* service thread */
    struct net         *rq_bc_net;         /* pointer to backchannel's
                                            * net namespace */
    void **             rq_lease_breaker;  /* The v4 client breaking a lease */
    unsigned int        rq_status_counter; /* RPC processing counter */
};
```

### Функции и описание

#### Base

``` c
/**
 * svc_create - Create an RPC service
 * @prog: the RPC program the new service will handle
 * @bufsize: maximum message size for @prog
 * @threadfn: a function to service RPC requests for @prog
 *
 * Returns an instantiated struct svc_serv object or NULL.
 */
struct svc_serv *svc_create(struct svc_program *prog, unsigned int bufsize,
                            int (*threadfn)(void *data));

/**
 * svc_create_pooled - Create an RPC service with pooled threads
 * @prog: the RPC program the new service will handle
 * @bufsize: maximum message size for @prog
 * @threadfn: a function to service RPC requests for @prog
 *
 * Returns an instantiated struct svc_serv object or NULL.
 */
struct svc_serv *svc_create_pooled(struct svc_program *prog,
                                   unsigned int bufsize,
                                   int (*threadfn)(void *data));

int svc_bind(struct svc_serv *serv, struct net *net);

/**
 * svc_put - decrement reference count on a SUNRPC serv
 * @serv:  the svc_serv to have count decremented
 *
 * When the reference count reaches zero, svc_destroy()
 * is called to clean up and free the serv.
 */
static inline void svc_put(struct svc_serv *serv);
```

#### Обеспечение обработки запросов

Для обеспечения корректной работы поток должен выполнять следующую
последовательность действий

1. Сделать поток блокируемым `set_freezable` -> `svc_thread_wait_for_work`
1. В цикле производится производится обработка запросов
    1. Условие выхода `svc_thread_should_stop(rqstp) / !kthread_should_stop()`
    1. Получить запрос `svc_recv(rqstp)`
> Еще в примерах обновляют maxconn, непонятно зачем...

На самом деле `svc_recv(rqstp)` делает куда больше получения ...
и является начальной точкой обработки. Дальнейшая обработка
выполняется за счет функций объявленных в структурах `program`, `version`,
`procedure`.

Помимо функции потока, необходимо объявить функцию диспетчирезации
`struct svc_version -> vs_dispatch`, которая будет вызвана в случае удачного
получения, исходя из примеров, непосредственно вызывает неоюходимую процедуру
и указывает последующее действие при помощи кода возврата.

Коды возврата
1. [0] --- запрос выполнен, не отсылать ответ
1. [1] --- запрос выполнен, отсылать ответ `rqstp->rq_res`
1. ... ?

Получаемые запросы должны использовать один из следующих
[типов аутентификации](https://www.iana.org/assignments/rpc-authentication-numbers/rpc-authentication-numbers.xhtml)
1. None
2. Unix
3. TLS

> При этом, остальные тоже
> [определены](https://elixir.bootlin.com/linux/v6.7-rc3/source/include/linux/sunrpc/msg_prot.h#L16),
> ~~но непонятно поддерживаются ли... (скорее нет, так как~~ определен
[массив](https://elixir.bootlin.com/linux/v6.7-rc3/source/net/sunrpc/svcauth.c#L36))

> Есть возможность [замены](https://elixir.bootlin.com/linux/v6.7-rc6/source/net/sunrpc/svcauth.c#L141),
> но изначально определены только 3

Поле `struct sv_program -> pg_authenticate` позволяет определить
[дополнительную аутентификацию](https://elixir.bootlin.com/linux/v6.7-rc6/source/net/sunrpc/svc.c#L1340)
на уровне приложения (после оновной `svc_authenticate` (выставляет тип, согласно
полученному `flavour`))

> ToDo: не забыть рассмотреть очередь

``` c
/**
 * svc_thread_should_stop - check if this thread should stop
 * @rqstp: the thread that might need to stop
 *
 * To stop an svc thread, the pool flags SP_NEED_VICTIM and SP_VICTIM_REMAINS
 * are set.  The first thread which sees SP_NEED_VICTIM clears it, becoming
 * the victim using this function.  It should then promptly call
 * svc_exit_thread() to complete the process, clearing SP_VICTIM_REMAINS
 * so the task waiting for a thread to exit can wake and continue.
 *
 * Return values:
 *   %true: caller should invoke svc_exit_thread()
 *   %false: caller should do nothing
 */
static inline bool svc_thread_should_stop(struct svc_rqst *rqstp);

/**
 * svc_recv - Receive and process the next request on any transport
 * @rqstp: an idle RPC service thread
 *
 * This code is carefully organised not to touch any cachelines in
 * the shared svc_serv structure, only cachelines in the local
 * svc_pool.
 */
void svc_recv(struct svc_rqst *rqstp);
```

### Что вообще нужно определить?

Все
![](https://i.pinimg.com/originals/b1/3e/19/b13e1928a298443993655983be8577d2.jpg)

Но есть generic'и

### Generic'и

``` c
__be32 svc_generic_init_request(struct svc_rqst *rqstp,
                                const struct svc_program *progp,
                                struct svc_process_info *ret);

int svc_generic_rpcbind_set(struct net *net, const struct svc_program *progp,
                            u32 version, int family, unsigned short proto,
                            unsigned short port);

/**
 * svc_set_client - Assign an appropriate 'auth_domain' as the client
 * @rqstp: RPC execution context
 *
 * Return values:
 *   %SVC_OK: Client was found and assigned
 *   %SVC_DENY: Client was explicitly denied
 *   %SVC_DROP: Ignore this request
 *   %SVC_CLOSE: Ignore this request and close the connection
 */
enum svc_auth_status svc_set_client(struct svc_rqst *rqstp); // pg_authenticate
```

### Примеры из ядра

#### `nfsd`
##### Функция потока
``` c
/*
 * This is the NFS server kernel thread
 */
static int nfsd(void *vrqstp)
{
    struct svc_rqst *rqstp = (struct svc_rqst *) vrqstp;
    struct svc_xprt *perm_sock = list_entry(rqstp->rq_server->sv_permsocks.next,
                                            typeof(struct svc_xprt), xpt_list);
    struct net *net = perm_sock->xpt_net;
    struct nfsd_net *nn = net_generic(net, nfsd_net_id);

    /* At this point, the thread shares current->fs
     * with the init process. We need to create files with the
     * umask as defined by the client instead of init's umask. */
    if (unshare_fs_struct() < 0) {
        printk("Unable to start nfsd thread: out of memory\n");
        goto out;
    }

    current->fs->umask = 0;

    atomic_inc(&nfsdstats.th_cnt);

    set_freezable();

    /*
     * The main request loop
     */
    while (!svc_thread_should_stop(rqstp)) {
        /* Update sv_maxconn if it has changed */
        rqstp->rq_server->sv_maxconn = nn->max_connections;

        svc_recv(rqstp);
        validate_process_creds();
    }

    atomic_dec(&nfsdstats.th_cnt);

out:
    /* Release the thread */
    svc_exit_thread(rqstp);
    return 0;
}
```

##### Функция диспетчирезации
``` c
/**
 * nfsd_dispatch - Process an NFS or NFSACL Request
 * @rqstp: incoming request
 *
 * This RPC dispatcher integrates the NFS server's duplicate reply cache.
 *
 * Return values:
 *  %0: Processing complete; do not send a Reply
 *  %1: Processing complete; send Reply in rqstp->rq_res
 */
int nfsd_dispatch(struct svc_rqst *rqstp)
{
    const struct svc_procedure *proc = rqstp->rq_procinfo;
    __be32 *statp = rqstp->rq_accept_statp;
    struct nfsd_cacherep *rp;
    unsigned int start, len;
    __be32 *nfs_reply;

    /*
     * Give the xdr decoder a chance to change this if it wants
     * (necessary in the NFSv4.0 compound case)
     */
    rqstp->rq_cachetype = proc->pc_cachetype;

    /*
     * ->pc_decode advances the argument stream past the NFS
     * Call header, so grab the header's starting location and
     * size now for the call to nfsd_cache_lookup().
     */
    start = xdr_stream_pos(&rqstp->rq_arg_stream);
    len = xdr_stream_remaining(&rqstp->rq_arg_stream);
    if (!proc->pc_decode(rqstp, &rqstp->rq_arg_stream))
        goto out_decode_err;

    /*
     * Release rq_status_counter setting it to an odd value after the rpc
     * request has been properly parsed. rq_status_counter is used to
     * notify the consumers if the rqstp fields are stable
     * (rq_status_counter is odd) or not meaningful (rq_status_counter
     * is even).
     */
    smp_store_release(&rqstp->rq_status_counter, rqstp->rq_status_counter | 1);

    rp = NULL;
    switch (nfsd_cache_lookup(rqstp, start, len, &rp)) {
    case RC_DOIT:
        break;
    case RC_REPLY:
        goto out_cached_reply;
    case RC_DROPIT:
        goto out_dropit;
    }

    nfs_reply = xdr_inline_decode(&rqstp->rq_res_stream, 0);
    *statp = proc->pc_func(rqstp);
    if (test_bit(RQ_DROPME, &rqstp->rq_flags))
        goto out_update_drop;

    if (!proc->pc_encode(rqstp, &rqstp->rq_res_stream))
        goto out_encode_err;

    /*
     * Release rq_status_counter setting it to an even value after the rpc
     * request has been properly processed.
     */
    smp_store_release(&rqstp->rq_status_counter, rqstp->rq_status_counter + 1);

    nfsd_cache_update(rqstp, rp, rqstp->rq_cachetype, nfs_reply);
out_cached_reply:
    return 1;

out_decode_err:
    trace_nfsd_garbage_args_err(rqstp);
    *statp = rpc_garbage_args;
    return 1;

out_update_drop:
    nfsd_cache_update(rqstp, rp, RC_NOCACHE, NULL);
out_dropit:
    return 0;

out_encode_err:
    trace_nfsd_cant_encode_err(rqstp);
    nfsd_cache_update(rqstp, rp, RC_NOCACHE, NULL);
    *statp = rpc_system_err;
    return 1;
}
```

#### `lockd`

##### Функция потока
``` c
/*
 * This is the lockd kernel thread
 */
static int lockd(void *vrqstp)
{
    struct svc_rqst *rqstp = vrqstp;
    struct net *net = &init_net;
    struct lockd_net *ln = net_generic(net, lockd_net_id);

    /* try_to_freeze() is called from svc_recv() */
    set_freezable();

    dprintk("NFS locking service started (ver " LOCKD_VERSION ").\n");

    /*
     * The main request loop. We don't terminate until the last
     * NFS mount or NFS daemon has gone away.
     */
    while (!svc_thread_should_stop(rqstp)) {
        /* update sv_maxconn if it has changed */
        rqstp->rq_server->sv_maxconn = nlm_max_connections;

        nlmsvc_retry_blocked(rqstp);
        svc_recv(rqstp);
    }
    if (nlmsvc_ops)
        nlmsvc_invalidate_all();
    nlm_shutdown_hosts();
    cancel_delayed_work_sync(&ln->grace_period_end);
    locks_end_grace(&ln->lockd_manager);

    dprintk("lockd_down: service stopped\n");

    svc_exit_thread(rqstp);
    return 0;
}
```

##### Функция диспетчирезации
``` c
/**
 * nlmsvc_dispatch - Process an NLM Request
 * @rqstp: incoming request
 *
 * Return values:
 *  %0: Processing complete; do not send a Reply
 *  %1: Processing complete; send Reply in rqstp->rq_res
 */
static int nlmsvc_dispatch(struct svc_rqst *rqstp)
{
    const struct svc_procedure *procp = rqstp->rq_procinfo;
    __be32 *statp = rqstp->rq_accept_statp;

    if (!procp->pc_decode(rqstp, &rqstp->rq_arg_stream))
        goto out_decode_err;

    *statp = procp->pc_func(rqstp);
    if (*statp == rpc_drop_reply)
        return 0;
    if (*statp != rpc_success)
        return 1;

    if (!procp->pc_encode(rqstp, &rqstp->rq_res_stream))
        goto out_encode_err;

    return 1;

out_decode_err:
    *statp = rpc_garbage_args;
    return 1;

out_encode_err:
    *statp = rpc_system_err;
    return 1;
}
```

##### Функция аутентификации
``` c
static enum svc_auth_status lockd_authenticate(struct svc_rqst *rqstp)
{
    rqstp->rq_client = NULL;
    switch (rqstp->rq_authop->flavour) {
        case RPC_AUTH_NULL:
        case RPC_AUTH_UNIX:
            rqstp->rq_auth_stat = rpc_auth_ok;
            if (rqstp->rq_proc == 0)
                return SVC_OK;
            if (is_callback(rqstp->rq_proc)) {
                /* Leave it to individual procedures to
                 * call nlmsvc_lookup_host(rqstp)
                 */
                return SVC_OK;
            }
            return svc_set_client(rqstp);
    }
    rqstp->rq_auth_stat = rpc_autherr_badcred;
    return SVC_DENIED;
}
```

##### Примеры заполнения
``` c
/*
 * Define NLM program and procedures
 */
static DEFINE_PER_CPU_ALIGNED(unsigned long, nlmsvc_version1_count[17]);
static const struct svc_version nlmsvc_version1 = {
    .vs_vers    = 1,
    .vs_nproc   = 17,
    .vs_proc    = nlmsvc_procedures,
    .vs_count   = nlmsvc_version1_count,
    .vs_dispatch    = nlmsvc_dispatch,
    .vs_xdrsize = NLMSVC_XDRSIZE,
};

static DEFINE_PER_CPU_ALIGNED(unsigned long,
                  nlmsvc_version3_count[ARRAY_SIZE(nlmsvc_procedures)]);
static const struct svc_version nlmsvc_version3 = {
    .vs_vers    = 3,
    .vs_nproc   = ARRAY_SIZE(nlmsvc_procedures),
    .vs_proc    = nlmsvc_procedures,
    .vs_count   = nlmsvc_version3_count,
    .vs_dispatch    = nlmsvc_dispatch,
    .vs_xdrsize = NLMSVC_XDRSIZE,
};

#ifdef CONFIG_LOCKD_V4
static DEFINE_PER_CPU_ALIGNED(unsigned long,
                  nlmsvc_version4_count[ARRAY_SIZE(nlmsvc_procedures4)]);
static const struct svc_version nlmsvc_version4 = {
    .vs_vers    = 4,
    .vs_nproc   = ARRAY_SIZE(nlmsvc_procedures4),
    .vs_proc    = nlmsvc_procedures4,
    .vs_count   = nlmsvc_version4_count,
    .vs_dispatch    = nlmsvc_dispatch,
    .vs_xdrsize = NLMSVC_XDRSIZE,
};
#endif

static const struct svc_version *nlmsvc_version[] = {
    [1] = &nlmsvc_version1,
    [3] = &nlmsvc_version3,
#ifdef CONFIG_LOCKD_V4
    [4] = &nlmsvc_version4,
#endif
};

static struct svc_stat      nlmsvc_stats;

#define NLM_NRVERS  ARRAY_SIZE(nlmsvc_version)
static struct svc_program   nlmsvc_program = {
    .pg_prog        = NLM_PROGRAM,      /* program number */
    .pg_nvers       = NLM_NRVERS,       /* number of entries in nlmsvc_version */
    .pg_vers        = nlmsvc_version,   /* version table */
    .pg_name        = "lockd",      /* service name */
    .pg_class       = "nfsd",       /* share authentication with nfsd */
    .pg_stats       = &nlmsvc_stats,    /* stats table */
    .pg_authenticate    = &lockd_authenticate,  /* export authentication */
    .pg_init_request    = svc_generic_init_request,
    .pg_rpcbind_set     = svc_generic_rpcbind_set,
};

const struct svc_procedure nlmsvc_procedures[24] = {
    [NLMPROC_NULL] = {
        .pc_func = nlmsvc_proc_null,
        .pc_decode = nlmsvc_decode_void,
        .pc_encode = nlmsvc_encode_void,
        .pc_argsize = sizeof(struct nlm_void),
        .pc_argzero = sizeof(struct nlm_void),
        .pc_ressize = sizeof(struct nlm_void),
        .pc_xdrressize = St,
        .pc_name = "NULL",
    },
    [NLMPROC_TEST] = {
        .pc_func = nlmsvc_proc_test,
        .pc_decode = nlmsvc_decode_testargs,
        .pc_encode = nlmsvc_encode_testres,
        .pc_argsize = sizeof(struct nlm_args),
        .pc_argzero = sizeof(struct nlm_args),
        .pc_ressize = sizeof(struct nlm_res),
        .pc_xdrressize = Ck+St+2+No+Rg,
        .pc_name = "TEST",
    },
    [NLMPROC_LOCK] = {
        .pc_func = nlmsvc_proc_lock,
        .pc_decode = nlmsvc_decode_lockargs,
        .pc_encode = nlmsvc_encode_res,
        .pc_argsize = sizeof(struct nlm_args),
        .pc_argzero = sizeof(struct nlm_args),
        .pc_ressize = sizeof(struct nlm_res),
        .pc_xdrressize = Ck+St,
        .pc_name = "LOCK",
    },
    [NLMPROC_CANCEL] = {
        .pc_func = nlmsvc_proc_cancel,
        .pc_decode = nlmsvc_decode_cancargs,
        .pc_encode = nlmsvc_encode_res,
        .pc_argsize = sizeof(struct nlm_args),
        .pc_argzero = sizeof(struct nlm_args),
        .pc_ressize = sizeof(struct nlm_res),
        .pc_xdrressize = Ck+St,
        .pc_name = "CANCEL",
    },
    [NLMPROC_UNLOCK] = {
        .pc_func = nlmsvc_proc_unlock,
        .pc_decode = nlmsvc_decode_unlockargs,
        .pc_encode = nlmsvc_encode_res,
        .pc_argsize = sizeof(struct nlm_args),
        .pc_argzero = sizeof(struct nlm_args),
        .pc_ressize = sizeof(struct nlm_res),
        .pc_xdrressize = Ck+St,
        .pc_name = "UNLOCK",
    },
    [NLMPROC_GRANTED] = {
        .pc_func = nlmsvc_proc_granted,
        .pc_decode = nlmsvc_decode_testargs,
        .pc_encode = nlmsvc_encode_res,
        .pc_argsize = sizeof(struct nlm_args),
        .pc_argzero = sizeof(struct nlm_args),
        .pc_ressize = sizeof(struct nlm_res),
        .pc_xdrressize = Ck+St,
        .pc_name = "GRANTED",
    },
    [NLMPROC_TEST_MSG] = {
        .pc_func = nlmsvc_proc_test_msg,
        .pc_decode = nlmsvc_decode_testargs,
        .pc_encode = nlmsvc_encode_void,
        .pc_argsize = sizeof(struct nlm_args),
        .pc_argzero = sizeof(struct nlm_args),
        .pc_ressize = sizeof(struct nlm_void),
        .pc_xdrressize = St,
        .pc_name = "TEST_MSG",
    },
    [NLMPROC_LOCK_MSG] = {
        .pc_func = nlmsvc_proc_lock_msg,
        .pc_decode = nlmsvc_decode_lockargs,
        .pc_encode = nlmsvc_encode_void,
        .pc_argsize = sizeof(struct nlm_args),
        .pc_argzero = sizeof(struct nlm_args),
        .pc_ressize = sizeof(struct nlm_void),
        .pc_xdrressize = St,
        .pc_name = "LOCK_MSG",
    },
    [NLMPROC_CANCEL_MSG] = {
        .pc_func = nlmsvc_proc_cancel_msg,
        .pc_decode = nlmsvc_decode_cancargs,
        .pc_encode = nlmsvc_encode_void,
        .pc_argsize = sizeof(struct nlm_args),
        .pc_argzero = sizeof(struct nlm_args),
        .pc_ressize = sizeof(struct nlm_void),
        .pc_xdrressize = St,
        .pc_name = "CANCEL_MSG",
    },
    [NLMPROC_UNLOCK_MSG] = {
        .pc_func = nlmsvc_proc_unlock_msg,
        .pc_decode = nlmsvc_decode_unlockargs,
        .pc_encode = nlmsvc_encode_void,
        .pc_argsize = sizeof(struct nlm_args),
        .pc_argzero = sizeof(struct nlm_args),
        .pc_ressize = sizeof(struct nlm_void),
        .pc_xdrressize = St,
        .pc_name = "UNLOCK_MSG",
    },
    [NLMPROC_GRANTED_MSG] = {
        .pc_func = nlmsvc_proc_granted_msg,
        .pc_decode = nlmsvc_decode_testargs,
        .pc_encode = nlmsvc_encode_void,
        .pc_argsize = sizeof(struct nlm_args),
        .pc_argzero = sizeof(struct nlm_args),
        .pc_ressize = sizeof(struct nlm_void),
        .pc_xdrressize = St,
        .pc_name = "GRANTED_MSG",
    },
    [NLMPROC_TEST_RES] = {
        .pc_func = nlmsvc_proc_null,
        .pc_decode = nlmsvc_decode_void,
        .pc_encode = nlmsvc_encode_void,
        .pc_argsize = sizeof(struct nlm_res),
        .pc_argzero = sizeof(struct nlm_res),
        .pc_ressize = sizeof(struct nlm_void),
        .pc_xdrressize = St,
        .pc_name = "TEST_RES",
    },
    [NLMPROC_LOCK_RES] = {
        .pc_func = nlmsvc_proc_null,
        .pc_decode = nlmsvc_decode_void,
        .pc_encode = nlmsvc_encode_void,
        .pc_argsize = sizeof(struct nlm_res),
        .pc_argzero = sizeof(struct nlm_res),
        .pc_ressize = sizeof(struct nlm_void),
        .pc_xdrressize = St,
        .pc_name = "LOCK_RES",
    },
    [NLMPROC_CANCEL_RES] = {
        .pc_func = nlmsvc_proc_null,
        .pc_decode = nlmsvc_decode_void,
        .pc_encode = nlmsvc_encode_void,
        .pc_argsize = sizeof(struct nlm_res),
        .pc_argzero = sizeof(struct nlm_res),
        .pc_ressize = sizeof(struct nlm_void),
        .pc_xdrressize = St,
        .pc_name = "CANCEL_RES",
    },
    [NLMPROC_UNLOCK_RES] = {
        .pc_func = nlmsvc_proc_null,
        .pc_decode = nlmsvc_decode_void,
        .pc_encode = nlmsvc_encode_void,
        .pc_argsize = sizeof(struct nlm_res),
        .pc_argzero = sizeof(struct nlm_res),
        .pc_ressize = sizeof(struct nlm_void),
        .pc_xdrressize = St,
        .pc_name = "UNLOCK_RES",
    },
    [NLMPROC_GRANTED_RES] = {
        .pc_func = nlmsvc_proc_granted_res,
        .pc_decode = nlmsvc_decode_res,
        .pc_encode = nlmsvc_encode_void,
        .pc_argsize = sizeof(struct nlm_res),
        .pc_argzero = sizeof(struct nlm_res),
        .pc_ressize = sizeof(struct nlm_void),
        .pc_xdrressize = St,
        .pc_name = "GRANTED_RES",
    },
    [NLMPROC_NSM_NOTIFY] = {
        .pc_func = nlmsvc_proc_sm_notify,
        .pc_decode = nlmsvc_decode_reboot,
        .pc_encode = nlmsvc_encode_void,
        .pc_argsize = sizeof(struct nlm_reboot),
        .pc_argzero = sizeof(struct nlm_reboot),
        .pc_ressize = sizeof(struct nlm_void),
        .pc_xdrressize = St,
        .pc_name = "SM_NOTIFY",
    },
    [17] = {
        .pc_func = nlmsvc_proc_unused,
        .pc_decode = nlmsvc_decode_void,
        .pc_encode = nlmsvc_encode_void,
        .pc_argsize = sizeof(struct nlm_void),
        .pc_argzero = sizeof(struct nlm_void),
        .pc_ressize = sizeof(struct nlm_void),
        .pc_xdrressize = St,
        .pc_name = "UNUSED",
    },
    [18] = {
        .pc_func = nlmsvc_proc_unused,
        .pc_decode = nlmsvc_decode_void,
        .pc_encode = nlmsvc_encode_void,
        .pc_argsize = sizeof(struct nlm_void),
        .pc_argzero = sizeof(struct nlm_void),
        .pc_ressize = sizeof(struct nlm_void),
        .pc_xdrressize = St,
        .pc_name = "UNUSED",
    },
    [19] = {
        .pc_func = nlmsvc_proc_unused,
        .pc_decode = nlmsvc_decode_void,
        .pc_encode = nlmsvc_encode_void,
        .pc_argsize = sizeof(struct nlm_void),
        .pc_argzero = sizeof(struct nlm_void),
        .pc_ressize = sizeof(struct nlm_void),
        .pc_xdrressize = St,
        .pc_name = "UNUSED",
    },
    [NLMPROC_SHARE] = {
        .pc_func = nlmsvc_proc_share,
        .pc_decode = nlmsvc_decode_shareargs,
        .pc_encode = nlmsvc_encode_shareres,
        .pc_argsize = sizeof(struct nlm_args),
        .pc_argzero = sizeof(struct nlm_args),
        .pc_ressize = sizeof(struct nlm_res),
        .pc_xdrressize = Ck+St+1,
        .pc_name = "SHARE",
    },
    [NLMPROC_UNSHARE] = {
        .pc_func = nlmsvc_proc_unshare,
        .pc_decode = nlmsvc_decode_shareargs,
        .pc_encode = nlmsvc_encode_shareres,
        .pc_argsize = sizeof(struct nlm_args),
        .pc_argzero = sizeof(struct nlm_args),
        .pc_ressize = sizeof(struct nlm_res),
        .pc_xdrressize = Ck+St+1,
        .pc_name = "UNSHARE",
    },
    [NLMPROC_NM_LOCK] = {
        .pc_func = nlmsvc_proc_nm_lock,
        .pc_decode = nlmsvc_decode_lockargs,
        .pc_encode = nlmsvc_encode_res,
        .pc_argsize = sizeof(struct nlm_args),
        .pc_argzero = sizeof(struct nlm_args),
        .pc_ressize = sizeof(struct nlm_res),
        .pc_xdrressize = Ck+St,
        .pc_name = "NM_LOCK",
    },
    [NLMPROC_FREE_ALL] = {
        .pc_func = nlmsvc_proc_free_all,
        .pc_decode = nlmsvc_decode_notify,
        .pc_encode = nlmsvc_encode_void,
        .pc_argsize = sizeof(struct nlm_args),
        .pc_argzero = sizeof(struct nlm_args),
        .pc_ressize = sizeof(struct nlm_void),
        .pc_xdrressize = 0,
        .pc_name = "FREE_ALL",
    },
};
```

