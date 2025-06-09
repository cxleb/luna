#include "environment.h"
#include <optional>

namespace luna {
    
void Environment::add_host_func(const std::string& name, host_function func) {
    uint64_t id = host_funcs.size();
    name_mapping.insert({name, id});
    host_funcs.push_back(func);
}

std::optional<uint16_t> Environment::get_func_id(const std::string& name) {
    if (!name_mapping.contains(name)) {
        return std::nullopt;
    }
    return name_mapping[name];
}

void Environment::invoke_function(runtime::Runtime* rt, uint16_t id, uint8_t nargs){
    host_funcs[id](rt, nargs);
}

}
