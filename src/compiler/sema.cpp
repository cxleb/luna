#include "sema.h"
namespace luna::compiler {

class Inference {
    friend class Sema;

    void visit(ref<Expr> expr) {
        switch(expr->kind) {
#define VISITOR_SWITCH(name) \
        case Expr::Kind##name: \
            return this->accept( \
                static_ref_cast<name>(expr));
        EXPR_NODES(VISITOR_SWITCH)
#undef VISITOR_SWITCH
        }
    }

    void visit(ref<Stmt> stmt) {
        switch(stmt->kind) {
#define VISITOR_SWITCH(name) \
        case Stmt::Kind##name: \
            this->accept(static_ref_cast<name>(stmt)); \
            break;
        STMT_NODES(VISITOR_SWITCH)
#undef VISITOR_SWITCH
        }
    }

    // Statements
    void accept(ref<Stmt> stmt) {
        printf("Oh fuck, we should not be here! (Stmt)\n");
    }

    void accept(ref<If> stmt) {
    }
    
    void accept(ref<Return> ret) {
    }
    
    void accept(ref<VarDecl> decl) {
    }
    
    void accept(ref<While> stmt) {
    }
    
    void accept(ref<For> stmt) {
    }
    
    void accept(ref<Block> block) {
    }

    void accept(ref<ExprStmt> expr_stmt) {
    }

    // Expressions
    void accept(ref<Expr> expr) {
        printf("Oh fuck, we should not be here! (Expr)\n");
    }

    void accept(ref<BinaryExpr> expr) {
        switch(expr->bin_kind) {
            case BinaryExpr::KindAdd:
                break;
            case BinaryExpr::KindSubtract:
                break;
            case BinaryExpr::KindMultiply:
                break;
            case BinaryExpr::KindDivide:
                break;
            case BinaryExpr::KindEqual:
                break;
            case BinaryExpr::KindNotEqual:
                break;
            case BinaryExpr::KindLessThan:
                break;
            case BinaryExpr::KindGreaterThan:
                break;
            case BinaryExpr::KindLessThanEqual:
                break;
            case BinaryExpr::KindGreaterThanEqual:
                break;
        }
    }
    
    void accept(ref<Unary> expr, std::optional<uint8_t> into) {
    }

    void accept(ref<Assign> assign) {
    }
    
    void accept(ref<Call> call) {
    }
    
    void accept(ref<Integer> expr) {
        // Nothing to do ?
    }
    
    void accept(ref<Float> expr) {
        // Nothing to do ?
    }
    
    void accept(ref<String> str) {
        // Nothing to do ?
    }
    
    void accept(ref<Identifier> ident) {
    }

    void accept(ref<Lookup> lookup) {
    }

    void accept(ref<ObjectLiteral> literal) {
    }

    void accept(ref<ArrayLiteral> literal) {
    }
};

}