#include "runtime.h"
#include "runtime/bytecode.h"
#include "runtime/heap.h"
#include "runtime/value.h"
#include "shared/environment.h"
#include "shared/error.h"
#include <stdio.h>
#include <cassert>
#include <cstdint>
#include <cstdio>

namespace luna::runtime {

Runtime::Runtime(Environment* env) {
    environment = env;
}

void Runtime::op_result_error(OpResult result, Value a, Value b) {
    error("Value Operation Error\n");
}

#define LOCAL_AT(i) locals[base + i]
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

    printf("constants %zu\n", module->constants.size());

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
                if (value_falsy(LOCAL_AT(inst.a))) {
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
                auto a = LOCAL_AT(inst.a);
                auto key = LOCAL_AT(inst.b);
                auto eq = LOCAL_AT(inst.c);
                if (a.type == TypeObject) {
                    auto cell = a.value_object;
                    if (cell->kind == Cell::KindObject) {
                        auto obj = static_cast<Object*>(cell);
                        obj->set(key, eq);
                    }
                } 
                break;
            } 
            case OpcodeObjectGet: {
                auto a = LOCAL_AT(inst.b);
                auto key = LOCAL_AT(inst.c);
                auto eq = Value();
                if (a.type == TypeObject) {
                    auto cell = a.value_object;
                    if (cell->kind == Cell::KindObject) {
                        auto obj = static_cast<Object*>(cell);
                        eq = obj->get(key);
                    }
                } 
                LOCAL_AT(inst.a) = eq;
                break;  
            }
            case OpcodeMove: {
                LOCAL_AT(inst.a) = LOCAL_AT(inst.b);
                break;
            }
            case OpcodeAdd: {
                auto a = LOCAL_AT(inst.a);
                auto b = LOCAL_AT(inst.b);

                auto result = value_add(a, b);
                if (result.not_valid) {
                    op_result_error(result, a, b);
                } 
                LOCAL_AT(inst.c) = result.value;

                break;
            }
            case OpcodeSub: {
                auto a = LOCAL_AT(inst.a);
                auto b = LOCAL_AT(inst.b);

                auto result = value_sub(a, b);
                if (result.not_valid) {
                    op_result_error(result, a, b);
                } 
                LOCAL_AT(inst.c) = result.value;

                break;
            }
            case OpcodeMul: {
                auto a = LOCAL_AT(inst.a);
                auto b = LOCAL_AT(inst.b);

                auto result = value_mul(a, b);
                if (result.not_valid) {
                    op_result_error(result, a, b);
                } 
                LOCAL_AT(inst.c) = result.value;

                break;
            }
            case OpcodeDiv: {
                auto a = LOCAL_AT(inst.a);
                auto b = LOCAL_AT(inst.b);

                auto result = value_div(a, b);
                if (result.not_valid) {
                    op_result_error(result, a, b);
                } 
                LOCAL_AT(inst.c) = result.value;

                break;
            }
            case OpcodeEq: {
                auto a = LOCAL_AT(inst.a);
                auto b = LOCAL_AT(inst.b);

                auto result = value_eq(a, b);
                if (result.not_valid) {
                    op_result_error(result, a, b);
                } 
                LOCAL_AT(inst.c) = result.value;

                break;
            }
            case OpcodeNotEq: {
                auto a = LOCAL_AT(inst.a);
                auto b = LOCAL_AT(inst.b);

                auto result = value_neq(a, b);
                if (result.not_valid) {
                    op_result_error(result, a, b);
                } 
                LOCAL_AT(inst.c) = result.value;

                break;
            }
            case OpcodeGr: {
                auto a = LOCAL_AT(inst.a);
                auto b = LOCAL_AT(inst.b);

                auto result = value_gr(a, b);
                if (result.not_valid) {
                    op_result_error(result, a, b);
                } 
                LOCAL_AT(inst.c) = result.value;

                break;
            }
            case OpcodeLess: {
                auto a = LOCAL_AT(inst.a);
                auto b = LOCAL_AT(inst.b);

                auto result = value_less(a, b);
                if (result.not_valid) {
                    op_result_error(result, a, b);
                } 
                LOCAL_AT(inst.c) = result.value;

                break;
            }
            case OpcodeGrEq: {
                auto a = LOCAL_AT(inst.a);
                auto b = LOCAL_AT(inst.b);

                auto result = value_gr_eq(a, b);
                if (result.not_valid) {
                    op_result_error(result, a, b);
                } 
                LOCAL_AT(inst.c) = result.value;

                break;
            }
            case OpcodeLessEq: {
                auto a = LOCAL_AT(inst.a);
                auto b = LOCAL_AT(inst.b);

                auto result = value_less_eq(a, b);
                if (result.not_valid) {
                    op_result_error(result, a, b);
                } 
                LOCAL_AT(inst.c) = result.value;

                break;
            }
            case OpcodeLoadConst: {
                LOCAL_AT(inst.a) = module->constants[inst.s];
                break;
            }
        }
    }
}

}