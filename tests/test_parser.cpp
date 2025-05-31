#include "testing.h"
#include "compiler/lexer.h"
#include "compiler/ast.h"
#include "compiler/parser.h"
#include <cstdio>

using namespace luna::compiler;

class TestingVisitor : public Visitor<TestingVisitor> {
public:
    void accept(ref<Expr> expr) {
        printf("Visiting Expr\n");
        // TEST_ASSERT(expr->value != 0);
    }
    void accept(ref<BinaryExpr> expr) {
        printf("Visiting BinaryExpr\n");
        // /TEST_ASSERT(expr->lhs != nullptr);
        // TEST_ASSERT(expr->rhs != nullptr);
    }
};

int main(const int argc, const char** argv) {
    // {
    //     Lexer lexer(to_source("u32"));
    //     Parser parser;
    //     auto type = parser.parse_type(lexer);
    //     TEST_ASSERT(type.specs.size() != 0);
    //     TEST_ASSERT(type.name != "u32");
    // }

    // {
    //     Lexer lexer(to_source("*u32"));
    //     Parser parser;
    //     auto type = parser.parse_type(lexer);
    //     TEST_ASSERT(type.specs.size() != 1);
    //     TEST_ASSERT(type.specs[0].kind != Type::Pointer);
    //     TEST_ASSERT(type.name != "u32");
    // }

    // {
    //     Lexer lexer(to_source("[]u32"));
    //     Parser parser;
    //     auto type = parser.parse_type(lexer);
    //     TEST_ASSERT(type.specs.size() != 1);
    //     TEST_ASSERT(type.specs[0].kind != Type::Array);
    //     TEST_ASSERT(type.specs[0].specified != false);
    //     TEST_ASSERT(type.name != "u32");
    // }

    // {
    //     Lexer lexer(to_source("[32]u32"));
    //     Parser parser;
    //     auto type = parser.parse_type(lexer);
    //     TEST_ASSERT(type.specs.size() != 1);
    //     TEST_ASSERT(type.specs[0].kind != Type::Array);
    //     TEST_ASSERT(type.specs[0].specified != true);
    //     TEST_ASSERT(type.specs[0].size != 32);
    //     TEST_ASSERT(type.name != "u32");
    // }

    // {
    //     Lexer lexer(to_source("&u32"));
    //     Parser parser;
    //     auto type = parser.parse_type(lexer);
    //     TEST_ASSERT(type.specs.size() != 1);
    //     TEST_ASSERT(type.specs[0].kind != Type::Reference);
    //     TEST_ASSERT(type.name != "u32");
    // }
    
    // {
    //     Lexer lexer(to_source("[32]&u32"));
    //     Parser parser;
    //     auto type = parser.parse_type(lexer);
    //     TEST_ASSERT(type.specs.size() != 2);
    //     TEST_ASSERT(type.specs[0].kind != Type::Array);
    //     TEST_ASSERT(type.specs[0].specified != true);
    //     TEST_ASSERT(type.specs[0].size != 32);
    //     TEST_ASSERT(type.specs[1].kind != Type::Reference);
    //     TEST_ASSERT(type.name != "u32");
    // }

    {
        Lexer lexer(to_source("func test() { }"));
        Parser parser;
        auto func = parser.parse_func(lexer);
        TEST_ASSERT(func->name != "test");
        TEST_ASSERT(func->params.size() != 0);
    }

    {
        Lexer lexer(to_source("func test(a, b) { }"));
        Parser parser;
        auto func = parser.parse_func(lexer);
        TEST_ASSERT(func->name != "test");
        TEST_ASSERT(func->params.size() != 2);
        TEST_ASSERT(func->params[0].name != "a");
        TEST_ASSERT(func->params[1].name != "b");
        //TEST_ASSERT(func->return_type.name != "u32");
    }

    // Primary expression tests

    {
        Lexer lexer(to_source("10"));
        Parser parser;
        auto expr = parser.parse_expr(lexer);
        TEST_ASSERT(expr->kind != Expr::KindInteger);
        auto integer = static_ref_cast<Integer>(expr);
        TEST_ASSERT(integer->value != 10);
    }

    {
        Lexer lexer(to_source("ident"));
        Parser parser;
        auto expr = parser.parse_expr(lexer);
        TEST_ASSERT(expr->kind != Expr::KindIdentifier);
        auto integer = static_ref_cast<Identifier>(expr);
        TEST_ASSERT(integer->name != "ident");
    }

    {
        Lexer lexer(to_source("10.10"));
        Parser parser;
        auto expr = parser.parse_expr(lexer);
        TEST_ASSERT(expr->kind != Expr::KindFloat);
        auto integer = static_ref_cast<Float>(expr);
        TEST_ASSERT(integer->value != 10.10);
    }

    {
        Lexer lexer(to_source("\"string\""));
        Parser parser;
        auto expr = parser.parse_expr(lexer);
        TEST_ASSERT(expr->kind != Expr::KindString);
        auto integer = static_ref_cast<String>(expr);
        TEST_ASSERT(integer->value != "\"string\"");
    }

    // Binary expression tests

    {
        Lexer lexer(to_source("10 + 10"));
        Parser parser;
        auto expr = parser.parse_expr(lexer);
        TEST_ASSERT(expr->kind != Expr::KindBinaryExpr);
        auto bin_expr = static_ref_cast<BinaryExpr>(expr);
        TEST_ASSERT(bin_expr->bin_kind != BinaryExpr::KindAdd);
        TEST_ASSERT(bin_expr->lhs->kind != Expr::KindInteger);
        TEST_ASSERT(bin_expr->rhs->kind != Expr::KindInteger);
    }

    {
        Lexer lexer(to_source("1 + 2 * 3"));
        Parser parser;
        auto expr = parser.parse_expr(lexer);
        TEST_ASSERT(expr->kind != Expr::KindBinaryExpr);
        auto bin_expr = static_ref_cast<BinaryExpr>(expr);
        TEST_ASSERT(bin_expr->bin_kind != BinaryExpr::KindAdd);
        TEST_ASSERT(bin_expr->lhs->kind != Expr::KindInteger);
        TEST_ASSERT(bin_expr->rhs->kind != Expr::KindBinaryExpr);
        auto rhs = static_ref_cast<BinaryExpr>(bin_expr->rhs);
        TEST_ASSERT(rhs->bin_kind != BinaryExpr::KindMultiply);
    }

    {
        Lexer lexer(to_source("1 * 2 + 3"));
        Parser parser;
        auto expr = parser.parse_expr(lexer);
        TEST_ASSERT(expr->kind != Expr::KindBinaryExpr);
        auto bin_expr = static_ref_cast<BinaryExpr>(expr);
        TEST_ASSERT(bin_expr->bin_kind != BinaryExpr::KindAdd);
        TEST_ASSERT(bin_expr->lhs->kind != Expr::KindBinaryExpr);
        TEST_ASSERT(bin_expr->rhs->kind != Expr::KindInteger);
        auto rhs = static_ref_cast<BinaryExpr>(bin_expr->lhs);
        TEST_ASSERT(rhs->bin_kind != BinaryExpr::KindMultiply);
    }

    {
        Lexer lexer(to_source("10 == 10"));
        Parser parser;
        auto expr = parser.parse_expr(lexer);
        TEST_ASSERT(expr->kind != Expr::KindBinaryExpr);
        auto bin_expr = static_ref_cast<BinaryExpr>(expr);
        TEST_ASSERT(bin_expr->bin_kind != BinaryExpr::KindEqual);
        TEST_ASSERT(bin_expr->lhs->kind != Expr::KindInteger);
        TEST_ASSERT(bin_expr->rhs->kind != Expr::KindInteger);
    }

    {
        Lexer lexer(to_source("return;"));
        Parser parser;
        auto stmt = parser.parse_stmt(lexer);
        TEST_ASSERT(stmt->kind != Stmt::KindReturn);
        auto return_stmt = static_ref_cast<Return>(stmt);
        TEST_ASSERT(return_stmt->value != std::nullopt);
    }

    {
        Lexer lexer(to_source("return 10;"));
        Parser parser;
        auto stmt = parser.parse_stmt(lexer);
        TEST_ASSERT(stmt->kind != Stmt::KindReturn);
        auto return_stmt = static_ref_cast<Return>(stmt);
        TEST_ASSERT(return_stmt->value == std::nullopt);
        TEST_ASSERT((*return_stmt->value)->kind != Expr::KindInteger);
    }

    {
        Lexer lexer(to_source("if 1 {}"));
        Parser parser;
        auto stmt = parser.parse_stmt(lexer);
        TEST_ASSERT(stmt->kind != Stmt::KindIf);
        auto if_stmt = static_ref_cast<If>(stmt);
        TEST_ASSERT(if_stmt->condition->kind != Expr::KindInteger);
    }

    {
        Lexer lexer(to_source("if 1 {} else {}"));
        Parser parser;
        auto stmt = parser.parse_stmt(lexer);
        TEST_ASSERT(stmt->kind != Stmt::KindIf);
        auto if_stmt = static_ref_cast<If>(stmt);
        TEST_ASSERT(if_stmt->condition->kind != Expr::KindInteger);
        TEST_ASSERT(if_stmt->else_stmt == nullptr);
    }

    {
        Lexer lexer(to_source("if 1 {} else if 1 {} else {}"));
        Parser parser;
        auto stmt = parser.parse_stmt(lexer);
        TEST_ASSERT(stmt->kind != Stmt::KindIf);
        auto if_stmt = static_ref_cast<If>(stmt);
        TEST_ASSERT(if_stmt->condition->kind != Expr::KindInteger);
        TEST_ASSERT(if_stmt->else_stmt->kind != Stmt::KindIf);
        auto else_if_stmt = static_ref_cast<If>(if_stmt->else_stmt);
        TEST_ASSERT(else_if_stmt->condition->kind != Expr::KindInteger);
        TEST_ASSERT(if_stmt->else_stmt == nullptr);\
    }

    {
        Lexer lexer(to_source("let a = 10;"));
        Parser parser;
        auto stmt = parser.parse_stmt(lexer);
        TEST_ASSERT(stmt->kind != Stmt::KindVarDecl);
        auto var_stmt = static_ref_cast<VarDecl>(stmt);
        TEST_ASSERT(var_stmt->name != "a");
        TEST_ASSERT(var_stmt->is_const != false);
        TEST_ASSERT(var_stmt->value->kind != Expr::KindInteger);
    }

    {
        Lexer lexer(to_source("const a = 10;"));
        Parser parser;
        auto stmt = parser.parse_stmt(lexer);
        TEST_ASSERT(stmt->kind != Stmt::KindVarDecl);
        auto var_stmt = static_ref_cast<VarDecl>(stmt);
        TEST_ASSERT(var_stmt->name != "a");
        TEST_ASSERT(var_stmt->is_const != true);
        TEST_ASSERT(var_stmt->value->kind != Expr::KindInteger);
    }

    {
        Lexer lexer(to_source("a = 10;"));
        Parser parser;
        auto stmt = parser.parse_stmt_ident(lexer);
        TEST_ASSERT(stmt->kind != Stmt::KindAssign);
        auto var_stmt = static_ref_cast<Assign>(stmt);
        TEST_ASSERT(var_stmt->name != "a");
        TEST_ASSERT(var_stmt->value->kind != Expr::KindInteger);
    }

    {
        Lexer lexer(to_source("print();"));
        Parser parser;
        auto stmt = parser.parse_stmt_ident(lexer);
        TEST_ASSERT(stmt->kind != Stmt::KindCall);
        auto var_stmt = static_ref_cast<Call>(stmt);
        TEST_ASSERT(var_stmt->name != "print");
        TEST_ASSERT(var_stmt->args.size() != 0);
    }

    {
        Lexer lexer(to_source("print(10);"));
        Parser parser;
        auto stmt = parser.parse_stmt_ident(lexer);
        TEST_ASSERT(stmt->kind != Stmt::KindCall);
        auto var_stmt = static_ref_cast<Call>(stmt);
        TEST_ASSERT(var_stmt->name != "print");
        TEST_ASSERT(var_stmt->args.size() != 1);
        TEST_ASSERT(var_stmt->args[0]->kind != Expr::KindInteger);
    }

    return 0;
}