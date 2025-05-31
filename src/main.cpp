#include "shared/environment.h"
#include "shared/utils.h"

#include "compiler/parser.h"
#include "compiler/gen.h"

#include "runtime/runtime.h"
#include <cstdio>
#include <vector>

void print(luna::runtime::Runtime* rt, uint64_t nargs) {
    std::vector<luna::runtime::Value> values;
    for(auto i = 0; i < nargs; i++) {
        values.push_back(rt->pop_last_value());
    }
    for(auto i = nargs; i > 0; i--) {
        auto value = values[i - 1];
        switch (value.type) {    
        case luna::runtime::TypeInt:
            printf("%lld ", value.value_int);
            break;
        case luna::runtime::TypeFloat:
            printf("%f ", value.value_float);
            break;
        case luna::runtime::TypeBool:
            printf("%s ", value.value_boolean ? "true" : "false");
            break;
        case luna::runtime::TypeObject:
            printf("<obj> ");
            break;
        }
    }
    printf("\n");
}

int main(int argc, const char** argv) {
    if (argc == 1) {
        printf("Expected `luna <source file>`\n");
        return 1;
    }
    
    printf("reading source file... "); fflush(stdout);
    auto path = argv[1];
    auto maybe_file = slerp(path);
    if (!maybe_file.has_value()) {
        printf("Could not find source file: %s\n", path);
        return 1;
    }

    luna::Environment env;
    env.add_host_func("print", print);
    
    luna::compiler::Parser parser;
    luna::compiler::Gen gen;
    printf("done\nparsing... "); fflush(stdout);
    auto module = parser.parse_file(std::move(*maybe_file));
    printf("done\ngenerating byte code... "); fflush(stdout);
    auto runtime_module = gen.generate(module, &env);
    printf("done\nstarting runtime... "); fflush(stdout);
    luna::runtime::Runtime runtime(&env);
    printf("done\nexecuting.\n"); fflush(stdout);
    runtime.exec(runtime_module);
    
    return 0;
}
