#ifndef _CLIENTS_EXAMPLE_INITIAL_PROCEDURE_API_H_
#define _CLIENTS_EXAMPLE_INITIAL_PROCEDURE_API_H_

#include <linux/sunrpc/clnt.h>
#include <linux/sunrpc/xdr.h>

#include "../../../types.h"

typedef struct {
    tasks value;
    int error;
} get_tasks_result_t;

get_tasks_result_t example_initial_get_tasks(void);
void release_result_example_initial_get_tasks(tasks* result);


#endif
