#pragma once

#include <cstdint>
#include <optional>
#include <unordered_map>
#include <vector>
#include "../runtime/heap.h"
#include "runtime/value.h"
#include <string>

namespace luna {

// forward declare Runtime
namespace runtime {
    class Runtime;
}

typedef void (*host_function) (runtime::Runtime* rt, runtime::Value* args, uint64_t nargs);

// contains the host functions
class Environment {
public:
    void add_host_func(const std::string& name, host_function func);
    std::optional<uint16_t> get_func_id(const std::string& name);
    void invoke_function(runtime::Runtime* rt, uint16_t id, runtime::Value* args, uint8_t nargs);

    runtime::Heap heap;
private:

    std::unordered_map<std::string, uint16_t> name_mapping;
    std::vector<host_function> host_funcs;
};

}