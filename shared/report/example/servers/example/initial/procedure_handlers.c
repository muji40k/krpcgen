#include "procedures.h"
#include <linux/init_task.h>

static __be32 init_result(tasks* res) {
    res->data = kmalloc(MAX_TASKS * sizeof(struct task), GFP_KERNEL);

    if (NULL == res->data) {
        return rpc_system_err;
    }

    for (size_t i = 0; MAX_TASKS > i; i++) {
        struct task *task = ((struct task *)res->data) + i;
        task->name.data = kmalloc(NAME_LEN * sizeof(char), GFP_KERNEL);
        task->name.size = 0;

        if (NULL == task->name.data) {
            return rpc_system_err;
        }
    }

    return rpc_success;
}

__be32 initial_get_tasks_handler(struct svc_rqst *rqstp) {
    tasks* res = rqstp->rq_resp;
    __be32 out = init_result(res);

    if (rpc_success != out) {
        return out;
    }

    struct task *tasks = res->data;
    const struct task_struct *head = &init_task;
    size_t i = 0;

    do {
        struct task *task = tasks + i;
        strncpy(task->name.data, head->comm, NAME_LEN);
        task->name.size = strlen(head->comm);
        task->pid = head->pid;
        task->flags = head->flags;
        task->state = head->__state;
    } while (MAX_TASKS > ++i && (head = next_task(head)) != &init_task);

    res->size = i;

    return rpc_success;
}

