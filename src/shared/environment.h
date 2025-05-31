#pragma once

#include <cstdint>
#include <optional>
#include <unordered_map>
#include <vector>

namespace luna {

// forward declare Runtime
namespace runtime {
    class Runtime;
}

typedef void (*host_function) (runtime::Runtime* rt, uint64_t nargs);

// contains the host functions
class Environment {
public:
    void add_host_func(const std::string& name, host_function func);
    std::optional<uint64_t> get_func_id(const std::string& name);
    void invoke_function(runtime::Runtime* rt, uint64_t id, uint64_t nargs);
private:
    std::unordered_map<std::string, uint64_t> name_mapping;
    std::vector<host_function> host_funcs;
};

}