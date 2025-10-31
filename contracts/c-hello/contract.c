#include <stdint.h>

// Exported entry points must be explicitly annotated so clang keeps them
// visible after dead-code elimination. Using export_name keeps the signature
// stable without requiring a linker script.
__attribute__((export_name("sum")))
int64_t sum(int64_t a, int64_t b) {
    return a + b;
}

__attribute__((export_name("version")))
int32_t version(void) {
    return 1;
}

// Simple storage-free counter that demonstrates control flow in straight C.
__attribute__((export_name("clamp_add")))
int64_t clamp_add(int64_t value, int64_t delta, int64_t max) {
    int64_t next = value + delta;
    if (next > max) {
        return max;
    }
    return next;
}
