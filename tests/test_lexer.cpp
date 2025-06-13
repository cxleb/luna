#include <cstring>
#include "testing.h"
#include "compiler/lexer.h"

using namespace luna::compiler;

#define ASSERT_TOKEN(token, string, ki) \
    TEST_ASSERT(token.kind != ki); \
    TEST_ASSERT(lexer.token_to_string(token).value() != string); \

#define TEST_SINGLE_TOKEN(string, ty) { \
    auto len = strlen(string); \
    Lexer lexer(to_source(string)); \
    auto token = lexer.next(); \
    ASSERT_TOKEN(token.value(), string, ty); \
    TEST_ASSERT(lexer.at != len); \
    TEST_ASSERT(lexer.col != len); \
}

int main(const int argc, const char** argv) {
    TEST_SINGLE_TOKEN("", TokenEndOfFile);
    TEST_SINGLE_TOKEN("^", TokenCaret);
    TEST_SINGLE_TOKEN("&", TokenAmpersand);
    TEST_SINGLE_TOKEN("*", TokenAstericks);
    TEST_SINGLE_TOKEN("+", TokenPlus);
    TEST_SINGLE_TOKEN("++", TokenPlusPlus);
    TEST_SINGLE_TOKEN("+=", TokenPlusEquals);
    TEST_SINGLE_TOKEN("-", TokenMinus);
    TEST_SINGLE_TOKEN("-=", TokenMinusEquals);
    TEST_SINGLE_TOKEN("--", TokenMinusMinus);
    TEST_SINGLE_TOKEN("=", TokenEquals);
    TEST_SINGLE_TOKEN("==", TokenEqualsEquals);
    TEST_SINGLE_TOKEN("!", TokenExclamation);
    TEST_SINGLE_TOKEN("!=", TokenExclamationEquals);
    TEST_SINGLE_TOKEN(":", TokenColon);
    TEST_SINGLE_TOKEN(";", TokenSemiColon);
    TEST_SINGLE_TOKEN(".", TokenDot);
    TEST_SINGLE_TOKEN(",", TokenComma);
    TEST_SINGLE_TOKEN("/", TokenForwardSlash);
    TEST_SINGLE_TOKEN("(", TokenLeftParen);
    TEST_SINGLE_TOKEN(")", TokenRightParen);
    TEST_SINGLE_TOKEN("[", TokenLeftBracket);
    TEST_SINGLE_TOKEN("]", TokenRightBracket);
    TEST_SINGLE_TOKEN("{", TokenLeftCurly);
    TEST_SINGLE_TOKEN("}", TokenRightCurly);
    TEST_SINGLE_TOKEN("<", TokenLessThen);
    TEST_SINGLE_TOKEN("<=", TokenLessThenEquals);
    TEST_SINGLE_TOKEN(">", TokenGreaterThen);
    TEST_SINGLE_TOKEN(">=", TokenGreaterThenEquals);
    TEST_SINGLE_TOKEN("hello", TokenIdentifier);
    TEST_SINGLE_TOKEN("hello1", TokenIdentifier);
    TEST_SINGLE_TOKEN("_hello1", TokenIdentifier);
    TEST_SINGLE_TOKEN("_he_l1lo1", TokenIdentifier);
    TEST_SINGLE_TOKEN("\"this is a string\"", TokenString);
    TEST_SINGLE_TOKEN("1234", TokenNumber);
    TEST_SINGLE_TOKEN("1234.5678", TokenNumber);

    // sequence of tokens is correct
    {
        Lexer lexer(to_source("ident1;1234"));
        auto token = lexer.next().value();
        ASSERT_TOKEN(token, "ident1", TokenIdentifier);
        token = lexer.next().value();
        ASSERT_TOKEN(token, ";", TokenSemiColon);
        token = lexer.next().value();
        ASSERT_TOKEN(token, "1234", TokenNumber);
    }

    // whitespace is correctly ignored
    {
        Lexer lexer(to_source(" \t\n\t  ident1  \t\t\t  ;   "));
        auto token = lexer.next().value();
        ASSERT_TOKEN(token, "ident1", TokenIdentifier);
        token = lexer.next().value();
        ASSERT_TOKEN(token, ";", TokenSemiColon);
    }

    // comments are correctly ignored
    {
        Lexer lexer(to_source("//this is a comment\nident1 // ident1 here is an identifier\n;"));
        auto token = lexer.next().value();
        ASSERT_TOKEN(token, "ident1", TokenIdentifier);
        token = lexer.next().value();
        ASSERT_TOKEN(token, ";", TokenSemiColon);
    }

    return 0;
}
