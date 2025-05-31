#include "runtime.h"
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

void Runtime::exec(ref<Module> module) {
    uint64_t base = 0;

    auto load_function = [&] (uint64_t id) {
        auto function = module->functions[id].get();
        frames.push({
            .code = function->code.data(),
            .ip = 0,
            .locals = function->locals,
        });
        if(locals.size() < base + function->locals) {
            locals.resize(base + function->locals);
        }
    };
    frames.clear();

    auto main_func_id = module->name_mapping["main"];
    load_function(main_func_id);

    while(true) {
        auto& frame = frames.peak();
        auto inst = frame.code[frame.ip++];
        switch (inst.opcode) {
            case OpcodeBr: {
                frame.ip = inst.operand_int;
                break;
            }
            case OpcodeCondBr: {
                auto a = stack.pop();
                if (value_truthy(a)) {
                    frame.ip = inst.operand_int;
                }
                break;
            }
            case OpcodeCall: {
                load_function(inst.operand_int);
                break;
            }
            case OpcodeCallHost: {
                auto id = inst.operand_int & 0xFFFFFFFF;
                auto nargs = inst.operand_int >> 32;
                environment->invoke_function(this, id, nargs);
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
            case OpcodeStore: {
                locals[base + inst.operand_int] = stack.pop();
                break;
            }
            case OpcodeLoad: {
                stack.push(locals[base + inst.operand_int]);
                break;
            }
            case OpcodeAdd: {
                auto b = stack.pop();
                auto a = stack.pop();

                auto result = value_add(a, b);
                if (result.not_valid) {
                    op_result_error(result, a, b);
                } 
                stack.push(result.value);

                break;
            }
            case OpcodeSub: {
                auto b = stack.pop();
                auto a = stack.pop();

                auto result = value_sub(a, b);
                if (result.not_valid) {
                    op_result_error(result, a, b);
                } 
                stack.push(result.value);

                break;
            }
            case OpcodeMul: {
                auto b = stack.pop();
                auto a = stack.pop();

                auto result = value_mul(a, b);
                if (result.not_valid) {
                    op_result_error(result, a, b);
                } 
                stack.push(result.value);

                break;
            }
            case OpcodeDiv: {
                auto b = stack.pop();
                auto a = stack.pop();

                auto result = value_div(a, b);
                if (result.not_valid) {
                    op_result_error(result, a, b);
                } 
                stack.push(result.value);

                break;
            }
            case OpcodeEq: {
                auto b = stack.pop();
                auto a = stack.pop();

                auto result = value_eq(a, b);
                if (result.not_valid) {
                    op_result_error(result, a, b);
                } 
                stack.push(result.value);

                break;
            }
            case OpcodeNotEq: {
                auto b = stack.pop();
                auto a = stack.pop();

                auto result = value_neq(a, b);
                if (result.not_valid) {
                    op_result_error(result, a, b);
                } 
                stack.push(result.value);

                break;
            }
            case OpcodeGr: {
                auto b = stack.pop();
                auto a = stack.pop();

                auto result = value_gr(a, b);
                if (result.not_valid) {
                    op_result_error(result, a, b);
                } 
                stack.push(result.value);

                break;
            }
            case OpcodeLess: {
                auto b = stack.pop();
                auto a = stack.pop();

                auto result = value_less(a, b);
                if (result.not_valid) {
                    op_result_error(result, a, b);
                } 
                stack.push(result.value);

                break;
            }
            case OpcodeGrEq: {
                auto b = stack.pop();
                auto a = stack.pop();

                auto result = value_gr_eq(a, b);
                if (result.not_valid) {
                    op_result_error(result, a, b);
                } 
                stack.push(result.value);

                break;
            }
            case OpcodeLessEq: {
                auto b = stack.pop();
                auto a = stack.pop();

                auto result = value_less_eq(a, b);
                if (result.not_valid) {
                    op_result_error(result, a, b);
                } 
                stack.push(result.value);

                break;
            }
            case OpcodeInt: {
                stack.push((int64_t)inst.operand_int);
                break;
            }
            case OpcodeFloat: {
                stack.push(inst.operand_float);
                break;
            }
        }
    }
}

}