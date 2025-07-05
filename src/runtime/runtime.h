#pragma once

#include <cstdint>
#include "../shared/utils.h"
#include "../shared/stack.h"
#include "bytecode.h"
#include "value.h"
#include "../shared/environment.h"

namespace luna::runtime {

struct Frame {
    Inst* code;
    uint64_t ip;
    uint64_t locals;
    uint64_t prev_base;
    uint8_t ret;
};

class Runtime {
public:
    Runtime(Environment* env);

    void exec(ref<Module> module);
    
    bool value_equal(Value a, Value b);
    void op_result_error(OpResult result, Value a, Value b);
    
    Value return_value;
private:
    Environment* environment;
    std::vector<Value> locals;
    //luna::Stack<Value> stack;
    luna::Stack<Frame> frames;
    uint64_t current_frame;
};

}