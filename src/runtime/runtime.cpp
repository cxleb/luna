#include "runtime.h"
#include "runtime/bytecode.h"
#include "runtime/value.h"
#include "shared/environment.h"
#include "shared/error.h"

namespace luna::runtime {

Runtime::Runtime(Environment* env) {
    environment = env;
}

void Runtime::op_result_error(OpResult result, Value a, Value b) {
    error("Value Operation Error\n");
}

#define LOCAL_AT(i) locals[base + i]

void Runtime::exec(ref<Module> module) {
    uint64_t base = 0;

    auto load_function = [&] (uint16_t id) {
        auto function = module->functions[id].get();
        frames.push({
            .code = function->code.data(),
            .ip = 0,
            .locals = function->locals,
        });
        // 32 temps + locals is the size we need
        // locals should already have an offset of 32 so we dont need to add it
        // here
        uint64_t needed_locals = base + function->locals;
        if(locals.size() < needed_locals) {
            locals.resize(needed_locals);
        }
    };
    frames.clear();

    auto main_func_id = module->name_mapping["main"];
    load_function(main_func_id);

    while(true) {
        auto& frame = frames.peek();
        auto inst = frame.code[frame.ip++];
        switch (inst.opcode) {
            case OpcodeBr: {
                frame.ip = inst.s;
                break;
            }
            case OpcodeCondBr: {
                if (value_truthy(LOCAL_AT(inst.a))) {
                    frame.ip = inst.s;
                }
                break;
            }
            case OpcodeCall: {
                load_function(inst.s);
                break;
            }
            case OpcodeCallHost: {
                environment->invoke_function(this, inst.s, inst.a);
                break;
            }
            case OpcodeRet: {
                auto popped_frame = frames.pop();
                if (frames.count() == 0) {
                    return;
                }
                base -= popped_frame.locals;
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
                LOCAL_AT(inst.a] = module->constants[inst.s);
                break;
            }
            //case OpcodeCell: {
            //    stack.push((Cell*)inst.operand_ptr);
            //    break;
            //}
            //case OpcodeInt: {
            //    stack.push((int64_t)inst.operand_int);
            //    break;
            //}
            //case OpcodeFloat: {
            //    stack.push(inst.operand_float);
            //    break;
            //}
        }
    }
}

}