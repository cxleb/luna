#pragma once

#include <cstdint>

namespace luna::runtime {

#define TYPES(A) \
    A(Int) \
    A(Float) \
    A(Bool) \
    A(Object) \

enum Type {
#define A(name) Type##name,
    TYPES(A)
#undef A
};

const char* get_name_for_type(Type type);

struct Value {
    Value() = default;
    Value(int64_t value) {
        value_int = value;
        type = TypeInt;
    }
    Value(double value) {
        value_float = value;
        type = TypeFloat;
    }
    Value(bool value) {
        value_int = 0;
        value_boolean = value;
        type = TypeBool;
    }
    Type type;
    union {
        int64_t value_int;
        double value_float;
        bool value_boolean;
        // not used yet
        void* value_object;
    };
};

struct OpResult {
    enum Reason {
        MismatchedTypes,
    };
    OpResult(int64_t val) : value(val), not_valid(false) {
    }
    OpResult(double val) : value(val), not_valid(false) {
    }
    OpResult(bool val) : value(val), not_valid(false) {
    }
    OpResult(Reason r) : reason(r), not_valid(true) {
    }
    bool not_valid;
    Value value;
    Reason reason;
};

OpResult value_add(Value a, Value b);
OpResult value_sub(Value a, Value b);
OpResult value_mul(Value a, Value b);
OpResult value_div(Value a, Value b);
OpResult value_eq(Value a, Value b);
OpResult value_neq(Value a, Value b);
OpResult value_gr(Value a, Value b);
OpResult value_gr_eq(Value a, Value b);
OpResult value_less(Value a, Value b);
OpResult value_less_eq(Value a, Value b);
bool value_truthy(Value a);
bool value_falsy(Value a);

}