#include "ast.h"
#include "shared/type.h"
#include <cstdio>

namespace luna::compiler {

std::string Expr::name() {
    switch(kind) {
        #define NAME(name) case Kind##name: return #name;
        EXPR_NODES(NAME)
        #undef NAME
    }
} 

std::string Stmt::name(){
    switch(kind) {
        #define NAME(name) case Kind##name: return #name;
        STMT_NODES(NAME)
        #undef NAME
    }
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
    type = int_type();
}

Float::Float() {
    kind = KindFloat;
    type = number_type();
}

String::String() {
    kind = KindString;
    type = string_type();
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