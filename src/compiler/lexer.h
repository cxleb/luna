#pragma once
#include <vector>
#include <string>
#include <stdint.h>

namespace luna::compiler {

#define TOKENS(A) \
    A(EndOfFile) \
    A(Caret) \
    A(Ampersand) \
    A(Astericks) \
    A(Plus) \
    A(PlusPlus) \
    A(PlusEquals) \
    A(Minus) \
    A(MinusMinus) \
    A(MinusEquals) \
    A(Equals) \
    A(EqualsEquals) \
    A(Colon) \
    A(SemiColon) \
    A(Dot) \
    A(Comma) \
    A(ForwardSlash) \
    A(LeftParen) \
    A(RightParen) \
    A(LeftBracket) \
    A(RightBracket) \
    A(LeftCurly) \
    A(RightCurly) \
    A(LessThen) \
    A(LessThenEquals) \
    A(GreaterThen) \
    A(GreaterThenEquals) \
    A(Exclamation) \
    A(ExclamationEquals) \
    A(Identifier) \
    A(String) \
    A(Number)

enum TokenKind {
#define A(name) Token##name,
    TOKENS(A)
#undef A
};

const char* get_token_name(TokenKind kind);

struct Token {
    TokenKind kind;
    uint64_t offset;
    uint64_t size;
    uint64_t line;
    uint64_t col;
};

struct Lexer {
    std::vector<char> source;
    uint64_t at, line, col;

    Lexer(std::vector<char>&& source);
    void eat_whitespace();
    Token peek();
    Token next();
    // Returns true if the token is a float
    bool is_token_int_or_float(Token token);
    void copy_token(char* buf, uint32_t size, Token token);
    double token_to_float(Token token);
    uint64_t token_to_int(Token token);
    std::string token_to_string(Token token);
    bool test(TokenKind kind);
    bool test(const std::string& str);
    Token expect(TokenKind kind);
};

}