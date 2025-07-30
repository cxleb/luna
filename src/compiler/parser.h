#pragma once

#include "ast.h"
#include "lexer.h"
#include "../shared/utils.h"
#include "../shared/error.h"

namespace luna::compiler {

class Parser {
public:
    Parser(std::vector<char>&& source);
    ErrorOr<ref<Module>> parse_module();
    ErrorOr<ref<Func>> parse_func();
    ErrorOr<ref<Stmt>> parse_stmt();
    ErrorOr<ref<Stmt>> parse_if();
    ErrorOr<ref<Stmt>> parse_for();
    ErrorOr<ref<Stmt>> parse_while();
    ErrorOr<ref<Stmt>> parse_return();
    ErrorOr<ref<Stmt>> parse_block();
    ErrorOr<ref<Stmt>> parse_var();
    ErrorOr<ref<Stmt>> parse_expr_stmt();
    ErrorOr<ref<Expr>> parse_expr();
    ErrorOr<ref<Expr>> parse_primary_expr();
    ErrorOr<ref<Expr>> parse_ident();
    ErrorOr<ref<Expr>> parse_number();
    ErrorOr<ref<Expr>> parse_string();
    ErrorOr<ref<Expr>> parse_object_literal();
    ErrorOr<ref<Expr>> parse_array_literal();
    u8 parse_prec(Token token);
    ErrorOr<BinaryExpr::Kind> parse_bin_op_kind(Token token);
    ErrorOr<ref<Expr>> parse_bin_expr(u8 prec);
    ErrorOr<ref<Expr>> parse_left_hand_side_expr();
    ErrorOr<ref<Type>> parse_type();
private:
    template<typename T>
    ref<T> make_node() {
        return make_node<T>(lexer.peek().loc);
    }
    template<typename T>
    ref<T> make_node(Token token) {
        return make_node<T>(token.loc);
    }
    template<typename T>
    ref<T> make_node(SourceLoc loc) {
        auto node = make_ref<T>();
        node->loc = loc;
        return node;
    }
    template<typename T>
    ref<Expr> finish_expr(ref<T> expr) {
        return static_ref_cast<Expr>(expr);
    }
    template<typename T>
    ref<Stmt> finish_stmt(ref<T> expr) {
        return static_ref_cast<Stmt>(expr);
    }
private:
    Lexer lexer;
};

Error parser_error(Token token, const char* message, ...);

}