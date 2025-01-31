
const NAME_LEN = 16;
const MAX_TASKS = 10;

struct task {
    string name<NAME_LEN>;
    int pid;
    unsigned int state;
    unsigned int flags;
};

typedef struct task tasks<MAX_TASKS>;

program example {
    version initial {
        tasks get_tasks(void) = 1;
    } = 1;
} = 0x20000001;


