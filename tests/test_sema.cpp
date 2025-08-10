#include "runtime/builtins.h"
#include "testing.h"
#include "compiler/sema.h"
#include "compiler/parser.h"

using namespace luna::compiler;

#define CASE_SUCCESS(source) { \
    Parser parser(to_source(source)); \
    auto module = parser.parse_module().value(); \
    Sema sema; \
    TEST_ASSERT(!sema.check(module, &env).has_value()); \
}

#define CASE_FAILURE(source) { \
    Parser parser(to_source(source)); \
    auto module = parser.parse_module().value(); \
    Sema sema; \
    TEST_ASSERT(sema.check(module, &env).has_value()); \
}

int main(const int argc, const char** argv) {
    luna::Environment env;
    luna::load_builtins(&env);

    CASE_SUCCESS(
        "func test() {" \
            "let a = 10;" \
        "}"
    )

    CASE_SUCCESS(
        "func test() {" \
            "let a: [][]int = [[]];" \
        "}"
    )

    CASE_FAILURE(
        "func test() {" \
            "let a: []int = [[]];" \
        "}"
    )

    return 0;
}