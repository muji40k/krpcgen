#include <linux/module.h>
#include <linux/kernel.h>

#include "clients/example/initial/procedure_api.h"

MODULE_LICENSE("GPL");

static int __init init_md(void)
{
    get_tasks_result_t res = example_initial_get_tasks();

    if (0 == res.error) {
        tasks *wrap = &res.value;
        struct task *t = wrap->data;

        for (size_t i = 0; wrap->size > i; i++) {
            struct task *task = t + i;
            printk("+ self - %s, pid - %d, state - %d, flags - %d\n",
                   (char *)task->name.data, task->pid, task->state,
                   task->flags);
        }
    }

    release_result_example_initial_get_tasks(&res.value);

    return res.error;
}

static void __exit exit_md(void) {}

module_init(init_md);
module_exit(exit_md);


