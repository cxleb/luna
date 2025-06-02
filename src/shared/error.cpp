#include "error.h"
#include <stdio.h>
#include <stdlib.h>

namespace luna {

Error error(const char* message, ...) {
    char buf[500];
    va_list args;
    va_start (args, message);
    auto count = vsnprintf (buf, 500, message, args);
    va_end (args);
    return std::string(buf, count);
}


Error verror (const char* message, va_list args) {
    char buf[500];
    fprintf(stderr, "Error: ");
    auto count = vsnprintf (buf, 500, message, args);
    vfprintf (stderr, message, args);
    return std::string(buf, count);
}

}