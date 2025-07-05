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
    case OpcodeNumberAdd: {
        printf("add.n %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeNumberSub: {
        printf("sub.n %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeNumberMul: {
        printf("mul.n %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeNumberDiv: {
        printf("div.n %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeNumberEq: {
        printf("eq.n %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeNumberNotEq: {
        printf("noteq.n %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeNumberGr: {
        printf("gr.n %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeNumberLess: {
        printf("less.n %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeNumberGrEq: {
        printf("greq.n %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeNumberLessEq: {
        printf("lesseq.n %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeIntAdd: {
        printf("add.i %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeIntSub: {
        printf("sub.i %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeIntMul: {
        printf("mul.i %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeIntDiv: {
        printf("div.i %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeIntEq: {
        printf("eq.i %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeIntNotEq: {
        printf("noteq.i %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeIntGr: {
        printf("gr.i %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeIntLess: {
        printf("less.i %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeIntGrEq: {
        printf("greq.i %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeIntLessEq: {
        printf("lesseq.i %u %u %u\n", inst.a, inst.b, inst.c);
        break;
    }
    case OpcodeConvert: {
        printf("conv %u %u\n", inst.a, inst.b);
        break;
    }
    case OpcodeTruncate: {
        printf("trunc %u %u\n", inst.a, inst.b);
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