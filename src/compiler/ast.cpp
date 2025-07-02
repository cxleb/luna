#include "ast.h"
#include <cstdio>

namespace luna::compiler {


Type::Type() {
    is_unknown = true;
}

Type::Type(TypeKind k) {
    is_unknown = false;
    array_count = 0;
    kind = k;
}

Type::Type(const std::string& str) {
    is_unknown = false;
    kind = TypeIdentifier;
    name = str;
    array_count = 0;
}

If::If() {
    kind = KindIf;
}

Return::Return() {
    kind = KindReturn;
}

VarDecl::VarDecl() {
    kind = KindVarDecl;
}

Block::Block() {
    kind = KindBlock;
}

While::While() {
    kind = KindWhile;
}

For::For() {
    kind = KindFor;
}

ExprStmt::ExprStmt() {
    kind = KindExprStmt;
}

BinaryExpr::BinaryExpr() {
    kind = KindBinaryExpr;
}

Unary::Unary() {
    kind = KindUnary;
}

Call::Call() {
    kind = KindCall;
}

Assign::Assign() {
    kind = KindAssign;
}

Lookup::Lookup() {
    kind = KindLookup;
}

Identifier::Identifier() {
    kind = KindIdentifier;
}

Integer::Integer() {
    kind = KindInteger;
    type = Type(TypeInteger);
}

Float::Float() {
    kind = KindFloat;
    type = Type(TypeNumber);
}

String::String() {
    kind = KindString;
    type = Type(TypeString);
}

ArrayLiteral::ArrayLiteral() {
    kind = KindArrayLiteral;
}

ObjectLiteral::ObjectLiteral() {
    kind = KindObjectLiteral;
}

void Module::dump() {
    for(auto& func: funcs) {
        printf("Func: %s\n", func->name.c_str());
    }
}

}