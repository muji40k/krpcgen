#ifndef _TYPES_H_
#define _TYPES_H_

#include <linux/module.h>

#include "constants.h"

#define STATIC_MAX(a, b) (((a) > (b)) ? (a) : (b))

struct _vla {
    u32 size;   // Amount of elements (For more information see the specification)
    void *data;
};
typedef struct _vla vla_t;
typedef struct _vla string_t;
#define vla(type) vla_t

struct task {
    string_t name;
    s32 pid;
    u32 state;
    u32 flags;
};

typedef vla(struct task) tasks;


#endif
