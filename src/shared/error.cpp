#include "error.h"
#include <stdio.h>
#include <stdlib.h>

void error [[noreturn]] (const char* message, ...) {
    fprintf(stderr, "Error: ");
    va_list args;
    va_start (args, message);
    vfprintf (stderr, message, args);
    va_end (args);
#ifdef NDEBUG
    exit(1);
#else
    abort();
#endif
}


void verror [[noreturn]] (const char* message, va_list args) {
    fprintf(stderr, "Error: ");
    vfprintf (stderr, message, args);
#ifdef NDEBUG
    exit(1);
#else
    abort();
#endif
}