#pragma once

#include "ast.h"
#include "lexer.h"
#include "../shared/utils.h"

namespace luna::compiler {

class Parser {
public:
    ref<Module> parse_file(std::vector<char>&& source);
    ref<Func> parse_func(Lexer& lexer);
    ref<Stmt> parse_stmt(Lexer& lexer);
    ref<Stmt> parse_if(Lexer& lexer);
    ref<Stmt> parse_for(Lexer& lexer);
    ref<Stmt> parse_while(Lexer& lexer);
    ref<Stmt> parse_return(Lexer& lexer);
    ref<Stmt> parse_block(Lexer& lexer);
    ref<Stmt> parse_var(Lexer& lexer);
    ref<Stmt> parse_stmt_ident(Lexer& lexer);
    ref<Expr> parse_expr(Lexer& lexer);
    ref<Expr> parse_primary_expr(Lexer& lexer);
    ref<Expr> parse_ident(Lexer& lexer);
    ref<Expr> parse_number(Lexer& lexer);
    ref<Expr> parse_string(Lexer& lexer);
    u8 parse_prec(Token token);
    BinaryExpr::Kind parse_bin_op_kind(Token token);
    ref<Expr> parse_bin_expr(Lexer& lexer, u8 prec);
    //ast::Type parse_type(Lexer& lexer);
};

void parser_error [[noreturn]] (Token token, const char* message, ...);

}