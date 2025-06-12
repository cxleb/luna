#include "runtime/value.h"
#include "shared/environment.h"
#include "shared/utils.h"

#include "compiler/parser.h"
#include "compiler/gen.h"

#include "runtime/runtime.h"
#include <cstdio>

void print(luna::runtime::Runtime* rt, luna::runtime::Value* args, uint64_t nargs) {
    for(auto i = nargs; i > 0; i--) {
        auto value = args[i - 1];
        luna::runtime::value_print(value);
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
    
    luna::compiler::Parser parser(std::move(*maybe_file));
    luna::compiler::Gen gen;
    printf("done\nparsing... "); fflush(stdout);
    auto module = parser.parse_module();
    if (module.is_error()) {
        printf("Error compiling: %s\n", module.error().msg().c_str());
    }
    printf("done\ngenerating byte code... "); fflush(stdout);
    auto runtime_module = gen.generate(module.value(), &env);
    luna::runtime::dump_module(runtime_module);
    printf("done\nstarting runtime... "); fflush(stdout);
    luna::runtime::Runtime runtime(&env);
    printf("done\nexecuting.\n"); fflush(stdout);
    runtime.exec(runtime_module);

    return 0;
}
