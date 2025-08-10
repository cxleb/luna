#include "shared/type.h"
#include "testing.h"
#include "compiler/ast.h"
#include "compiler/parser.h"

using namespace luna::compiler;

//class TestingVisitor : public Visitor<TestingVisitor> {
//public:
//    void accept(ref<Expr> expr) {
//        printf("Visiting Expr\n");
//        // TEST_ASSERT(expr->value != 0);
//    }
//    void accept(ref<BinaryExpr> expr) {
//        printf("Visiting BinaryExpr\n");
//        // /TEST_ASSERT(expr->lhs != nullptr);
//        // TEST_ASSERT(expr->rhs != nullptr);
//    }
//};

int main(const int argc, const char** argv) {
    {
        TEST_ASSERT(int_type()->compare(int_type()));
        TEST_ASSERT(!int_type()->compare(number_type()));
    }

    {
        Parser parser(to_source("int"));
        auto type = parser.parse_type().value();
        TEST_ASSERT(type->compare(int_type()));
    }

    {
        Parser parser(to_source("[]int"));
        auto type = parser.parse_type().value();
        TEST_ASSERT(type->compare(array_type(int_type())));
    }

    {
        Parser parser(to_source("[][]string"));
        auto type = parser.parse_type().value();
        TEST_ASSERT(type->compare(array_type(array_type(string_type()))));
    }

    {
        Parser parser(to_source("func test() { }"));
        auto func = parser.parse_func().value();
        TEST_ASSERT(func->name == "test");
        TEST_ASSERT(func->params.size() == 0);
    }

    {
        Parser parser(to_source("func test(a: int, b: int) { }"));
        auto func = parser.parse_func().value();
        TEST_ASSERT(func->name == "test");
        TEST_ASSERT(func->params.size() == 2);
        TEST_ASSERT(func->params[0].name == "a");
        TEST_ASSERT(func->params[0].type->compare(int_type()));
        TEST_ASSERT(func->params[1].name == "b");
        TEST_ASSERT(func->params[1].type->compare(int_type()));
    }

    {
        Parser parser(to_source("func test() int { }"));
        auto func = parser.parse_func().value();
        TEST_ASSERT(func->name == "test");
        TEST_ASSERT(func->params.size() == 0);
        TEST_ASSERT(func->return_type.value()->compare(int_type()));
    }

    // Primary expression tests

    {
        Parser parser(to_source("10"));
        auto expr = parser.parse_expr().value();
        TEST_ASSERT(expr->kind == Expr::KindInteger);
        auto integer = static_ref_cast<Integer>(expr);
        TEST_ASSERT(integer->value == 10);
    }

    {
        Parser parser(to_source("ident"));
        auto expr = parser.parse_expr().value();
        TEST_ASSERT(expr->kind == Expr::KindIdentifier);
        auto integer = static_ref_cast<Identifier>(expr);
        TEST_ASSERT(integer->name == "ident");
    }

    {
        Parser parser(to_source("10.10"));
        auto expr = parser.parse_expr().value();
        TEST_ASSERT(expr->kind == Expr::KindFloat);
        auto integer = static_ref_cast<Float>(expr);
        TEST_ASSERT(integer->value == 10.10);
    }

    {
        Parser parser(to_source("\"string\""));
        auto expr = parser.parse_expr().value();
        TEST_ASSERT(expr->kind == Expr::KindString);
        auto integer = static_ref_cast<String>(expr);
        TEST_ASSERT(integer->value == "string");
    }

    {
        Parser parser(to_source("[]"));
        auto expr = parser.parse_expr().value();
        TEST_ASSERT(expr->kind == Expr::KindArrayLiteral);
        auto literal = static_ref_cast<ArrayLiteral>(expr);
        TEST_ASSERT(literal->elements.size() == 0);
    }

    {
        Parser parser(to_source("[1, 2, 3]"));
        auto expr = parser.parse_expr().value();
        TEST_ASSERT(expr->kind == Expr::KindArrayLiteral);
        auto literal = static_ref_cast<ArrayLiteral>(expr);
        TEST_ASSERT(literal->elements.size() == 3);
    }

    {
        Parser parser(to_source("{}"));
        auto expr = parser.parse_expr().value();
        TEST_ASSERT(expr->kind == Expr::KindObjectLiteral);
        auto literal = static_ref_cast<ObjectLiteral>(expr);
    }

    {
        Parser parser(to_source("array[0]"));
        auto expr = parser.parse_expr().value();
        TEST_ASSERT(expr->kind == Expr::KindLookup);
        auto lookup = static_ref_cast<Lookup>(expr);
        TEST_ASSERT(lookup->expr->kind == Expr::KindIdentifier);
        TEST_ASSERT(lookup->index->kind == Expr::KindInteger);
    }

    {
        Parser parser(to_source("array[\"string\"]"));
        auto expr = parser.parse_expr().value();
        TEST_ASSERT(expr->kind == Expr::KindLookup);
        auto lookup = static_ref_cast<Lookup>(expr);
        TEST_ASSERT(lookup->expr->kind == Expr::KindIdentifier);
        TEST_ASSERT(lookup->index->kind == Expr::KindString);
    }

    {
        Parser parser(to_source("array[10 + a]"));
        auto expr = parser.parse_expr().value();
        TEST_ASSERT(expr->kind == Expr::KindLookup);
        auto lookup = static_ref_cast<Lookup>(expr);
        TEST_ASSERT(lookup->expr->kind == Expr::KindIdentifier);
        TEST_ASSERT(lookup->index->kind == Expr::KindBinaryExpr);
    }

    // Left Hand Side Expression tests

    {
        Parser parser(to_source("a = 10"));
        auto expr = parser.parse_left_hand_side_expr().value();
        TEST_ASSERT(expr->kind == Expr::KindAssign);
        auto assign = static_ref_cast<Assign>(expr);
        TEST_ASSERT(assign->local->kind == Expr::KindIdentifier);
        TEST_ASSERT(assign->value->kind == Expr::KindInteger);
    }

    {
        Parser parser(to_source("print()"));
        auto expr = parser.parse_left_hand_side_expr().value();
        TEST_ASSERT(expr->kind == Expr::KindCall);
        auto call = static_ref_cast<Call>(expr);
        TEST_ASSERT(call->name == "print");
        TEST_ASSERT(call->args.size() == 0);
    }

    {
        Parser parser(to_source("print(10)"));
        auto expr = parser.parse_left_hand_side_expr().value();
        TEST_ASSERT(expr->kind == Expr::KindCall);
        auto var_stmt = static_ref_cast<Call>(expr);
        TEST_ASSERT(var_stmt->name == "print");
        TEST_ASSERT(var_stmt->args.size() == 1);
        TEST_ASSERT(var_stmt->args[0]->kind == Expr::KindInteger);
    }

    // Binary expression tests

    {
        Parser parser(to_source("10 + 10"));
        auto expr = parser.parse_expr().value();
        TEST_ASSERT(expr->kind == Expr::KindBinaryExpr);
        auto bin_expr = static_ref_cast<BinaryExpr>(expr);
        TEST_ASSERT(bin_expr->bin_kind == BinaryExpr::KindAdd);
        TEST_ASSERT(bin_expr->lhs->kind == Expr::KindInteger);
        TEST_ASSERT(bin_expr->rhs->kind == Expr::KindInteger);
    }

    {
        Parser parser(to_source("1 + 2 * 3"));
        auto expr = parser.parse_expr().value();
        TEST_ASSERT(expr->kind == Expr::KindBinaryExpr);
        auto bin_expr = static_ref_cast<BinaryExpr>(expr);
        TEST_ASSERT(bin_expr->bin_kind == BinaryExpr::KindAdd);
        TEST_ASSERT(bin_expr->lhs->kind == Expr::KindInteger);
        TEST_ASSERT(bin_expr->rhs->kind == Expr::KindBinaryExpr);
        auto rhs = static_ref_cast<BinaryExpr>(bin_expr->rhs);
        TEST_ASSERT(rhs->bin_kind == BinaryExpr::KindMultiply);
    }

    {
        Parser parser(to_source("1 * 2 + 3"));
        auto expr = parser.parse_expr().value();
        TEST_ASSERT(expr->kind == Expr::KindBinaryExpr);
        auto bin_expr = static_ref_cast<BinaryExpr>(expr);
        TEST_ASSERT(bin_expr->bin_kind == BinaryExpr::KindAdd);
        TEST_ASSERT(bin_expr->lhs->kind == Expr::KindBinaryExpr);
        TEST_ASSERT(bin_expr->rhs->kind == Expr::KindInteger);
        auto rhs = static_ref_cast<BinaryExpr>(bin_expr->lhs);
        TEST_ASSERT(rhs->bin_kind == BinaryExpr::KindMultiply);
    }

    {
        Parser parser(to_source("10 == 10"));
        auto expr = parser.parse_expr().value();
        TEST_ASSERT(expr->kind == Expr::KindBinaryExpr);
        auto bin_expr = static_ref_cast<BinaryExpr>(expr);
        TEST_ASSERT(bin_expr->bin_kind == BinaryExpr::KindEqual);
        TEST_ASSERT(bin_expr->lhs->kind == Expr::KindInteger);
        TEST_ASSERT(bin_expr->rhs->kind == Expr::KindInteger);
    }

    // Statements

    {
        Parser parser(to_source("return;"));
        auto stmt = parser.parse_stmt().value();
        TEST_ASSERT(stmt->kind == Stmt::KindReturn);
        auto return_stmt = static_ref_cast<Return>(stmt);
        TEST_ASSERT(return_stmt->value == std::nullopt);
    }

    {
        Parser parser(to_source("return 10;"));
        auto stmt = parser.parse_stmt().value();
        TEST_ASSERT(stmt->kind == Stmt::KindReturn);
        auto return_stmt = static_ref_cast<Return>(stmt);
        TEST_ASSERT(return_stmt->value != std::nullopt);
        TEST_ASSERT((*return_stmt->value)->kind == Expr::KindInteger);
    }

    {
        Parser parser(to_source("if 1 {}"));
        auto stmt = parser.parse_stmt().value();
        TEST_ASSERT(stmt->kind == Stmt::KindIf);
        auto if_stmt = static_ref_cast<If>(stmt);
        TEST_ASSERT(if_stmt->condition->kind == Expr::KindInteger);
    }

    {
        Parser parser(to_source("if 1 {} else {}"));
        auto stmt = parser.parse_stmt().value();
        TEST_ASSERT(stmt->kind == Stmt::KindIf);
        auto if_stmt = static_ref_cast<If>(stmt);
        TEST_ASSERT(if_stmt->condition->kind == Expr::KindInteger);
        TEST_ASSERT(if_stmt->else_stmt != nullptr);
    }

    {
        Parser parser(to_source("if 1 {} else if 1 {} else {}"));
        auto stmt = parser.parse_stmt().value();
        TEST_ASSERT(stmt->kind == Stmt::KindIf);
        auto if_stmt = static_ref_cast<If>(stmt);
        TEST_ASSERT(if_stmt->condition->kind == Expr::KindInteger);
        TEST_ASSERT(if_stmt->else_stmt->kind == Stmt::KindIf);
        auto else_if_stmt = static_ref_cast<If>(if_stmt->else_stmt);
        TEST_ASSERT(else_if_stmt->condition->kind == Expr::KindInteger);
        TEST_ASSERT(if_stmt->else_stmt != nullptr);
    }

    {
        Parser parser(to_source("let a = 10;"));
        auto stmt = parser.parse_stmt().value();
        TEST_ASSERT(stmt->kind == Stmt::KindVarDecl);
        auto var_stmt = static_ref_cast<VarDecl>(stmt);
        TEST_ASSERT(var_stmt->name == "a");
        TEST_ASSERT(var_stmt->is_const == false);
        TEST_ASSERT(var_stmt->value->kind == Expr::KindInteger);
    }

    {
        Parser parser(to_source("let a: int = 10;"));
        auto stmt = parser.parse_stmt().value();
        TEST_ASSERT(stmt->kind == Stmt::KindVarDecl);
        auto var_stmt = static_ref_cast<VarDecl>(stmt);
        TEST_ASSERT(var_stmt->name == "a");
        TEST_ASSERT(var_stmt->is_const == false);
        TEST_ASSERT(var_stmt->value->kind == Expr::KindInteger);
    }

    {
        Parser parser(to_source("const a = 10;"));
        auto stmt = parser.parse_stmt().value();
        TEST_ASSERT(stmt->kind == Stmt::KindVarDecl);
        auto var_stmt = static_ref_cast<VarDecl>(stmt);
        TEST_ASSERT(var_stmt->name == "a");
        TEST_ASSERT(var_stmt->is_const == true);
        TEST_ASSERT(var_stmt->value->kind == Expr::KindInteger);
    }

    {
        Parser parser(to_source("a = 10;"));
        auto stmt = parser.parse_expr_stmt().value();
        TEST_ASSERT(stmt->kind == Stmt::KindExprStmt);
        auto expr_stmt = static_ref_cast<ExprStmt>(stmt);
        TEST_ASSERT(expr_stmt->expr->kind == Expr::KindAssign);
    }

    {
        Parser parser(to_source("print();"));
        auto stmt = parser.parse_expr_stmt().value();
        TEST_ASSERT(stmt->kind == Stmt::KindExprStmt);
        auto expr_stmt = static_ref_cast<ExprStmt>(stmt);
        TEST_ASSERT(expr_stmt->expr->kind == Expr::KindCall);
    }

    {
        Parser parser(to_source("print(10);"));
        auto stmt = parser.parse_expr_stmt().value(); 
        TEST_ASSERT(stmt->kind == Stmt::KindExprStmt);
        auto expr_stmt = static_ref_cast<ExprStmt>(stmt);
        TEST_ASSERT(expr_stmt->expr->kind == Expr::KindCall);
    }

    return 0;
}