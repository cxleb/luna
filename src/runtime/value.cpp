#include "value.h"
#include <cmath>

namespace luna::runtime {

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
    // if the values are the same type, then just do a direct comparison
    if (a.type == b.type) {
        return a.value_int == b.value_int;
    }
    if (a.type == TypeInt && b.type == TypeFloat) {
        return a.value_int == b.value_float;
    }
    if (a.type == TypeFloat && b.type == TypeInt) {
        return a.value_float == b.value_int;
    }
    return OpResult::MismatchedTypes;
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

}