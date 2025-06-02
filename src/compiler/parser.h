#pragma once

#include "ast.h"
#include "lexer.h"
#include "../shared/utils.h"
#include "../shared/error.h"

namespace luna::compiler {

class Parser {
public:
    ErrorOr<ref<Module>> parse_file(std::vector<char>&& source);
    ErrorOr<ref<Func>> parse_func(Lexer& lexer);
    ErrorOr<ref<Stmt>> parse_stmt(Lexer& lexer);
    ErrorOr<ref<Stmt>> parse_if(Lexer& lexer);
    ErrorOr<ref<Stmt>> parse_for(Lexer& lexer);
    ErrorOr<ref<Stmt>> parse_while(Lexer& lexer);
    ErrorOr<ref<Stmt>> parse_return(Lexer& lexer);
    ErrorOr<ref<Stmt>> parse_block(Lexer& lexer);
    ErrorOr<ref<Stmt>> parse_var(Lexer& lexer);
    ErrorOr<ref<Stmt>> parse_stmt_ident(Lexer& lexer);
    ErrorOr<ref<Expr>> parse_expr(Lexer& lexer);
    ErrorOr<ref<Expr>> parse_primary_expr(Lexer& lexer);
    ErrorOr<ref<Expr>> parse_ident(Lexer& lexer);
    ErrorOr<ref<Expr>> parse_number(Lexer& lexer);
    ErrorOr<ref<Expr>> parse_string(Lexer& lexer);
    u8 parse_prec(Token token);
    ErrorOr<BinaryExpr::Kind> parse_bin_op_kind(Token token);
    ErrorOr<ref<Expr>> parse_bin_expr(Lexer& lexer, u8 prec);
    //ast::Type parse_type(Lexer& lexer);
};

Error parser_error(Token token, const char* message, ...);

}