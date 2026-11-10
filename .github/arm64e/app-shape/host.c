// A C host for the Rust static library in lib.rs, built and linked by clang.
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

typedef struct {
    uint32_t (*add)(uint32_t, uint32_t);
} Callbacks;

typedef struct {
    uint32_t (*square)(uint32_t);
} RustTable;

extern int32_t rust_constructor_ran(void);
extern void rust_register(Callbacks callbacks);
extern uint32_t rust_call_registered_on_thread(uint32_t a, uint32_t b);
extern RustTable rust_table(void);
extern uint32_t rust_sort_with_qsort(void);
extern int32_t rust_checked(uint32_t divisor);
extern void rust_escaping_panic(void);
extern int32_t rust_on_foreign_thread(void);

static uint32_t host_add(uint32_t a, uint32_t b) { return a + b; }

static void *foreign_thread(void *result) {
    *(int32_t *)result = rust_on_foreign_thread();
    return NULL;
}

int main(int argc, char **argv) {
    if (argc > 1 && strcmp(argv[1], "escape") == 0) {
        rust_escaping_panic();
        return 0;
    }
    rust_register((Callbacks){host_add});
    uint32_t callback = rust_call_registered_on_thread(40, 2);
    RustTable table = rust_table();
    int32_t foreign = 0;
    pthread_t thread;
    pthread_create(&thread, NULL, foreign_thread, &foreign);
    pthread_join(thread, NULL);
    printf("constructor=%d callback=%u rust_table=%u qsort=%u caught=%d foreign_thread=%d\n",
           rust_constructor_ran(), callback, table.square(7), rust_sort_with_qsort(),
           rust_checked(0), foreign);
    return 0;
}
