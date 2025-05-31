#include "ast.h"

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

Call::Call() {
    kind = KindCall;
}

Block::Block() {
    kind = KindBlock;
}

Assign::Assign() {
    kind = KindAssign;
}

While::While() {
    kind = KindWhile;
}

For::For() {
    kind = KindFor;
}

BinaryExpr::BinaryExpr() {
    kind = KindBinaryExpr;
}

Unary::Unary() {
    kind = KindUnary;
}

CallExpr::CallExpr() {
    kind = KindCallExpr;
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

}