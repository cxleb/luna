#include "ast.h"
#include <sys/syslimits.h>

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

}