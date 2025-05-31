#pragma once
#include <cassert>
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
    D(Assign) \
    D(While) \
    D(For) \
    D(Call) \
    D(Block)
    
#define EXPR_NODES(D) \
    D(BinaryExpr) \
    D(Unary) \
    D(CallExpr) \
    D(Integer) \
    D(Float) \
    D(String) \
    D(Identifier)

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

class Call : public Stmt {
public:
    Call();
    std::string name;
    std::vector<ref<Expr>> args;
};

class Block : public Stmt {
public:
    Block();
    std::vector<ref<Stmt>> stmts;
};

class Assign : public Stmt {
public:
    Assign();
    std::string name;
    ref<Expr> value;
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

class CallExpr : public Expr {
public:
    CallExpr();
    Call call;
};

class Identifier : public Expr {
public:
    Identifier();
    std::string name;
};

class Integer : public Expr {
public:
    Integer();
    uint64_t value;
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
template <typename Impl, typename... Args>
class Visitor {
public:
    void visit(ref<Expr> expr, Args&... args) {
        switch(expr->kind) {
#define VISITOR_SWITCH(name) \
        case Expr::Kind##name: \
            static_cast<Impl*>(this)->accept( \
                static_ref_cast<name>(expr), args...); \
            return;
        EXPR_NODES(VISITOR_SWITCH)
#undef VISITOR_SWITCH
        default:
            return;
        }
    }

    void visit(ref<Stmt> stmt, Args&... args) {
        switch(stmt->kind) {
#define VISITOR_SWITCH(name) \
        case Stmt::Kind##name: \
            static_cast<Impl*>(this)->accept( \
                static_ref_cast<name>(stmt), args...); \
            return;
        STMT_NODES(VISITOR_SWITCH)
#undef VISITOR_SWITCH
        default:
            return;
        }
    }

    // Provides default accept implementations for a visitor
// #define VISITOR_DEF(name) void accept(ref<name> n) { em_assert(true);}
//     STMT_NODES(VISITOR_DEF)
//     EXPR_NODES(VISITOR_DEF)
// #undef VISITOR_DEF
};


}