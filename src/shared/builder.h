#pragma once

#include "environment.h"
#include "runtime/bytecode.h"
#include "runtime/value.h"
#include "shared/stack.h"
#include <cstdint>
#include <unordered_map>
#include <vector>

namespace luna {

class ModuleBuilder;

class FunctionBuilder {
public:
    FunctionBuilder(const std::string& name, ModuleBuilder* builder);
    // Variables
    void push_scope();
    void pop_scope();
    uint8_t create_local(const std::string& name);
    std::optional<uint8_t> get_local_id(const std::string& name);
    uint8_t alloc_temp();
    void free_temp_if_not_used(uint8_t, uint8_t);
    void free_temp(uint8_t);
    // Label management
    uint16_t new_label();
    void mark_label(uint16_t label);
    // Op codes
    void insert(runtime::Inst inst);
    void arg(uint8_t arg, uint8_t reg);
    void call(const std::string& function_name, uint8_t nargs, uint8_t ret);
    void ret();
    void ret(uint8_t ret);
    void br(uint16_t label);
    void condbr(uint8_t reg, uint16_t label);
    void move(uint8_t a, uint8_t b);
    void store(uint8_t reg, const std::string& name);
    void load(uint8_t reg, const std::string& name);
    void object_new(uint8_t reg);
    void object_set(uint8_t reg, uint8_t idx, uint8_t eq);
    void object_get(uint8_t reg, uint8_t idx, uint8_t eq);
    void add_n(uint8_t lhs, uint8_t rhs, uint8_t eq);
    void sub_n(uint8_t lhs, uint8_t rhs, uint8_t eq);
    void mul_n(uint8_t lhs, uint8_t rhs, uint8_t eq);
    void div_n(uint8_t lhs, uint8_t rhs, uint8_t eq);
    void eq_n(uint8_t lhs, uint8_t rhs, uint8_t eq);
    void noteq_n(uint8_t lhs, uint8_t rhs, uint8_t eq);
    void gr_n(uint8_t lhs, uint8_t rhs, uint8_t eq);
    void gr_eq_n(uint8_t lhs, uint8_t rhs, uint8_t eq);
    void less_n(uint8_t lhs, uint8_t rhs, uint8_t eq);
    void less_eq_n(uint8_t lhs, uint8_t rhs, uint8_t eq);
    void add_i(uint8_t lhs, uint8_t rhs, uint8_t eq);
    void sub_i(uint8_t lhs, uint8_t rhs, uint8_t eq);
    void mul_i(uint8_t lhs, uint8_t rhs, uint8_t eq);
    void div_i(uint8_t lhs, uint8_t rhs, uint8_t eq);
    void eq_i(uint8_t lhs, uint8_t rhs, uint8_t eq);
    void noteq_i(uint8_t lhs, uint8_t rhs, uint8_t eq);
    void gr_i(uint8_t lhs, uint8_t rhs, uint8_t eq);
    void gr_eq_i(uint8_t lhs, uint8_t rhs, uint8_t eq);
    void less_i(uint8_t lhs, uint8_t rhs, uint8_t eq);
    void less_eq_i(uint8_t lhs, uint8_t rhs, uint8_t eq);
    void load_const(uint8_t reg, runtime::Value val);
    //void int_(uint64_t value);
    //void float_(double value);
    //void cell(runtime::Cell* cell);

    ref<runtime::Function> build();
private:
    ModuleBuilder* builder;
    uint16_t label_counter;
    std::vector<uint16_t> labels;
    std::unordered_map<uint8_t, bool> temporaries;
    luna::Stack<std::unordered_map<std::string, uint8_t>> scopes;
    ref<runtime::Function> function;
};

class ModuleBuilder {
public:
    ModuleBuilder(Environment* env);
    FunctionBuilder new_function(const std::string& name);
    void add_function(ref<runtime::Function> function);
    uint16_t get_func_name_id(const std::string& name);
    uint16_t push_constant(runtime::Value value);
    inline ref<runtime::Module> link();
    inline ref<runtime::Module> get_module() { return module; }
    inline Environment* get_env() { return environment; }
private:
    Environment* environment;
    ref<runtime::Module> module;
};

}