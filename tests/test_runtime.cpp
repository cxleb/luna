#include "runtime/runtime.h"
#include "runtime/value.h"
#include "shared/builder.h"
#include "shared/environment.h"
#include "testing.h"
#include <cstdio>

using namespace luna::runtime;

#define BASIC_RUNTIME_TEST(block, expected) { \
    Value expected_value(expected); \
    luna::ModuleBuilder module_builder(&env); \
    luna::FunctionBuilder builder = module_builder.new_function("main"); \
    block \
    module_builder.add_function(builder.build()); \
    Runtime runtime(&env); \
    runtime.exec(module_builder.get_module()); \
    auto last = runtime.return_value; \
    printf("ret val = %lld\n", last.value_int); \
    TEST_ASSERT(last.type != expected_value.type); \
    TEST_ASSERT(last.value_int != expected_value.value_int); \
} 

int main(int argc, const char** argv) {

    luna::Environment env;

    BASIC_RUNTIME_TEST({
        auto lhs = builder.alloc_temp();
        auto rhs = builder.alloc_temp();
        auto eq = builder.alloc_temp();
        builder.load_const(lhs, (int64_t)10);
        builder.load_const(rhs, (int64_t)20);
        builder.add(lhs, rhs, eq);
        builder.ret(eq);
    }, (int64_t)30);

    BASIC_RUNTIME_TEST({
        auto lhs = builder.alloc_temp();
        auto rhs = builder.alloc_temp();
        auto eq = builder.alloc_temp();
        builder.load_const(lhs, (int64_t)10);
        builder.load_const(rhs, (int64_t)10);
        builder.eq(lhs, rhs, eq);
        builder.ret(eq);
    }, true);

    BASIC_RUNTIME_TEST({
        auto lhs = builder.alloc_temp();
        auto rhs = builder.alloc_temp();
        auto eq = builder.alloc_temp();
        builder.load_const(lhs, (int64_t)10);
        builder.load_const(rhs, (int64_t)10);
        builder.noteq(lhs, rhs, eq);
        builder.ret(eq);
    }, false);

    BASIC_RUNTIME_TEST({
        auto lhs = builder.alloc_temp();
        auto rhs = builder.alloc_temp();

        // does 2 adds, but should skip the first add only add up to 20 
        builder.load_const(lhs, (int64_t)10);
        auto label = builder.new_label();
        builder.br(label);
        builder.load_const(rhs, (int64_t)10);
        builder.add(lhs, rhs, lhs);
        builder.mark_label(label);
        builder.load_const(rhs, (int64_t)10);
        builder.add(lhs, rhs, lhs);
        builder.ret(lhs);
    }, (int64_t)20);

    BASIC_RUNTIME_TEST({
        auto lhs = builder.alloc_temp();
        auto rhs = builder.alloc_temp();
        
        // does 2 adds, but should skip the first add only add up to 20 
        // this time using a condbr
        builder.load_const(lhs, (int64_t)10);
        auto label = builder.new_label();
        builder.load_const(rhs, true);
        builder.condbr(rhs, label);
        builder.load_const(rhs, (int64_t)10);
        builder.add(lhs, rhs, lhs);
        builder.mark_label(label);
        builder.load_const(rhs, (int64_t)10);
        builder.add(lhs, rhs, lhs);
        builder.ret(lhs);
    }, (int64_t)20);

    BASIC_RUNTIME_TEST({
        auto lhs = builder.alloc_temp();
        auto rhs = builder.alloc_temp();
        
        // does 2 adds, but should not skip the first add add up to 30
        // tests a condbr can fail
        builder.load_const(lhs, (int64_t)10);
        auto label = builder.new_label();
        builder.load_const(rhs, false);
        builder.condbr(rhs, label);
        builder.load_const(rhs, (int64_t)10);
        builder.add(lhs, rhs, lhs);
        builder.mark_label(label);
        builder.load_const(rhs, (int64_t)10);
        builder.add(lhs, rhs, lhs);
        builder.ret(lhs);
    }, (int64_t)30);

    return 0;
}