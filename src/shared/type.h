#pragma once
#include "utils.h"
#include <vector>

enum TypeKind {
    TypeKindUnknown,
    TypeKindInteger,
    TypeKindNumber,
    TypeKindString,
    TypeKindBool,
    TypeKindArray,
    TypeKindFunction,
    TypeKindInterface,
    TypeKindStruct,
};

class Type {
public:
    virtual bool is_numeric() const { return false; }
    virtual bool compare(const ref<Type> other) const {
        return other->kind == kind;
    }
    // Checks if the other type has the same shape as this type, `this` is not
    // supposed to have unknown in it, and the `other` type is supposed to be 
    // the one with the unknown type somewhere.
    virtual bool are_compatible(const ref<Type> other) const {
        if (other->kind == TypeKindUnknown) {
            return true;
        }
        return this->kind == other->kind;
    }
    virtual bool is_unknown() const { return kind == TypeKindUnknown; }
    inline bool is_integer() const { return kind == TypeKindInteger; }
    inline bool is_array() const { return kind == TypeKindArray; }
    TypeKind kind;
protected:
    Type(TypeKind kind) : kind(kind) {}
};

class UnknownType : public Type {
public:
    UnknownType() : Type(TypeKindUnknown) {}
};

class IntegerType : public Type {
public:
    IntegerType() : Type(TypeKindInteger) {}
    bool is_numeric() const override { return true; }
};

class NumberType : public Type {
public:
    NumberType() : Type(TypeKindNumber) {}
    bool is_numeric() const override { return true; }
};

class StringType : public Type {
public:
    StringType() : Type(TypeKindString) {}
};

class BoolType : public Type {
public:
    BoolType() : Type(TypeKindBool) {}
};

class ArrayType : public Type {
public:
    ArrayType() : Type(TypeKindArray) {}
    bool compare(const ref<Type> other) const override;
    bool is_unknown() const override;
    bool are_compatible(const ref<Type> other) const override;

    ref<Type> element_type;
};

class FunctionType : public Type {
public:
    FunctionType() : Type(TypeKindFunction) {}
    bool compare(const ref<Type> other) const override;
    std::optional<ref<Type>> return_type;
    std::vector<ref<Type>> parameters;
};

ref<Type> unknown_type();
ref<Type> int_type();
ref<Type> number_type();
ref<Type> string_type();
ref<Type> bool_type();
ref<Type> array_type(ref<Type> element_type);
ref<Type> function_type(std::optional<ref<Type>> return_type, const std::vector<ref<Type>>& params);

bool are_types_compatible(ref<Type> a, ref<Type> b);