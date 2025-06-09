#pragma once
#include <string>
#include <vector>
#include <optional>
#include "../shared/utils.h"
#include "lexer.h"

namespace luna::compiler {

#define STMT_NODES(D) \
    D(If) \
    D(Return) \
    D(VarDecl) \
    D(While) \
    D(For) \
    D(Block) \
    D(ExprStmt) \
    
#define EXPR_NODES(D) \
    D(BinaryExpr) \
    D(Unary) \
    D(Assign) \
    D(Call) \
    D(Integer) \
    D(Float) \
    D(String) \
    D(Identifier) \
    D(Lookup) \
    D(ArrayLiteral) \
    D(ObjectLiteral)

class Node {
public:
    // The underlying token, can be used to get the source locations
    Token token;
private:
};

// class Type {
// public:
//     enum Kind {
//         Pointer,
//         Reference,
//         Array
//     };
//     struct Spec {
//         Kind kind;
//         // used when the type is Kind::Array
//         bool specified; 
//         uint64_t size;
//     };

//     std::vector<Spec> specs;
//     std::string name;
//     bool is_unknown;
// };

class Stmt : public Node {
public:  
    enum Kind {
        #define NAME(name) Kind##name,
        STMT_NODES(NAME)
        #undef NAME
    } kind;
};

class Expr : public Node {
public:
    enum Kind {
        #define NAME(name) Kind##name,
        EXPR_NODES(NAME)
        #undef NAME
    } kind;
};

// Statements

class If : public Stmt {
public:
    If();
    ref<Expr> condition;
    ref<Stmt> then_stmt;
    ref<Stmt> else_stmt;
};

class Return : public Stmt {
public:
    Return();
    std::optional<ref<Expr>> value;
};

class VarDecl : public Stmt {
public:
    VarDecl();
    std::string name;
    bool is_const;
    //Type type;
    ref<Expr> value;
};

class Block : public Stmt {
public:
    Block();
    std::vector<ref<Stmt>> stmts;
};

class While : public Stmt {
public:
    While();
    ref<Expr> condition;
    ref<Stmt> loop;
};

class For : public Stmt {
public:
    For();
    std::string name;
    ref<Expr> iterator;
    ref<Stmt> loop;
};

class ExprStmt : public Stmt {
public:
    ExprStmt();
    ref<Expr> expr;
};

// Expressions

class BinaryExpr : public Expr {
public:
    BinaryExpr();
    enum Kind {
        KindAdd,
        KindSubtract,
        KindMultiply,
        KindDivide,
        KindEqual,
        KindNotEqual,
        KindLessThan,
        KindGreaterThan,
        KindLessThanEqual,
        KindGreaterThanEqual
    } bin_kind;
    ref<Expr> lhs;
    ref<Expr> rhs;
};

class Unary : public Expr {
    Unary();
    ref<Expr> expr;
};

class Call : public Expr {
public:
    Call();
    std::string name;
    std::vector<ref<Expr>> args;
};

class Assign : public Expr {
public:
    Assign();
    ref<Expr> local;
    ref<Expr> value;
};

class Identifier : public Expr {
public:
    Identifier();
    std::string name;
};

class Lookup : public Expr {
public:
    Lookup();
    ref<Expr> expr;
    ref<Expr> index;
};

class ObjectLiteral : public Expr {
public:
    ObjectLiteral();
    std::vector<ref<Expr>> elements;
};

class ArrayLiteral : public Expr {
public:
    ArrayLiteral();
    std::vector<ref<Expr>> elements;
};

class Integer : public Expr {
public:
    Integer();
    int64_t value;
};

class Float : public Expr {
public:
    Float();
    double value;
};

class String : public Expr {
public:
    String();
    std::string value;
};

// Functions

struct Parameter {
    std::string name;
    //Type type;
};

class Func : public Node {
public:
    std::string name;
    std::vector<Parameter> params;
    //Type return_type;
    ref<Stmt> root;
};

// Modules

class Module : public Node {
public:
    std::vector<ref<Func>> funcs;
};

// Visitor
template <typename Impl, typename StmtRet, typename ExprRet, typename... Args>
class Visitor {
public:
    uint8_t visit(ref<Expr> expr, Args&... args) {
        switch(expr->kind) {
#define VISITOR_SWITCH(name) \
        case Expr::Kind##name: \
            return static_cast<Impl*>(this)->accept( \
                static_ref_cast<name>(expr), args...);
        EXPR_NODES(VISITOR_SWITCH)
#undef VISITOR_SWITCH
        }
    }

    StmtRet visit(ref<Stmt> stmt, Args&... args) {
        switch(stmt->kind) {
#define VISITOR_SWITCH(name) \
        case Stmt::Kind##name: \
            return static_cast<Impl*>(this)->accept( \
                static_ref_cast<name>(stmt), args...);
        STMT_NODES(VISITOR_SWITCH)
#undef VISITOR_SWITCH
        }
    }

    // Provides default accept implementations for a visitor
// #define VISITOR_DEF(name) void accept(ref<name> n) { em_assert(true);}
//     STMT_NODES(VISITOR_DEF)
//     EXPR_NODES(VISITOR_DEF)
// #undef VISITOR_DEF
};


}