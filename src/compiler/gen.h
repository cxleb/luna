#pragma once
#include "../runtime/bytecode.h"
#include "ast.h"
#include "../shared/environment.h"

namespace luna::compiler {

class Gen {
public:
    ref<runtime::Module> generate(ref<Module> module, Environment* env);
};

}