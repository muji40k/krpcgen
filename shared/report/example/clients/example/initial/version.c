#include "version.h"
#include "constants.h"
#include "../authentication.h"

#include "procedures.h"

const struct rpc_procinfo initial_procedures[] = {
    [get_tasks] = {
        .p_proc = get_tasks,
        .p_encode = initial_get_tasks_encode,
        .p_decode = initial_get_tasks_decode,
        .p_arglen = AUTH_HANDLE_SIZE+0,
        .p_replen = sizeof(u32)+MAX_TASKS*(sizeof(u32)+NAME_LEN*(sizeof(char))+sizeof(s32)+sizeof(u32)+sizeof(u32)),
        .p_statidx = get_tasks,
        .p_name = "get_tasks",
    },
};

static unsigned int initial_call_count = 0;
const struct rpc_version initial_version = {
    .number = initial,
    .nrprocs = ARRAY_SIZE(initial_procedures),
    .procs = initial_procedures,
    .counts = &initial_call_count,
};
