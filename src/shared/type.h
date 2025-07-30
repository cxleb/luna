#pragma once
#include "utils.h"

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
    inline bool is_unknown() const { return kind == TypeKindUnknown; }
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

