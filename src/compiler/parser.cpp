#include "parser.h"
#include "ast.h"
#include "lexer.h"
#include "../shared/error.h"
#include "../shared/utils.h"
#include <cstdio>

namespace luna::compiler {

Error parser_error(Token token, const char* message, ...) {
    fprintf(stderr, "%llu:%llu: ", token.loc.line + 1, token.loc.col + 1);
    va_list args;
    va_start(args, message);
    auto err = verror(message, args);
    va_end(args);
    return err; 
}

Parser::Parser(std::vector<char>&& source) : lexer(std::move(source)) {
}

ErrorOr<ref<Module>> Parser::parse_module() {
    auto module = make_ref<Module>();

    auto token = lexer.peek();
    while(token.kind != TokenEndOfFile) {
        if (lexer.test("func")) {
            auto func = parse_func();
            CHECK(func);
            module->funcs.push_back(func.value());
        }
        token = lexer.peek();
    }

    return module;
}
    
ErrorOr<ref<Func>> Parser::parse_func() {
    luna_assert(lexer.test("func"));
    lexer.expect(TokenIdentifier);
    auto func = make_ref<Func>();
    auto func_name = lexer.expect(TokenIdentifier);
    CHECK(func_name);
    func->name = lexer.token_to_string(func_name.value());
    lexer.expect(TokenLeftParen);
    auto token = lexer.peek();
    while (token.kind != TokenRightParen) {
        Parameter param;
        auto param_name = lexer.expect(TokenIdentifier);
        CHECK(param_name);
        param.name = lexer.token_to_string(param_name.value());
        CHECK(lexer.expect(TokenColon));
        auto type = parse_type();
        CHECK(type);
        param.type = type.value(); 
        func->params.push_back(param);
        token = lexer.peek();
        if (token.kind == TokenComma) {
            lexer.next();
            token = lexer.peek();
        }
    }
    CHECK(lexer.expect(TokenRightParen));
    // If the next token is not a curly then its a type
    if (!lexer.test(TokenLeftCurly)) {
        auto type = parse_type();
        CHECK(type);
        func->return_type = type.value();
    }
    auto block = parse_block();
    CHECK(block);
    func->root = block.value();
    
    return func;
}

ErrorOr<ref<Stmt>> Parser::parse_stmt() {
    if (lexer.test("if")) {
        return parse_if();        
    } else if (lexer.test("while")) {
        return parse_while();        
    } else if (lexer.test("for")) {
        return parse_for();        
    } else if (lexer.test("return")) {
        return parse_return();
    } else if (lexer.test("let")) {
        return parse_var();
    } else if (lexer.test("const")) {
        return parse_var();
    } else {
        return parse_expr_stmt();
    }
}

ErrorOr<ref<Stmt>> Parser::parse_if() {
    auto stmt = make_node<If>();
    lexer.expect(TokenIdentifier);
    auto expr = parse_expr();
    CHECK(expr)
    stmt->condition = expr.value();
    auto then_stmt = parse_block();
    CHECK(then_stmt);
    stmt->then_stmt = then_stmt.value();
    if (lexer.test("else")) {
        lexer.next();
        if (lexer.test("if")) {
            auto else_if = parse_if();
            CHECK(else_if);
            stmt->else_stmt = else_if.value();
        } else {
            auto else_stmt = parse_block();
            CHECK(else_stmt);
            stmt->else_stmt = else_stmt.value();
        }
    } else {
        stmt->else_stmt = nullptr;
    }
    return finish_stmt(stmt);
}

ErrorOr<ref<Stmt>> Parser::parse_for() {
    auto stmt = make_node<For>();
    CHECK(lexer.expect(TokenIdentifier));
    auto name = lexer.expect(TokenIdentifier);
    CHECK(name);
    stmt->name = lexer.token_to_string(name.value());
    if(!lexer.test("in")) {
        return parser_error(lexer.next(), "Expected \'in\' in for statement");
    }
    CHECK(lexer.expect(TokenIdentifier));
    auto expr = parse_expr();
    CHECK(expr);
    stmt->iterator = expr.value();
    auto blk = parse_block();
    CHECK(blk);
    stmt->loop = blk.value();
    return finish_stmt(stmt);
}

ErrorOr<ref<Stmt>> Parser::parse_while() {
    auto stmt = make_node<While>();
    CHECK(lexer.expect(TokenIdentifier));
    auto expr = parse_expr();
    CHECK(expr);
    stmt->condition = expr.value();
    auto blk = parse_block();
    CHECK(blk);
    stmt->loop = blk.value();
    return finish_stmt(stmt);
}

ErrorOr<ref<Stmt>> Parser::parse_return() {
    auto stmt = make_node<Return>();
    CHECK(lexer.expect(TokenIdentifier));
    if (!lexer.test(TokenSemiColon)) {
        auto expr = parse_expr();
        CHECK(expr);
        stmt->value = expr.value();
    }
    lexer.expect(TokenSemiColon);
    return finish_stmt(stmt);
}

ErrorOr<ref<Stmt>> Parser::parse_var() {
    auto stmt = make_node<VarDecl>();
    if(lexer.test("const")) {
        stmt->is_const = true;
    } else {
        stmt->is_const = false;
    }
    lexer.expect(TokenIdentifier);
    auto name = lexer.expect(TokenIdentifier);
    CHECK(name);
    stmt->name = lexer.token_to_string(name.value());
    if (lexer.test(TokenColon)) {
        lexer.next();
        auto type = parse_type();
        CHECK(type);
        stmt->type = type.value();
    }
    CHECK(lexer.expect(TokenEquals));
    auto expr = parse_expr();
    CHECK(expr);
    stmt->value = expr.value();
    CHECK(lexer.expect(TokenSemiColon));
    return finish_stmt(stmt);
}

ErrorOr<ref<Stmt>> Parser::parse_expr_stmt() {
    auto expr_stmt = make_node<ExprStmt>();
    auto expr = parse_expr();
    CHECK(expr);
    expr_stmt->expr = expr.value();
    CHECK(lexer.expect(TokenSemiColon));
    return finish_stmt(expr_stmt);
}

ErrorOr<ref<Stmt>> Parser::parse_block() {
    auto block = make_node<Block>();
    CHECK(lexer.expect(TokenLeftCurly));
    while(!lexer.test(TokenRightCurly)) {
        auto stmt = parse_stmt();
        CHECK(stmt);
        block->stmts.push_back(stmt.value());
    }
    CHECK(lexer.expect(TokenRightCurly));
    return finish_stmt(block);
}

ErrorOr<ref<Expr>> Parser::parse_expr() {
    return parse_bin_expr(0);
}

ErrorOr<ref<Expr>> Parser::parse_primary_expr() {
    if (lexer.test(TokenIdentifier)) {
        return parse_ident();
    } else if (lexer.test(TokenNumber)) {
        return parse_number();
    } else if (lexer.test(TokenString)) {
        return parse_string();
    } else if (lexer.test(TokenLeftCurly)) {
        return parse_object_literal();
    } else if (lexer.test(TokenLeftBracket)) {
        return parse_array_literal();
    } else {
        return parser_error(lexer.peek(), 
            "Expected expression, found token: %s\n", 
            get_token_name(lexer.peek().kind));
    }
}

ErrorOr<ref<Expr>> Parser::parse_ident() {
    auto token = lexer.expect(TokenIdentifier);
    CHECK(token);
    auto name = lexer.token_to_string(token.value());
    if (lexer.test(TokenLeftParen)) {
        // parse function call
        auto call = make_node<Call>(token.value());
        call->name = name;
        CHECK(lexer.expect(TokenLeftParen));
        auto token = lexer.peek();
        while (token.kind != TokenRightParen) {
            auto expr = parse_expr();
            CHECK(expr);
            call->args.push_back(expr.value());
            token = lexer.peek();
            if (token.kind == TokenComma) {
                lexer.next();
                token = lexer.peek();
            }
        }
        CHECK(lexer.expect(TokenRightParen));
        return finish_expr(call);
    } else {
        auto expr = make_node<Identifier>(token.value());
        expr->name = name;
        return finish_expr(expr);
    }
}

ErrorOr<ref<Expr>> Parser::parse_number() {
    auto token = lexer.expect(TokenNumber);
    CHECK(token);
    if (lexer.is_token_int_or_float(token.value())) {
        auto expr = make_node<Float>(token.value());
        expr->value = lexer.token_to_float(token.value());
        return finish_expr(expr);
    } else {
        auto expr = make_node<Integer>(token.value());
        expr->value = lexer.token_to_int(token.value());
        return finish_expr(expr);
    }
}

ErrorOr<ref<Expr>> Parser::parse_string() {
    auto error_or_token = lexer.expect(TokenString);
    CHECK(error_or_token);
    auto token = error_or_token.value();
    auto expr = make_node<String>(token);
    // trim the leading and trailing quote marks
    token.loc.offset += 1;
    token.loc.size -= 2;
    expr->value = lexer.token_to_string(token);
    return finish_expr(expr);
}

ErrorOr<ref<Expr>> Parser::parse_object_literal() {
    auto expr = make_node<ObjectLiteral>();
    lexer.expect(TokenLeftCurly);
    lexer.expect(TokenRightCurly);
    return finish_expr(expr);
}

ErrorOr<ref<Expr>> Parser::parse_array_literal() {
    auto literal = make_node<ArrayLiteral>();
    lexer.expect(TokenLeftBracket);
    while(lexer.peek().kind != TokenRightBracket) {
        auto expr = parse_expr();
        CHECK(expr);
        literal->elements.push_back(expr.value());
        auto token = lexer.peek();
        if (token.kind == TokenComma) {
            lexer.next();
            token = lexer.peek();
        }
    }
    lexer.expect(TokenRightBracket);
    return finish_expr(literal);
}

u8 Parser::parse_prec(Token token) {
    switch(token.kind) {
        case TokenEqualsEquals:
        case TokenExclamationEquals:
        case TokenLessThen:
        case TokenGreaterThen:
        case TokenLessThenEquals:
        case TokenGreaterThenEquals:
            return 1;
        case TokenPlus:
        case TokenMinus:
            return 2;
        case TokenAstericks:
        case TokenForwardSlash:
            return 3;
        default:
            return 0;
    }
}

ErrorOr<BinaryExpr::Kind> Parser::parse_bin_op_kind(Token token) {
    switch(token.kind) {
        case TokenPlus:
            return BinaryExpr::KindAdd;
        case TokenMinus:
            return BinaryExpr::KindSubtract;
        case TokenAstericks:
            return BinaryExpr::KindMultiply;
        case TokenForwardSlash:
            return BinaryExpr::KindDivide;
        case TokenEqualsEquals:
            return BinaryExpr::KindEqual;
        case TokenExclamationEquals:
            return BinaryExpr::KindNotEqual;
        case TokenLessThen:
            return BinaryExpr::KindLessThan;
        case TokenGreaterThen:
            return BinaryExpr::KindGreaterThan;
        case TokenLessThenEquals:
            return BinaryExpr::KindLessThanEqual;
        case TokenGreaterThenEquals:
            return BinaryExpr::KindGreaterThanEqual;
        default:
            return parser_error(token, "Unknown binary operator");
    }
}

ErrorOr<ref<Expr>> Parser::parse_bin_expr(u8 prec) {
    auto error_or_lhs = parse_left_hand_side_expr();
    CHECK(error_or_lhs);
    auto lhs = error_or_lhs.value();
    while (true) {
        auto token = lexer.peek();
        u8 new_prec = parse_prec(token);
        if (new_prec <= prec) {
            break;
        }
        lexer.next();
        auto rhs = parse_bin_expr(new_prec);
        CHECK(rhs);
        auto expr = make_node<BinaryExpr>(token);
        auto kind = parse_bin_op_kind(token);
        CHECK(kind);
        expr->bin_kind = kind.value();
        expr->lhs = lhs;
        expr->rhs = rhs.value();
        lhs = expr;
    }
    return finish_expr(lhs);
}

ErrorOr<ref<Expr>> Parser::parse_left_hand_side_expr() {
    auto error_or_expr = parse_primary_expr();
    CHECK(error_or_expr);
    auto expr = error_or_expr.value();

    while (true) {
        if (lexer.test(TokenLeftBracket)) {
            CHECK(lexer.expect(TokenLeftBracket));
            auto index = parse_expr();
            CHECK(index);
            CHECK(lexer.expect(TokenRightBracket));
            auto lookup = make_node<Lookup>(expr->loc);
            lookup->expr = expr;
            lookup->index = index.value();
            expr = static_ref_cast<Expr>(lookup);
        }  else if (lexer.test(TokenEquals)) {
            auto assign = make_node<Assign>(expr->loc);
            assign->local = expr;
            CHECK(lexer.expect(TokenEquals));
            auto value = parse_expr();
            CHECK(value);
            assign->value = value.value();
            return static_ref_cast<Expr>(assign);
        } else {
            return expr;
        }
    }
}


ErrorOr<ref<Type>> Parser::parse_type() {
    // Type type{};
    // type.array_count = 0;
    // type.kind = TypeUnknown;
    // Token token;

    // bool working = true;
    // while (lexer.test(TokenLeftBracket))
    // {
    //     lexer.next();
    //     CHECK(lexer.expect(TokenRightBracket));
    //     type.array_count += 1;
    // }

    // token = lexer.next();
    // if (token.kind == TokenIdentifier) {
    //     type.name = lexer.token_to_string(token);
    //     type.kind = TypeIdentifier;
    // } else {
    // }

    // if (type.name == "string") {
    //     type.kind = TypeString;
    // } else if (type.name == "bool") {
    //     type.kind = TypeBool;
    // } else if (type.name == "int") {
    //     type.kind = TypeInteger;
    // } else if (type.name == "number") {
    //     type.kind = TypeNumber;
    // }

    // return type;

    if (lexer.test(TokenLeftBracket)) {
        lexer.next();
        CHECK(lexer.expect(TokenRightBracket));
        auto element_type = parse_type();
        CHECK(element_type);
        return array_type(element_type.value());
    } else if (lexer.test("string")) {
        lexer.next();
        return string_type();
    } else if (lexer.test("bool")) {
        lexer.next();
        return bool_type();
    } else if (lexer.test("int")) {
        lexer.next();
        return int_type();
    } else if(lexer.test("number")) {
        lexer.next();
        return number_type();
    } else {
        return parser_error(lexer.next(), "Unexpected token when defining a type");
    }
}

}