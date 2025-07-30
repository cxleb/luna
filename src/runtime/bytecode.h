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
    OpcodeNumberAdd, // r[c] = r[a] + r[b]
    OpcodeNumberSub, // r[c] = r[a] + r[b]
    OpcodeNumberMul, // r[c] = r[a] + r[b]
    OpcodeNumberDiv, // r[c] = r[a] + r[b]
    OpcodeNumberEq, // r[c] = r[a] + r[b]
    OpcodeNumberNotEq, // r[c] = r[a] + r[b]
    OpcodeNumberGr, // r[c] = r[a] + r[b]
    OpcodeNumberLess,  // r[c] = r[a] + r[b]
    OpcodeNumberGrEq, // r[c] = r[a] + r[b]
    OpcodeNumberLessEq, // r[c] = r[a] + r[b]

    OpcodeIntAdd, // r[c] = r[a] + r[b]
    OpcodeIntSub, // r[c] = r[a] + r[b]
    OpcodeIntMul, // r[c] = r[a] + r[b]
    OpcodeIntDiv, // r[c] = r[a] + r[b]
    OpcodeIntEq, // r[c] = r[a] + r[b]
    OpcodeIntNotEq, // r[c] = r[a] + r[b]
    OpcodeIntGr, // r[c] = r[a] + r[b]
    OpcodeIntLess,  // r[c] = r[a] + r[b]
    OpcodeIntGrEq, // r[c] = r[a] + r[b]
    OpcodeIntLessEq, // r[c] = r[a] + r[b]
    
    OpcodeConvert, // r[b] = (Number)r[a]
    OpcodeTruncate, // r[b] = (Int)r[a]

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

struct Constant {
    std::optional<std::string> name;
    Type type;
    Value value;
};

struct Module {
    std::unordered_map<std::string, uint64_t> name_mapping;
    std::vector<ref<Function>> functions;
    std::vector<Value> constants;
};

void dump_inst(Inst inst);
void dump_module(ref<Module>);

}