#pragma once

#include <cstdio>
#include <stdarg.h>
#include <string>
#include <variant>

namespace luna {

#ifdef NDEBUG
#define luna_assert(cond) 
#else
#define luna_assert(cond) \
    if (!(cond)) { \
        fprintf(stderr, "assert failed %s in %s at %d\n", #cond, __FUNCTION__, __LINE__); \
        abort(); \
    } 
#endif

class Error {
public:
    Error(const std::string& msg) : m_msg(msg) {}
    std::string msg() { return m_msg; }
private:
    std::string m_msg;
};

Error error (const char* message, ...);
Error verror (const char* message, va_list args);

#define TRY(e) ({ \
    auto v = e; \
    if(v.is_error()) { \
        return v.error(); \
    } \
    v.value(); \
}) 

template<typename T>
class ErrorOr {
    public:
    ErrorOr(T value) : m_value(value), m_is_error(false) {}
    ErrorOr(Error error) : m_value(error), m_is_error(true) {}

    Error error() {
        luna_assert(m_is_error == true);
        return std::get<Error>(m_value);
    }

    T value() {
        luna_assert(m_is_error == false);
        return std::get<T>(m_value);
    }

    bool is_error() {
        return m_is_error;
    }
private:
    bool m_is_error;
    std::variant<T, Error> m_value;
};

}