#include "value.h"
#include "heap.h"
#include <cmath>
#include <functional>

namespace luna::runtime {


const char* TypeKindNames[] = {
#define A(name) #name,
    TYPES(A)
#undef A
};

const char* get_name_for_type(Type type) {
    return TypeKindNames[type];
}

bool Value::eq(const Value& b) const {
    if (type != b.type) return false;
    switch (type) {
    case TypeNull:
        return true;
    case TypeInt:
        return value_int == b.value_int;
    case TypeFloat:
        return value_float == b.value_float;
    case TypeBool:
        return value_boolean == b.value_boolean;
    case TypeObject:
        return value_object->equal(b.value_object);
    }
}

OpResult value_add(Value a, Value b) {
    // if the values are the same type, then just do a direct comparison
    if (a.type == TypeInt && b.type == TypeInt) {
        return a.value_int + b.value_int;
    }
    if (a.type == TypeFloat && b.type == TypeFloat) {
        return a.value_float + b.value_float;
    }
    if (a.type == TypeInt && b.type == TypeFloat) {
        return a.value_int + b.value_float;
    }
    if (a.type == TypeFloat && b.type == TypeInt) {
        return a.value_float + b.value_int;
    }
    return OpResult::MismatchedTypes;
}

OpResult value_sub(Value a, Value b) {
    // if the values are the same type, then just do a direct comparison
    if (a.type == TypeInt && b.type == TypeInt) {
        return a.value_int - b.value_int;
    }
    if (a.type == TypeFloat && b.type == TypeFloat) {
        return a.value_float - b.value_float;
    }
    if (a.type == TypeInt && b.type == TypeFloat) {
        return a.value_int - b.value_float;
    }
    if (a.type == TypeFloat && b.type == TypeInt) {
        return a.value_float - b.value_int;
    }
    return OpResult::MismatchedTypes;
}

OpResult value_mul(Value a, Value b) {
    // if the values are the same type, then just do a direct comparison
    if (a.type == TypeInt && b.type == TypeInt) {
        return a.value_int * b.value_int;
    }
    if (a.type == TypeFloat && b.type == TypeFloat) {
        return a.value_float * b.value_float;
    }
    if (a.type == TypeInt && b.type == TypeFloat) {
        return a.value_int * b.value_float;
    }
    if (a.type == TypeFloat && b.type == TypeInt) {
        return a.value_float * b.value_int;
    }
    return OpResult::MismatchedTypes;
}

OpResult value_div(Value a, Value b) {
    // if the values are the same type, then just do a direct comparison
    if (a.type == TypeInt && b.type == TypeInt) {
        return a.value_int / b.value_int;
    }
    if (a.type == TypeFloat && b.type == TypeFloat) {
        return a.value_float / b.value_float;
    }
    if (a.type == TypeInt && b.type == TypeFloat) {
        return a.value_int / b.value_float;
    }
    if (a.type == TypeFloat && b.type == TypeInt) {
        return a.value_float / b.value_int;
    }
    return OpResult::MismatchedTypes;
}

OpResult value_eq(Value a, Value b) {
    return a.eq(b);
}

OpResult value_neq(Value a, Value b) {
    // if the values are the same type, then just do a direct comparison
    if (a.type == b.type) {
        return a.value_int != b.value_int;
    }
    if (a.type == TypeInt && b.type == TypeFloat) {
        return a.value_int != b.value_float;
    }
    if (a.type == TypeFloat && b.type == TypeInt) {
        return a.value_float != b.value_int;
    }
    return OpResult::MismatchedTypes;
}


OpResult value_gr(Value a, Value b) {
    // if the values are the same type, then just do a direct comparison
    if (a.type == TypeInt && b.type == TypeInt) {
        return a.value_int > b.value_int;
    }
    if (a.type == TypeFloat && b.type == TypeFloat) {
        return a.value_float > b.value_float;
    }
    if (a.type == TypeInt && b.type == TypeFloat) {
        return a.value_int > b.value_float;
    }
    if (a.type == TypeFloat && b.type == TypeInt) {
        return a.value_float > b.value_int;
    }
    return OpResult::MismatchedTypes;
}

OpResult value_gr_eq(Value a, Value b) {
    // if the values are the same type, then just do a direct comparison
    if (a.type == TypeInt && b.type == TypeInt) {
        return a.value_int >= b.value_int;
    }
    if (a.type == TypeFloat && b.type == TypeFloat) {
        return a.value_float >= b.value_float;
    }
    if (a.type == TypeInt && b.type == TypeFloat) {
        return a.value_int >= b.value_float;
    }
    if (a.type == TypeFloat && b.type == TypeInt) {
        return a.value_float >= b.value_int;
    }
    return OpResult::MismatchedTypes;
}

OpResult value_less(Value a, Value b) {
    // if the values are the same type, then just do a direct comparison
    if (a.type == TypeInt && b.type == TypeInt) {
        return a.value_int < b.value_int;
    }
    if (a.type == TypeFloat && b.type == TypeFloat) {
        return a.value_float < b.value_float;
    }
    if (a.type == TypeInt && b.type == TypeFloat) {
        return a.value_int < b.value_float;
    }
    if (a.type == TypeFloat && b.type == TypeInt) {
        return a.value_float < b.value_int;
    }
    return OpResult::MismatchedTypes;
}

OpResult value_less_eq(Value a, Value b) {
    // if the values are the same type, then just do a direct comparison
    if (a.type == TypeInt && b.type == TypeInt) {
        return a.value_int <= b.value_int;
    }
    if (a.type == TypeFloat && b.type == TypeFloat) {
        return a.value_float <= b.value_float;
    }
    if (a.type == TypeInt && b.type == TypeFloat) {
        return a.value_int <= b.value_float;
    }
    if (a.type == TypeFloat && b.type == TypeInt) {
        return a.value_float <= b.value_int;
    }
    return OpResult::MismatchedTypes;
}

bool value_truthy(Value a) {
    return !value_falsy(a);
}

bool value_falsy(Value a) {
    switch (a.type) {
        case TypeNull:
            return true;
        case TypeInt:
            return a.value_int == 0;
        case TypeFloat:
            return a.value_float == 0 || a.value_float == NAN;
        case TypeBool:
            return !a.value_boolean;
        case TypeObject:
            return a.value_object == nullptr;
    }
}

uint64_t value_hash(Value a) {
    switch (a.type) {
        case TypeNull:
            return 0;
        case TypeInt:
            return std::hash<int64_t>()(a.value_int);
        case TypeFloat:
            return std::hash<int64_t>()(a.value_float);
        case TypeBool:
            return std::hash<bool>()(a.value_float);
        case TypeObject:
            return a.value_object->hash();
    }
    return 0;
}

void value_print(Value a) {
    switch (a.type) {    
    case luna::runtime::TypeNull:
        printf("Null ");
        break;
    case luna::runtime::TypeInt:
        printf("%lld ", a.value_int);
        break;
    case luna::runtime::TypeFloat:
        printf("%f ", a.value_float);
        break;
    case luna::runtime::TypeBool:
        printf("%s ", a.value_boolean ? "true" : "false");
        break;
    case luna::runtime::TypeObject: {
        auto* cell = a.value_object;
        switch(cell->kind) {
            case luna::runtime::Cell::KindString: {
                auto* str = static_cast<luna::runtime::String*>(cell);
                printf("%s ", str->c_str());
                break;
            }
            case luna::runtime::Cell::KindObject: {
                printf("<obj> ");
                break;
            }
        }
        break;
    }
    }
}

}