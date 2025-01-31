#include "procedure_api.h"
#include "procedures.h"
#include "version.h"
#include "constants.h"
#include "../../client.h"

get_tasks_result_t example_initial_get_tasks(void) {
    get_tasks_result_t res;
    struct rpc_clnt *client = client_get();

    if (IS_ERR(client)) {
        res.error = PTR_ERR(client);
    } else if (NULL == client) {
        res.error = -EINVAL;
    } else {
        struct rpc_message msg = {
            .rpc_proc = &initial_procedures[get_tasks],
            .rpc_resp = &res.value,
            .rpc_cred = get_current_cred(),
        };

        res.error = rpc_call_sync(client, &msg, 0);
    }

    return res;
}
EXPORT_SYMBOL(example_initial_get_tasks);

void release_result_example_initial_get_tasks(tasks* result) {
    if (NULL == result) {
        return;
    }

    if (NULL != (*result).data) {
        struct task* _base = (struct task* )((*result).data);
        struct task* base = _base;
        for (size_t i = 0; (*result).size > i; i++) {
            { // struct task
                if (NULL != ((base[i]).name).data) {
                    kfree(((base[i]).name).data);
                    ((base[i]).name).data = NULL;
                    ((base[i]).name).size = 0;
                }
            }
        }
        kfree((*result).data);
        (*result).data = NULL;
        (*result).size = 0;
    }
}
EXPORT_SYMBOL(release_result_example_initial_get_tasks);

