#pragma once

#include "environment.h"
#include "runtime/bytecode.h"
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
    void create_local(const std::string& name);
    std::optional<uint32_t> get_local_id(const std::string& name);
    // Label management
    uint64_t new_label();
    void mark_label(uint64_t label);
    // Op codes
    void insert(runtime::Inst inst);
    void call(const std::string& function_name, uint64_t nargs);
    void ret();
    void br(uint64_t label);
    void condbr(uint64_t label);
    void store(const std::string& name);
    void load(const std::string& name);
    void add();
    void sub();
    void mul();
    void div();
    void eq();
    void noteq();
    void gr();
    void gr_eq();
    void less();
    void less_eq();
    void int_(uint64_t value);
    void float_(double value);

    ref<runtime::Function> build();
private:
    ModuleBuilder* builder;
    uint64_t label_counter;
    std::vector<uint64_t> labels;
    luna::Stack<std::unordered_map<std::string, uint64_t>> scopes;
    ref<runtime::Function> function;
};

class ModuleBuilder {
public:
    ModuleBuilder(Environment* env);
    FunctionBuilder new_function(const std::string& name);
    void add_function(ref<runtime::Function> function);
    uint64_t get_func_name_id(const std::string& name);
    inline ref<runtime::Module> link();
    inline ref<runtime::Module> get_module() { return module; }
    inline Environment* get_env() { return environment; }
private:
    Environment* environment;
    ref<runtime::Module> module;
};

}