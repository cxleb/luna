
#include "compiler/parser.h"
#include "compiler/gen.h"

#include "runtime/bytecode.h"
#include "runtime/runtime.h"
#include "runtime/value.h"


void print(luna::runtime::Runtime* rt, luna::runtime::Value* args, uint64_t nargs) {
    for(auto i = nargs; i > 0; i--) {
        auto value = args[i - 1];
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

void _assert(luna::runtime::Runtime* rt, luna::runtime::Value* args, uint64_t nargs) {
    auto value = args[0];
    if (value.type != luna::runtime::TypeBool) {
        printf("Expected bool value but got %s\n", 
            luna::runtime::get_name_for_type(value.type));
        exit(1);
    }
    if (!value.value_boolean) {
        printf("Assert failed\n");
        exit(1);
    }
}

int main(int argc, const char** argv) {
    auto maybe_file = slerp("../tests/test_self.luna");
    if (!maybe_file.has_value()) {
        printf("Could not load self test file\n");
        return 1;
    }

    luna::Environment env;
    env.add_host_func("print", print);
    env.add_host_func("assert", _assert);
    
    luna::compiler::Parser parser(std::move(*maybe_file));
    luna::compiler::Gen gen;
    auto module = parser.parse_module();
    if (module.is_error()) {
        printf("Error compiling: %s\n", module.error().msg().c_str());
        return 1;
    }
    auto runtime_module = gen.generate(module.value(), &env);
    luna::runtime::Runtime runtime(&env);
    luna::runtime::dump_module(runtime_module);
    runtime.exec(runtime_module);
    
    return 0;
}