#include "type.h"
#include "shared/utils.h"

bool ArrayType::compare(const ref<Type> other) const {
    if (other->kind != TypeKindArray) return false;
    auto array_other = static_ref_cast<ArrayType>(other);
    return element_type->compare(array_other->element_type);
} 

bool ArrayType::is_unknown() const {
    return element_type->is_unknown();
}

bool ArrayType::are_compatible(const ref<Type> other) const {
    if (other->kind == TypeKindUnknown) {
        return true;
    }
    if (other->kind != TypeKindArray) return false;
    auto array_other = static_ref_cast<ArrayType>(other);
    return element_type->are_compatible(array_other->element_type);
}

bool FunctionType::compare(const ref<Type> other) const {
    if (other->kind != TypeKindFunction) return false;
    auto other_ = static_ref_cast<FunctionType>(other);
    if (return_type.has_value()) {
        if (!other_->return_type.has_value()) {
            return false;
        }
        if (!(*return_type)->compare(*other_->return_type)) {
            return false;
        }
    } else {
        if (other_->return_type.has_value()) {
            return false;
        }
    }
    if (parameters.size() != other_->parameters.size()) {
        return false;
    }
    for(auto i = 0; i < parameters.size(); i++) {
        if(!parameters[i]->compare(other_->parameters[i])) {
            return false;
        }
    }
    return true;
} 

ref<Type> unknown_type() {
    static auto type = make_ref<UnknownType>();
    return static_ref_cast<Type>(type);
}

ref<Type> int_type() {
    static auto type = make_ref<IntegerType>();
    return static_ref_cast<Type>(type);
}

ref<Type> number_type() {
    static auto type = make_ref<NumberType>();
    return static_ref_cast<Type>(type);
}

ref<Type> string_type() {
    static auto type = make_ref<StringType>();
    return static_ref_cast<Type>(type);
}

ref<Type> bool_type() {
    static auto type = make_ref<BoolType>();
    return static_ref_cast<Type>(type);
}

ref<Type> array_type(ref<Type> element_type) {
    auto type = make_ref<ArrayType>();
    type->element_type = element_type;
    return static_ref_cast<Type>(type);
}