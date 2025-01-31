#include "version.h"
#include "../../../types.h"
#include "../../common.h"
#include "../authentication.h"
#include "constants.h"

#include "procedures.h"

static const struct svc_procedure initial_procedures[] = {
    [get_tasks] = {
        .pc_func = initial_get_tasks_handler,
        .pc_decode = initial_get_tasks_decode,
        .pc_encode = initial_get_tasks_encode,
        .pc_argsize = 0,
        .pc_argzero = 0,
        .pc_release = initial_get_tasks_release,
        .pc_ressize = sizeof(tasks),
        .pc_xdrressize = sizeof(u32)+MAX_TASKS*(sizeof(u32)+NAME_LEN*(sizeof(char))+sizeof(s32)+sizeof(u32)+sizeof(u32)),
        .pc_name = "get_tasks",
    },
};

static unsigned long initial_call_count = 0;
const struct svc_version initial_version = {
    .vs_vers = initial,
    .vs_nproc = ARRAY_SIZE(initial_procedures),
    .vs_proc = initial_procedures,
    .vs_count = &initial_call_count,
    .vs_dispatch = dispatch,
    .vs_xdrsize = AUTH_HANDLE_SIZE+sizeof(u32)+MAX_TASKS*(sizeof(u32)+NAME_LEN*(sizeof(char))+sizeof(s32)+sizeof(u32)+sizeof(u32)),
    .vs_hidden = false,
    .vs_rpcb_optnl = false,
    .vs_need_cong_ctrl = false
};
