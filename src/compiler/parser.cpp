#include "parser.h"
#include "ast.h"
#include "lexer.h"
#include "../shared/error.h"
#include "../shared/utils.h"
#include <cstdio>

namespace luna::compiler {

Error parser_error(Token token, const char* message, ...) {
    fprintf(stderr, "%llu:%llu: ", token.line + 1, token.col + 1);
    va_list args;
    va_start(args, message);
    auto err = verror(message, args);
    va_end(args);
    return err;  
}

ErrorOr<ref<Module>> Parser::parse_file(std::vector<char>&& source) {
    auto lexer = Lexer(std::move(source));
    
    auto module = make_ref<Module>();

    auto token = TRY(lexer.peek());
    while(token.kind != TokenEndOfFile) {
        if (TRY(lexer.test("func"))) {
            auto func = TRY(parse_func(lexer));
            module->funcs.push_back(func);
        }
        token = TRY(lexer.peek());
    }

    return module;
}

ErrorOr<ref<Func>> Parser::parse_func(Lexer& lexer) {
    luna_assert(TRY(lexer.test("func")));
    lexer.expect(TokenIdentifier);
    auto func = make_ref<Func>();
    func->name = TRY(lexer.token_to_string(TRY(lexer.expect(TokenIdentifier))));
    lexer.expect(TokenLeftParen);
    auto token = TRY(lexer.peek());
    while (token.kind != TokenRightParen) {
        Parameter param;
        param.name = TRY(lexer.token_to_string(TRY(lexer.expect(TokenIdentifier))));
        //lexer.expect(TokenColon);
        //param.type = parse_type(lexer);
        func->params.push_back(param);
        token = TRY(lexer.peek());
        if (token.kind == TokenComma) {
            lexer.next();
            token = TRY(lexer.peek());
        }
    }
    lexer.expect(TokenRightParen);
    //func->return_type = parse_type(lexer);
    func->root = TRY(parse_block(lexer));
    
    return func;
}

ErrorOr<ref<Stmt>> Parser::parse_stmt(Lexer& lexer) {
    if (TRY(lexer.test("if"))) {
        return parse_if(lexer);        
    } else if (TRY(lexer.test("while"))) {
        return parse_while(lexer);        
    } else if (TRY(lexer.test("for"))) {
        return parse_for(lexer);        
    } else if (TRY(lexer.test("return"))) {
        return parse_return(lexer);
    } else if (TRY(lexer.test("let"))) {
        return parse_var(lexer);
    } else if (TRY(lexer.test("const"))) {
        return parse_var(lexer);
    } else if (TRY(lexer.test(TokenIdentifier))) {
        return parse_stmt_ident(lexer);
    } else if (TRY(lexer.test(TokenLeftCurly))) {
        return parse_block(lexer);
    } else {
        return parser_error(TRY(lexer.peek()), "Expected statement\n");
    }
}

ErrorOr<ref<Stmt>> Parser::parse_if(Lexer& lexer) {
    lexer.expect(TokenIdentifier);
    auto stmt = make_ref<If>();
    auto expr = TRY(parse_expr(lexer));
    stmt = make_ref<If>();
    stmt->condition = expr;
    stmt->then_stmt = TRY(parse_block(lexer));
    if (TRY(lexer.test("else"))) {
        TRY(lexer.next());
        if (TRY(lexer.test("if"))) {
            stmt->else_stmt = TRY(parse_if(lexer));
        } else {
            stmt->else_stmt = TRY(parse_block(lexer));
        }
    } else {
        stmt->else_stmt = nullptr;
    }
    return static_ref_cast<Stmt>(stmt);
}

ErrorOr<ref<Stmt>> Parser::parse_for(Lexer& lexer) {
    auto stmt = make_ref<For>();
    TRY(lexer.expect(TokenIdentifier));
    stmt->name = TRY(lexer.token_to_string(TRY(lexer.expect(TokenIdentifier))));
    if(!TRY(lexer.test("in"))) {
        return parser_error(TRY(lexer.next()), "Expected \'in\' in for statement");
    }
    lexer.expect(TokenIdentifier);
    stmt->iterator = TRY(parse_expr(lexer));
    stmt->loop = TRY(parse_block(lexer));
    return static_ref_cast<Stmt>(stmt);
}

ErrorOr<ref<Stmt>> Parser::parse_while(Lexer& lexer) {
    auto stmt = make_ref<While>();
    TRY(lexer.expect(TokenIdentifier));
    stmt->condition = TRY(parse_expr(lexer));
    stmt->loop = TRY(parse_block(lexer));
    return static_ref_cast<Stmt>(stmt);
}

ErrorOr<ref<Stmt>> Parser::parse_return(Lexer& lexer) {
    TRY(lexer.expect(TokenIdentifier));
    auto stmt = make_ref<Return>();
    if (!TRY(lexer.test(TokenSemiColon))) {
        stmt->value = TRY(parse_expr(lexer));
    }
    lexer.expect(TokenSemiColon);
    return static_ref_cast<Stmt>(stmt);
}

ErrorOr<ref<Stmt>> Parser::parse_var(Lexer& lexer) {
    auto stmt = make_ref<VarDecl>();
    if(TRY(lexer.test("const"))) {
        stmt->is_const = true;
    } else {
        stmt->is_const = false;
    }
    lexer.expect(TokenIdentifier);
    stmt->name = TRY(lexer.token_to_string(TRY(lexer.expect(TokenIdentifier))));
    //if (lexer.test(TokenColon)) {
    //    lexer.next();
    //    //stmt->type = parse_type(lexer);
    //} else {
    //    stmt->type.is_unknown = true;
    //}
    TRY(lexer.expect(TokenEquals));
    stmt->value = TRY(parse_expr(lexer));
    TRY(lexer.expect(TokenSemiColon));
    return static_ref_cast<Stmt>(stmt);
}

ErrorOr<ref<Stmt>> Parser::parse_stmt_ident(Lexer& lexer) {
    auto name = TRY(lexer.token_to_string(TRY(lexer.expect(TokenIdentifier))));
    if (TRY(lexer.test(TokenLeftParen))) {
        // parse function call
        auto stmt = make_ref<Call>();
        stmt->name = name;
        TRY(lexer.expect(TokenLeftParen));
        auto token = TRY(lexer.peek());
        while (token.kind != TokenRightParen) {
            auto expr = TRY(parse_expr(lexer));
            stmt->args.push_back(expr);
            token = TRY(lexer.peek());
            if (token.kind == TokenComma) {
                lexer.next();
                token = TRY(lexer.peek());
            }
        }
        TRY(lexer.expect(TokenRightParen));
        TRY(lexer.expect(TokenSemiColon));
        return static_ref_cast<Stmt>(stmt);
    } else if (TRY(lexer.test(TokenEquals))) {
        auto stmt = make_ref<Assign>();
        stmt->name = name;
        TRY(lexer.expect(TokenEquals));
        stmt->value = TRY(parse_expr(lexer));
        TRY(lexer.expect(TokenSemiColon));
        return static_ref_cast<Stmt>(stmt);
    }
    return parser_error(TRY(lexer.next()), "Unexpected token after identifier");
}

ErrorOr<ref<Stmt>> Parser::parse_block(Lexer& lexer) {
    auto stmt = make_ref<Block>();
    TRY(lexer.expect(TokenLeftCurly));
    while(!TRY(lexer.test(TokenRightCurly))) {
        stmt->stmts.push_back(TRY(parse_stmt(lexer)));
    }
    TRY(lexer.expect(TokenRightCurly));
    return static_ref_cast<Stmt>(stmt);
}

ErrorOr<ref<Expr>> Parser::parse_expr(Lexer& lexer) {
    return parse_bin_expr(lexer, 0);
}

ErrorOr<ref<Expr>> Parser::parse_primary_expr(Lexer& lexer) {
    if (TRY(lexer.test(TokenIdentifier))) {
        return parse_ident(lexer);
    } else if (TRY(lexer.test(TokenNumber))) {
        return parse_number(lexer);
    } else if (TRY(lexer.test(TokenString))) {
        return parse_string(lexer);
    } else {
        return parser_error(TRY(lexer.peek()), 
            "Expected expression, found token: %s\n", 
            get_token_name(TRY(lexer.peek()).kind));
    }
}

ErrorOr<ref<Expr>> Parser::parse_ident(Lexer& lexer) {
    auto name = TRY(lexer.token_to_string(TRY(lexer.expect(TokenIdentifier))));
    if (TRY(lexer.test(TokenLeftParen))) {
        // parse function call
        auto call = make_ref<CallExpr>();
        call->call.name = name;
        TRY(lexer.expect(TokenLeftParen));
        auto token = TRY(lexer.peek());
        while (token.kind != TokenRightParen) {
            auto expr = TRY(parse_expr(lexer));
            call->call.args.push_back(expr);
            token = TRY(lexer.peek());
            if (token.kind == TokenComma) {
                lexer.next();
                token = TRY(lexer.peek());
            }
        }
        lexer.expect(TokenRightParen);
        return static_ref_cast<Expr>(call);
    } else {
        auto expr = make_ref<Identifier>();
        expr->name = name;
        return static_ref_cast<Expr>(expr);
    }
}

ErrorOr<ref<Expr>> Parser::parse_number(Lexer& lexer) {
    auto token = TRY(lexer.expect(TokenNumber));
    if (lexer.is_token_int_or_float(token)) {
        auto expr = make_ref<Float>();
        expr->value = TRY(lexer.token_to_float(token));
        return static_ref_cast<Expr>(expr);
    } else {
        auto expr = make_ref<Integer>();
        expr->value = TRY(lexer.token_to_int(token));
        return static_ref_cast<Expr>(expr);
    }
}

ErrorOr<ref<Expr>> Parser::parse_string(Lexer& lexer) {
    auto token = TRY(lexer.expect(TokenString));
    auto expr = make_ref<String>();
    expr->value = TRY(lexer.token_to_string(token));
    return static_ref_cast<Expr>(expr);
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

ErrorOr<ref<Expr>> Parser::parse_bin_expr(Lexer& lexer, u8 prec) {
    auto lhs = TRY(parse_primary_expr(lexer));
    while (true) {
        auto token = TRY(lexer.peek());
        u8 new_prec = parse_prec(token);
        if (new_prec <= prec) {
            break;
        }
        lexer.next();
        auto rhs = TRY(parse_bin_expr(lexer, new_prec));
        auto expr = make_ref<BinaryExpr>();
        expr->bin_kind = TRY(parse_bin_op_kind(token));
        expr->lhs = lhs;
        expr->rhs = rhs;
        lhs = expr;
    }
    return lhs;
}

// Type Parser::parse_type(Lexer& lexer) {
//     Type type;
//     type.is_unknown = false;
//     Token token;

//     bool working = true;
//     while (working)
//     {
//         token = lexer.next();
//         uint64_t size = 0;
//         bool specified = false;
//         switch(token.kind) {
//         case TokenAstericks:
//             type.specs.push_back(Type::Spec{
//                 .kind = Type::Pointer,
//             });
//             break;
//         case TokenAmpersand:
//             type.specs.push_back(Type::Spec{
//                 .kind = Type::Reference,
//             });
//             break;
//         case TokenLeftBracket:
//             token = lexer.next();
//             if (token.kind == TokenNumber) {
//                 size = lexer.token_to_int(token);
//                 token = lexer.next();
//                 specified = true;
//             }
//             if (token.kind != TokenRightBracket) {
//                 parser_error(token, "For array, you need close with ']'");
//             }            
//             type.specs.push_back(Type::Spec{
//                 .kind = Type::Array,
//                 .specified = specified,
//                 .size = size,
//             });
//             break;
//         default:
//             working = false;
//             break;
//         }
//     }

//     if (token.kind == TokenIdentifier) {
//         type.name = lexer.token_to_string(token);
//     } else {
//         parser_error(token, "Expected identifier when defining a type");
//     }

//     return type;
// }

}