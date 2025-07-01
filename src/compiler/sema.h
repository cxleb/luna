#pragma once
#include "ast.h"
#include "../shared/environment.h"

namespace luna::compiler {

class Sema {
public:
    std::optional<Error> check(ref<Module> module, Environment* env);
};

}