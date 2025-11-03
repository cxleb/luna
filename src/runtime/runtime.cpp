#include "runtime.h"
#include "runtime/bytecode.h"
#include "runtime/heap.h"
#include "runtime/value.h"
#include "shared/environment.h"
#include "shared/error.h"
#include <stdio.h>
#include <cassert>
#include <cstdint>

namespace luna::runtime {

Runtime::Runtime(Environment* env) {
    environment = env;
}

void Runtime::op_result_error(OpResult result, Value a, Value b) {
    error("Value Operation Error\n");
}

#define LOCAL_AT(i) locals[base + i]
#define LOCAL_AT_NUMBER(i) locals[base + i].value_number
#define LOCAL_AT_INT(i) locals[base + i].value_int
#define LOCAL_AT_CELL(i) locals[base + i].value_cell
//#define LOCAL_AT(i) ({assert(base + i < top); locals[base + i];})

void Runtime::exec(ref<Module> module) {
    //using namespace std::chrono_literals;

    uint64_t base = 0;
    uint64_t top = 0;

    auto load_function = [&] (uint16_t id, uint8_t ret) {
        auto function = module->functions[id].get();
        frames.push({
            .code = function->code.data(),
            .ip = 0,
            .locals = function->locals,
            .prev_base = base,
            .ret = ret
        });
        uint64_t needed_locals = top + function->locals;
        if(locals.size() < needed_locals) {
            locals.resize(needed_locals);
        }
        base = top;
        top = needed_locals;
    };

    frames.clear();

    auto main_func_id = module->name_mapping["main"];
    load_function(main_func_id, 0);

    //printf("constants %zu\n", module->constants.size());

    while(true) {
        auto& frame = frames.peek();
        //printf("%llx [%llu] ", (uint64_t)frame.code, frame.ip);
        auto inst = frame.code[frame.ip++];
        //dump_inst(inst);
        switch (inst.opcode) {
            case OpcodeBr: {
                frame.ip = inst.s;
                break;
            }
            case OpcodeCondBr: {
                if (LOCAL_AT_INT(inst.a) == 0) {
                    frame.ip = inst.s;
                }
                break;
            }
            case OpcodeCall: {
                load_function(inst.s, inst.a);
                break;
            }
            case OpcodeCallHost: {
                environment->invoke_function(this, inst.s, &locals[top], inst.a);
                break;
            }
            case OpcodeArg: {
                auto needed_locals = top + inst.a + 1;
                if(locals.size() < needed_locals) {
                    locals.resize(needed_locals);
                }
                locals[top + inst.a] = LOCAL_AT(inst.b);
                break;
            }
            case OpcodeRetVal: {
                return_value = LOCAL_AT(inst.a);
                auto popped_frame = frames.pop();
                if (frames.count() == 0) {
                    return;
                }
                base = popped_frame.prev_base;
                LOCAL_AT(popped_frame.ret) = return_value;
                break;
            }
            case OpcodeRet: {
                auto popped_frame = frames.pop();
                if (frames.count() == 0) {
                    return;
                }
                base = popped_frame.prev_base;
                break;
            }
            case OpcodeObjectNew: {
                LOCAL_AT(inst.a) = environment->heap.alloc_object();
                break;
            }
            case OpcodeObjectSet: {
                auto a = LOCAL_AT_CELL(inst.a);
                auto key = LOCAL_AT(inst.b);
                auto eq = LOCAL_AT(inst.c);
                auto obj = static_cast<Object*>(a);
                obj->set(key.value_int, eq);
                break;
            } 
            case OpcodeObjectGet: {
                auto a = LOCAL_AT_CELL(inst.b);
                auto key = LOCAL_AT(inst.c);
                auto obj = static_cast<Object*>(a);
                LOCAL_AT(inst.a) = obj->get(key.value_int);
                break;  
            }
            case OpcodeMove: {
                LOCAL_AT(inst.a) = LOCAL_AT(inst.b);
                break;
            }
            case OpcodeNumberAdd: {
                auto a = LOCAL_AT_NUMBER(inst.a);
                auto b = LOCAL_AT_NUMBER(inst.b);
                LOCAL_AT(inst.c) = a + b;
                break;
            }
            case OpcodeNumberSub: {
                auto a = LOCAL_AT_NUMBER(inst.a);
                auto b = LOCAL_AT_NUMBER(inst.b);
                LOCAL_AT(inst.c) = a - b;
                break;
            }
            case OpcodeNumberMul: {
                auto a = LOCAL_AT_NUMBER(inst.a);
                auto b = LOCAL_AT_NUMBER(inst.b);
                LOCAL_AT(inst.c) = a * b;
                break;
            }
            case OpcodeNumberDiv: {
                auto a = LOCAL_AT_NUMBER(inst.a);
                auto b = LOCAL_AT_NUMBER(inst.b);
                LOCAL_AT(inst.c) = a / b;
                break;
            }
            case OpcodeNumberEq: {
                auto a = LOCAL_AT_NUMBER(inst.a);
                auto b = LOCAL_AT_NUMBER(inst.b);
                LOCAL_AT(inst.c) = a == b;
                break;
            }
            case OpcodeNumberNotEq: {
                auto a = LOCAL_AT_NUMBER(inst.a);
                auto b = LOCAL_AT_NUMBER(inst.b);
                LOCAL_AT(inst.c) = a != b;
                break;
            }
            case OpcodeNumberGr: {
                auto a = LOCAL_AT_NUMBER(inst.a);
                auto b = LOCAL_AT_NUMBER(inst.b);
                LOCAL_AT(inst.c) = a > b;
                break;
            }
            case OpcodeNumberLess: {
                auto a = LOCAL_AT_NUMBER(inst.a);
                auto b = LOCAL_AT_NUMBER(inst.b);
                LOCAL_AT(inst.c) = a < b;
                break;
            }
            case OpcodeNumberGrEq: {
                auto a = LOCAL_AT_NUMBER(inst.a);
                auto b = LOCAL_AT_NUMBER(inst.b);
                LOCAL_AT(inst.c) = a >= b;
                break;
            }
            case OpcodeNumberLessEq: {
                auto a = LOCAL_AT_NUMBER(inst.a);
                auto b = LOCAL_AT_NUMBER(inst.b);
                LOCAL_AT(inst.c) = a <= b;
                break;
            }
            case OpcodeIntAdd: {
                auto a = LOCAL_AT_INT(inst.a);
                auto b = LOCAL_AT_INT(inst.b);
                LOCAL_AT(inst.c) = a + b;
                break;
            }
            case OpcodeIntSub: {
                auto a = LOCAL_AT_INT(inst.a);
                auto b = LOCAL_AT_INT(inst.b);
                LOCAL_AT(inst.c) = a - b;
                break;
            }
            case OpcodeIntMul: {
                auto a = LOCAL_AT_INT(inst.a);
                auto b = LOCAL_AT_INT(inst.b);
                LOCAL_AT(inst.c) = a * b;
                break;
            }
            case OpcodeIntDiv: {
                auto a = LOCAL_AT_INT(inst.a);
                auto b = LOCAL_AT_INT(inst.b);
                LOCAL_AT(inst.c) = a / b;
                break;
            }
            case OpcodeIntEq: {
                auto a = LOCAL_AT_INT(inst.a);
                auto b = LOCAL_AT_INT(inst.b);
                LOCAL_AT(inst.c) = a == b;
                break;
            }
            case OpcodeIntNotEq: {
                auto a = LOCAL_AT_INT(inst.a);
                auto b = LOCAL_AT_INT(inst.b);
                LOCAL_AT(inst.c) = a != b;
                break;
            }
            case OpcodeIntGr: {
                auto a = LOCAL_AT_INT(inst.a);
                auto b = LOCAL_AT_INT(inst.b);
                LOCAL_AT(inst.c) = a > b;
                break;
            }
            case OpcodeIntGrEq: {
                auto a = LOCAL_AT_INT(inst.a);
                auto b = LOCAL_AT_INT(inst.b);
                LOCAL_AT(inst.c) = a >= b;
                break;
            }
            case OpcodeIntLess: {
                auto a = LOCAL_AT_INT(inst.a);
                auto b = LOCAL_AT_INT(inst.b);
                LOCAL_AT(inst.c) = a < b;
                break;
            }
            case OpcodeIntLessEq: {
                auto a = LOCAL_AT_INT(inst.a);
                auto b = LOCAL_AT_INT(inst.b);
                LOCAL_AT(inst.c) = a <= b;
                break;
            }
            case OpcodeLoadConst: {
                LOCAL_AT(inst.a) = module->constants[inst.s].value;
                break;
            }
        }
    }
}

}