#include "builder.h"
#include "runtime/bytecode.h"
#include "shared/environment.h"
#include "shared/utils.h"
#include <cstdint>
#include <optional>
#include <unordered_map>

namespace luna {

FunctionBuilder::FunctionBuilder(const std::string& name, ModuleBuilder* builder) {
    label_counter = 0;
    function = make_ref<runtime::Function>();
    function->name = name;
    function->locals = 0; // allocate 32 temporary values
    //for(auto i = 0; i < 32; i++) {
    //    temporaries[i] = false;
    //}
    this->builder = builder;
}

void FunctionBuilder::push_scope() {
    scopes.push(std::unordered_map<std::string, uint64_t>());
}

void FunctionBuilder::pop_scope() {
    scopes.pop();
}

uint8_t FunctionBuilder::create_local(const std::string& name) {
    uint8_t idx = function->locals++;
    scopes.peek().insert({
        name,
        idx,
    });
    return idx;
}

std::optional<uint8_t> FunctionBuilder::get_local_id(const std::string& name) {
    for(auto it = scopes.rbegin(); it != scopes.rend(); it += 1) {
        auto& scope = *it;
        if (scope.contains(name)) {
            return (*it)[name];
        }
    }
    return std::nullopt;
}

uint8_t FunctionBuilder::alloc_temp() {
    // find first free temporary
    for (auto& temp: temporaries) {
        if(temp.second == false) {
            temp.second = true;
            return temp.first;
        }
    }
    // if we didnt get one then create one
    uint8_t idx = function->locals++;
    temporaries.insert({
        idx,
        true,
    });
    return idx;

}

void FunctionBuilder::free_temp_if_not_used(uint8_t temp, uint8_t local) {
    if(temp != local) {
        free_temp(temp);
    }
}

void FunctionBuilder::free_temp(uint8_t id) {
    if (temporaries.contains(id)) {
        temporaries[id] = false;
    }
}

void FunctionBuilder::insert(runtime::Inst inst) {
    function->code.push_back(inst);
}

uint16_t FunctionBuilder::new_label() {
    auto label = label_counter++;
    labels.resize(label_counter);
    return label;
}

void FunctionBuilder::mark_label(uint16_t label) {
    labels[label] = function->code.size();
}

void FunctionBuilder::arg(uint8_t arg, uint8_t reg) {
    insert({
        .opcode = runtime::OpcodeArg,
        .a = arg,
        .b = reg,
    });
}

void FunctionBuilder::call(const std::string& function_name, uint8_t nargs, uint8_t ret) {
    auto host_func_id = builder->get_env()->get_func_id(function_name);
    if (host_func_id.has_value()) {
        insert({
            .opcode = runtime::OpcodeCallHost,
            .a = nargs,
            .s = *host_func_id,// | ( nargs << 32),
        });
    } else {
        auto id = builder->get_func_name_id(function_name);
        insert({
            .opcode = runtime::OpcodeCall,
            .a = ret,
            .s = id,// | ( nargs << 32),
        });
    }
}

void FunctionBuilder::ret() {
    insert({
        .opcode = runtime::OpcodeRet,
    });
}

void FunctionBuilder::ret(uint8_t ret) {
    insert({
        .opcode = runtime::OpcodeRetVal,
        .a = ret,
    });
}

void FunctionBuilder::br(uint16_t label) {
    insert({
        .opcode = runtime::OpcodeBr,
        .s = label,
    });
}

void FunctionBuilder::condbr(uint8_t reg, uint16_t label) {
    insert({
        .opcode = runtime::OpcodeCondBr,
        .a = reg,
        .s = label,
    });
}

void FunctionBuilder::move(uint8_t a, uint8_t b) {
    insert({
        .opcode = runtime::OpcodeMove,
        .a = a,
        .b = b,
    });
}


void FunctionBuilder::store(uint8_t reg, const std::string& name) {
    auto id = get_local_id(name);
    insert({
        .opcode = runtime::OpcodeMove,
        .a = *id,
        .b = reg,
    });
}

void FunctionBuilder::load(uint8_t reg, const std::string& name) {
    auto id = get_local_id(name);
    insert({
        .opcode = runtime::OpcodeMove,
        .a = reg,
        .b = *id,
    });
}

void FunctionBuilder::object_new(uint8_t a) {
    insert({
        .opcode = runtime::OpcodeObjectNew,
        .a = a
    });
}

void FunctionBuilder::object_set(uint8_t reg, uint8_t idx, uint8_t eq) {
    insert({
        .opcode = runtime::OpcodeObjectSet,
        .a = reg,
        .b = idx,
        .c = eq
    });
}

void FunctionBuilder::object_get(uint8_t reg, uint8_t idx, uint8_t eq) {
    insert({
        .opcode = runtime::OpcodeObjectGet,
        .a = eq,
        .b = reg,
        .c = idx
    });
}

void FunctionBuilder::add(uint8_t lhs, uint8_t rhs, uint8_t eq) {
    insert({
        .opcode = runtime::OpcodeAdd,
        .a = lhs,
        .b = rhs,
        .c = eq
    });
}

void FunctionBuilder::sub(uint8_t lhs, uint8_t rhs, uint8_t eq) {
    insert({
        .opcode = runtime::OpcodeSub,
        .a = lhs,
        .b = rhs,
        .c = eq
    });
}

void FunctionBuilder::mul(uint8_t lhs, uint8_t rhs, uint8_t eq) {
    insert({
        .opcode = runtime::OpcodeMul,
        .a = lhs,
        .b = rhs,
        .c = eq
    });
}

void FunctionBuilder::div(uint8_t lhs, uint8_t rhs, uint8_t eq) {
    insert({
        .opcode = runtime::OpcodeDiv,
        .a = lhs,
        .b = rhs,
        .c = eq
    });
}

void FunctionBuilder::eq(uint8_t lhs, uint8_t rhs, uint8_t eq) {
    insert({
        .opcode = runtime::OpcodeEq,
        .a = lhs,
        .b = rhs,
        .c = eq
    });
}

void FunctionBuilder::noteq(uint8_t lhs, uint8_t rhs, uint8_t eq) {
    insert({
        .opcode = runtime::OpcodeNotEq,
        .a = lhs,
        .b = rhs,
        .c = eq
    });
}

void FunctionBuilder::gr(uint8_t lhs, uint8_t rhs, uint8_t eq) {
    insert({
        .opcode = runtime::OpcodeGr,
        .a = lhs,
        .b = rhs,
        .c = eq
    });
}

void FunctionBuilder::gr_eq(uint8_t lhs, uint8_t rhs, uint8_t eq) {
    insert({
        .opcode = runtime::OpcodeGrEq,
        .a = lhs,
        .b = rhs,
        .c = eq
    });
}

void FunctionBuilder::less(uint8_t lhs, uint8_t rhs, uint8_t eq) {
    insert({
        .opcode = runtime::OpcodeLess,
        .a = lhs,
        .b = rhs,
        .c = eq
    });
}

void FunctionBuilder::less_eq(uint8_t lhs, uint8_t rhs, uint8_t eq) {
    insert({
        .opcode = runtime::OpcodeLessEq,
        .a = lhs,
        .b = rhs,
        .c = eq
    });
}

void FunctionBuilder::load_const(uint8_t reg, runtime::Value value) {
    auto idx = builder->push_constant(value);
    insert({
        .opcode = runtime::OpcodeLoadConst,
        .a = reg,
        .s = idx,
    });
}

//void FunctionBuilder::int_(uint64_t value) {
//    insert({
//        .opcode = runtime::OpcodeInt,
//        .operand_int = value,
//    });
//}
//
//void FunctionBuilder::float_(double value) {
//    insert({
//        .opcode = runtime::OpcodeFloat,
//        .operand_float = value,
//    });
//}
//
//void FunctionBuilder::cell(runtime::Cell* cell) {
//    insert({
//        .opcode = runtime::OpcodeCell,
//        .operand_ptr = cell,
//    });
//}


ref<runtime::Function> FunctionBuilder::build() {
    // if the last bytecode is not a return, add a return so we always return.
    auto& code = function->code;
    if(code[code.size() - 1].opcode != runtime::OpcodeRet) {
        insert({
            .opcode = runtime::OpcodeRet,
        });
    }
    // Fix all labels into offsets into the code
    for(auto& inst: function->code) {
        if (inst.opcode == runtime::OpcodeBr 
            || inst.opcode == runtime::OpcodeCondBr) {
            inst.s = labels[inst.s];
        }
    }

    return function;
}

ModuleBuilder::ModuleBuilder(Environment* env) {
    module = make_ref<runtime::Module>();
    environment = env;
}

FunctionBuilder ModuleBuilder::new_function(const std::string& name) {
    get_func_name_id(name);
    return FunctionBuilder(name, this);
}

void ModuleBuilder::add_function(ref<runtime::Function> function) {
    auto id = get_func_name_id(function->name);
    module->functions[id] = function;
}

uint16_t ModuleBuilder::get_func_name_id(const std::string& name) {
    if (module->name_mapping.contains(name)) {
        return module->name_mapping[name];
    }
    auto id = module->functions.size();
    module->name_mapping.insert({name, id});
    module->functions.resize(id + 1);
    return id;
}

uint16_t ModuleBuilder::push_constant(runtime::Value value) {
    uint16_t i = 0;
    for (auto& c: module->constants) {
        if (value == c) {
            return i;
        }
        ++i;
    }
    uint16_t idx = module->constants.size();
    module->constants.push_back(value);
    return idx;
}


ref<runtime::Module> ModuleBuilder::link() {
    return module;
}

}