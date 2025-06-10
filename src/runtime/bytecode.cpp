#include "bytecode.h"
#include <cstdio>

namespace luna::runtime {

void dump_inst(Inst inst) {
    switch (inst.opcode) {
    case OpcodeBr: {
        printf("br %u\n", inst.s);
        break;
    }
    case OpcodeCondBr: {
        printf("condbr %u %u\n", inst.a, inst.s);
        break;
    }
    case OpcodeCall: {
        printf("call %u %u\n", inst.s, inst.a);
        break;
    }
    case OpcodeCallHost: {
        printf("call host %u %u\n", inst.s, inst.a);
        break;
    }
    case OpcodeArg: {
        printf("arg %u %u\n", inst.a, inst.b);
        break;
    }
    case OpcodeRetVal: {
        printf("ret val %u\n", inst.a);
        break;
    }
    case OpcodeRet: {
        printf("ret\n");
        break;
    }
    case OpcodeObjectNew: {
        printf("obj new %u\n", inst.a);
        break;
    }
    case OpcodeObjectSet: {
        printf("obj set %u[%u] = %u\n", inst.a, inst.b, inst.c);
        break;
    } 
    case OpcodeObjectGet: {
        printf("obj get %u = %u[%u]\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeMove: {
        printf("move %u %u\n", inst.a, inst.b);
        break;
    }
    case OpcodeAdd: {
        printf("add %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeSub: {
        printf("add %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeMul: {
        printf("mul %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeDiv: {
        printf("add %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeEq: {
        printf("eq %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeNotEq: {
        printf("not eq %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeGr: {
        printf("gr %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeLess: {
        printf("less %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeGrEq: {
        printf("gr eq %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeLessEq: {
        printf("less eq %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeLoadConst: {
        printf("const %u %u\n", inst.a, inst.s);
        break;
    }
    }
}

void dump_module(ref<Module> module) {
    for(auto func: module->functions) {
        printf("Func: %s\n", func->name.c_str());
        for(auto i = 0; i < func->code.size(); i++) {
            printf("[%u] ", i);
            dump_inst(func->code[i]);
        }
    }
}

}