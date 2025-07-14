#include "builtins.h"
#include "runtime/heap.h"

// void print(luna::runtime::Runtime* rt, luna::runtime::Value* args, uint64_t nargs) {
//     for(auto i = nargs; i > 0; i--) {
//         auto value = args[i - 1];
//         //luna::runtime::value_print(value);
//     }
//     printf("\n");
// }

void print_int(luna::runtime::Runtime* rt, luna::runtime::Value* args, uint64_t nargs) {
    printf("%lld\n", args[0].value_int);
}

void print_number(luna::runtime::Runtime* rt, luna::runtime::Value* args, uint64_t nargs) {
    printf("%f\n", args[0].value_number);
}

void print_string(luna::runtime::Runtime* rt, luna::runtime::Value* args, uint64_t nargs) {
    auto cell = args[0].value_cell;
    auto string = static_cast<luna::runtime::String*>(cell);
    printf("%s\n", string->c_str());
}

void print_bool(luna::runtime::Runtime* rt, luna::runtime::Value* args, uint64_t nargs) {
    if (args[0].value_int == 0) {
        printf("false\n");
    } else {
        printf("true\n");
    }
}

namespace luna {

void load_builtins(Environment* env) {
    //env->add_host_func("print", print);
    env->add_host_func("print_int", print_int);
    env->add_host_func("print_number", print_number);
    env->add_host_func("print_string", print_string);
    env->add_host_func("print_bool", print_bool);

}

}
