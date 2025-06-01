cmake_minimum_required(VERSION 3.24)

project(luna)

# standard c++ cmake setup
set(CMAKE_BUILD_TYPE Debug)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_CXX_STANDARD 20)

set(luna_sources 
    src/shared/utils.cpp
    src/shared/error.cpp
    src/shared/builder.cpp
    src/shared/environment.cpp
    src/compiler/lexer.cpp
    src/compiler/ast.cpp
    src/compiler/parser.cpp
    src/compiler/gen.cpp
    src/runtime/bytecode.cpp
    src/runtime/value.cpp
    src/runtime/runtime.cpp
)

add_executable(luna 
    src/main.cpp
    ${luna_sources}
    )
target_include_directories(luna PRIVATE src)

#####################
# Build Tests

if(BUILD_TESTS)
    include(CTest)
    add_executable(luna-test-lexer tests/test_lexer.cpp ${luna_sources})
    target_include_directories(luna-test-lexer PRIVATE src)
    add_test(
        NAME luna-test-lexer
        COMMAND luna-test-lexer
    )
    add_executable(luna-test-parser tests/test_parser.cpp  ${luna_sources})
    target_include_directories(luna-test-parser PRIVATE src)
    add_test(
        NAME luna-test-parser
        COMMAND luna-test-parser
    )
    add_executable(luna-test-runtime tests/test_runtime.cpp  ${luna_sources})
    target_include_directories(luna-test-runtime PRIVATE src)
    add_test(
        NAME luna-test-runtime
        COMMAND luna-test-runtime
    )
    add_executable(luna-test-self tests/test_self.cpp  ${luna_sources})
    target_include_directories(luna-test-self PRIVATE src)
    add_test(
        NAME luna-test-self
        COMMAND luna-test-self 
    )
ENDIF()