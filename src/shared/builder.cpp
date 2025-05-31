#include "builder.h"
#include "runtime/bytecode.h"
#include "shared/environment.h"
#include "shared/utils.h"
#include <cstdint>
#include <unordered_map>

namespace luna {

FunctionBuilder::FunctionBuilder(const std::string& name, ModuleBuilder* builder) {
    label_counter = 0;
    function = make_ref<runtime::Function>();
    function->name = name;
    this->builder = builder;
}

void FunctionBuilder::push_scope() {
    scopes.push(std::unordered_map<std::string, uint64_t>());
}

void FunctionBuilder::pop_scope() {
    scopes.pop();
}

void FunctionBuilder::create_local(const std::string& name) {
    scopes.peak().insert({
        name,
        function->locals++,
    });
}

std::optional<uint32_t> FunctionBuilder::get_local_id(const std::string& name) {
    for(auto it = scopes.rbegin(); it != scopes.rend(); it += 1) {
        auto& scope = *it;
        if (scope.contains(name)) {
            return (*it)[name];
        }
    }
    return std::nullopt;
}

void FunctionBuilder::insert(runtime::Inst inst) {
    function->code.push_back(inst);
}

uint64_t FunctionBuilder::new_label() {
    auto label = label_counter++;
    labels.resize(label_counter);
    return label;
}

void FunctionBuilder::mark_label(uint64_t label) {
    labels[label] = function->code.size();
}

void FunctionBuilder::call(const std::string& function_name, uint64_t nargs) {
    auto host_func_id = builder->get_env()->get_func_id(function_name);
    if (host_func_id.has_value()) {
        insert({
            .opcode = runtime::OpcodeCallHost,
            .operand_int = *host_func_id | ( nargs << 32),
        });
    } else {
        auto id = builder->get_func_name_id(function_name);
        insert({
            .opcode = runtime::OpcodeCall,
            .operand_int = id | ( nargs << 32),
        });
    }
}

void FunctionBuilder::ret() {
    insert({
        .opcode = runtime::OpcodeRet,
    });
}

void FunctionBuilder::br(uint64_t label) {
    insert({
        .opcode = runtime::OpcodeBr,
        .operand_int = label,
    });
}

void FunctionBuilder::condbr(uint64_t label) {
    insert({
        .opcode = runtime::OpcodeCondBr,
        .operand_int = label,
    });
}

void FunctionBuilder::store(const std::string& name) {
    auto id = get_local_id(name);
    insert({
        .opcode = runtime::OpcodeStore,
        .operand_int = *id,
    });
}

void FunctionBuilder::load(const std::string& name) {
    auto id = get_local_id(name);
    insert({
        .opcode = runtime::OpcodeLoad,
        .operand_int = *id,
    });
}

void FunctionBuilder::add() {
    insert({
        .opcode = runtime::OpcodeAdd,
    });
}

void FunctionBuilder::sub() {
    insert({
        .opcode = runtime::OpcodeSub,
    });
}

void FunctionBuilder::mul() {
    insert({
        .opcode = runtime::OpcodeMul,
    });
}

void FunctionBuilder::div() {
    insert({
        .opcode = runtime::OpcodeDiv,
    });
}

void FunctionBuilder::eq() {
    insert({
        .opcode = runtime::OpcodeEq,
    });
}

void FunctionBuilder::noteq() {
    insert({
        .opcode = runtime::OpcodeNotEq,
    });
}

void FunctionBuilder::gr() {
    insert({
        .opcode = runtime::OpcodeGr,
    });
}

void FunctionBuilder::gr_eq() {
    insert({
        .opcode = runtime::OpcodeGrEq,
    });
}

void FunctionBuilder::less() {
    insert({
        .opcode = runtime::OpcodeLess,
    });
}

void FunctionBuilder::less_eq() {
    insert({
        .opcode = runtime::OpcodeLessEq,
    });
}

void FunctionBuilder::int_(uint64_t value) {
    insert({
        .opcode = runtime::OpcodeInt,
        .operand_int = value,
    });
}

void FunctionBuilder::float_(double value) {
    insert({
        .opcode = runtime::OpcodeFloat,
        .operand_float = value,
    });
}

ref<runtime::Function> FunctionBuilder::build() {
    // if the last bytecode is not a return, add a return so we always return.
    if((*function->code.end()).opcode != runtime::OpcodeRet) {
        insert({
            .opcode = runtime::OpcodeRet,
        });
    }
    // Fix all labels into offsets into the code
    for(auto& inst: function->code) {
        if (inst.opcode == runtime::OpcodeBr 
            || inst.opcode == runtime::OpcodeCondBr) {
            inst.operand_int = labels[inst.operand_int];
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

uint64_t ModuleBuilder::get_func_name_id(const std::string& name) {
    if (module->name_mapping.contains(name)) {
        return module->name_mapping[name];
    }
    auto id = module->functions.size();
    module->name_mapping.insert({name, id});
    module->functions.resize(id + 1);
    return id;
}

ref<runtime::Module> ModuleBuilder::link() {
    return module;
}

}