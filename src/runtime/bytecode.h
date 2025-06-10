#pragma once

#include "../shared/utils.h"
#include "runtime/value.h"

#include <cstdint>
#include <vector>
#include <unordered_map>
#include <string>

namespace luna::runtime {

// Instructions are 32 bit have eiter 3 8-bit operands or 1 8-bit and 1 16-bit
enum Opcode {
    // Control Flow
    OpcodeBr,
    OpcodeCondBr,
    // calls have the lower 32 bits as the id and the upper 32 bits as the 
    // number of arguements
    OpcodeArg, // arg[a] = r[b]
    OpcodeCall, // Calls another byte code function
    OpcodeCallHost, // Calls a native function
    OpcodeRet,
    OpcodeRetVal, // retval = r[a]
    // Memory
    OpcodeMove, // r[a] = r[b]
    OpcodeObjectNew, // r[a] = new object
    OpcodeObjectSet, // r[a][r[b]] = r[c] 
    OpcodeObjectGet, // r[a] = r[b][r[c]]

    // Numeric
    OpcodeAdd, // r[c] = r[a] + r[b]
    OpcodeSub, // r[c] = r[a] + r[b]
    OpcodeMul, // r[c] = r[a] + r[b]
    OpcodeDiv, // r[c] = r[a] + r[b]
    OpcodeEq, // r[c] = r[a] + r[b]
    OpcodeNotEq, // r[c] = r[a] + r[b]
    OpcodeGr, // r[c] = r[a] + r[b]
    OpcodeLess,  // r[c] = r[a] + r[b]
    OpcodeGrEq, // r[c] = r[a] + r[b]
    OpcodeLessEq, // r[c] = r[a] + r[b]

    // Values
    OpcodeLoadConst, // r[a] = const[b]
};

struct Inst {
    uint8_t opcode;
    uint8_t a;
    union {
        uint16_t s;
        struct {
            uint8_t b;
            uint8_t c;
        };
    };
};

struct Function {
    std::string name;
    std::vector<Inst> code;
    uint64_t locals;
};

struct Module {
    std::unordered_map<std::string, uint64_t> name_mapping;
    std::vector<ref<Function>> functions;
    std::vector<Value> constants;
};

void dump_inst(Inst inst);
void dump_module(ref<Module>);

}