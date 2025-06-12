#include "ast.h"
#include <cstdio>

namespace luna::compiler {

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
}

Float::Float() {
    kind = KindFloat;
}

String::String() {
    kind = KindString;
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