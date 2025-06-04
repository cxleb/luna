#pragma once

#include "../shared/utils.h"

#include <cstdint>
#include <vector>
#include <unordered_map>
#include <string>

namespace luna::runtime {

enum Opcode {
    // Control Flow
    OpcodeBr,
    OpcodeCondBr,
    // calls have the lower 32 bits as the id and the upper 32 bits as the 
    // number of arguements
    OpcodeCall, // Calls another byte code function
    OpcodeCallHost, // Calls a native function
    OpcodeRet,
    // Memory
    OpcodeStore,
    OpcodeLoad,
    OpcodeObjectCreate,
    OpcodeObjectSet,
    OpcodeObjectGet,

    // Numeric
    OpcodeAdd,
    OpcodeSub,
    OpcodeMul,
    OpcodeDiv,
    OpcodeEq,
    OpcodeNotEq,
    OpcodeGr,
    OpcodeLess,
    OpcodeGrEq,
    OpcodeLessEq,

    // Values
    OpcodeInt,
    OpcodeFloat,
    OpcodeCell,
};

struct Inst {
    uint32_t opcode;
    union {
        uint64_t operand_int;
        double operand_float;
        bool operand_boolean;
        void* operand_ptr;
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
};

}