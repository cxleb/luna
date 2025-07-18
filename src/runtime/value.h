#pragma once

#include <cstdint>

namespace luna::runtime {

#define TYPES(A) \
    A(Null) \
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

// forward declare Cell from heap
struct Cell;
struct Value;

struct Value {
    Value() {
        //type = TypeNull;
        value_number = 0;
    }
    Value(Cell* cell) {
        value_cell = cell;
        //type = TypeObject;
    }
    Value(int64_t value) {
        value_int = value;
        //type = TypeInt;
    }
    Value(double value) {
        value_number = value;
        //type = TypeFloat;
    }
    Value(bool value) {
        value_int = 0;
        value_boolean = value;
        //type = TypeBool;
    }
    //Type type;
    union {
        int64_t value_int;
        double value_number;
        bool value_boolean;
        // not used yet
        Cell* value_cell;
    };

    //bool eq(const Value& other) const;

    //bool operator==(const Value& other) const {
    //    //value_eq(this, &other);
    //    return eq(other);
    //}
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


// OpResult value_add(Value a, Value b);
// OpResult value_sub(Value a, Value b);
// OpResult value_mul(Value a, Value b);
// OpResult value_div(Value a, Value b);
// OpResult value_eq(Value a, Value b);
// OpResult value_neq(Value a, Value b);
// OpResult value_gr(Value a, Value b);
// OpResult value_gr_eq(Value a, Value b);
// OpResult value_less(Value a, Value b);
// OpResult value_less_eq(Value a, Value b);
// bool value_truthy(Value a);
// bool value_falsy(Value a);
// uint64_t value_hash(Value a);
// void value_print(Value a);

}

// template<>
// struct std::hash<luna::runtime::Value> {
//     std::size_t operator()(const luna::runtime::Value& v) const {
//         return value_hash(v);
//     }
// };