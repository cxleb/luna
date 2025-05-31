#include "runtime/runtime.h"
#include "shared/builder.h"
#include "shared/environment.h"
#include "testing.h"

using namespace luna::runtime;

#define BASIC_RUNTIME_TEST(block, expected) { \
    Value expected_value(expected); \
    luna::ModuleBuilder module_builder(&env); \
    luna::FunctionBuilder builder = module_builder.new_function("main"); \
    block \
    module_builder.add_function(builder.build()); \
    Runtime runtime(&env); \
    runtime.exec(module_builder.get_module()); \
    auto last = runtime.pop_last_value(); \
    TEST_ASSERT(last.type != expected_value.type); \
    TEST_ASSERT(last.value_int != expected_value.value_int); \
} 

int main(int argc, const char** argv) {

    luna::Environment env;

    BASIC_RUNTIME_TEST({
        builder.int_(10);
        builder.int_(20);
        builder.add();
        builder.ret();
    }, (int64_t)30);

    BASIC_RUNTIME_TEST({
        builder.int_(10);
        builder.int_(10);
        builder.eq();
        builder.ret();
    }, true);

    BASIC_RUNTIME_TEST({
        builder.int_(10);
        builder.int_(10);
        builder.noteq();
        builder.ret();
    }, false);

    BASIC_RUNTIME_TEST({
        // does 2 adds, but should skip the first add only add up to 20 
        builder.int_(10);
        auto label = builder.new_label();
        builder.br(label);
        builder.int_(10);
        builder.add();
        builder.mark_label(label);
        builder.int_(10);
        builder.add();
        builder.ret();
    }, (int64_t)20);

    BASIC_RUNTIME_TEST({
        // does 2 adds, but should skip the first add only add up to 20 
        // this time using a condbr
        builder.int_(10);
        auto label = builder.new_label();
        builder.int_(1);
        builder.condbr(label);
        builder.int_(10);
        builder.add();
        builder.mark_label(label);
        builder.int_(10);
        builder.add();
        builder.ret();
    }, (int64_t)20);

    BASIC_RUNTIME_TEST({
        // does 2 adds, but should not skip the first add add up to 30
        // tests a condbr can fail
        builder.int_(10);
        auto label = builder.new_label();
        builder.int_(0);
        builder.condbr(label);
        builder.int_(10);
        builder.add();
        builder.mark_label(label);
        builder.int_(10);
        builder.add();
        builder.ret();
    }, (int64_t)30);

    return 0;
}