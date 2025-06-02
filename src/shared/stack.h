#pragma once
#include <cstdint>
#include <cstring>
#include "error.h"

namespace luna {

// Basic stack implementation where pop() actually returns the popped element
template<typename T>
class Stack {
    class reverse_iterator {
    public:
        reverse_iterator(T* slice, uint64_t pos) : slice(slice), pos(pos) {

        }
        bool operator!=(reverse_iterator&& other) {
            return pos != other.pos;
        }
        bool operator++() {
            return pos--;
        }
        bool operator+=(uint64_t amt) {
            return pos -= amt;
        }
        T& operator*() {
            return slice[pos-1];
        }
    private:
        uint64_t pos;
        T* slice;
    };
    class iterator {
    public:
        iterator(T* slice, uint64_t pos) : slice(slice), pos(pos) {

        }
        bool operator!=(iterator& other) {
            return pos != other.pos;
        }
        bool operator++() {
            return pos++;
        }

        T& operator*() {
            return slice[pos];
        }
    private:
        uint64_t pos;
        T* slice;
    };
public:
    Stack() {
        top = 0;
        size = 0;
        data = nullptr;
    }
    Stack(Stack&) = default;
    Stack(Stack&&) = default;
    ~Stack() {
        if (size != 0) {
            remove();
        }
    }

    void push(T t) {
        ensure();
        data[top++] = t;    
    }

    T pop() {
        luna_assert(top != 0);
        return data[--top];
    }

    T& peak() {
        luna_assert(top != 0);
        return data[top - 1];
    }

    void clear() {
        top = 0;    
    }

    uint64_t count() {
        return top;
    }

    iterator begin() {
        return iterator(data, 0);
    }

    iterator end() {
        return iterator(data, top);
    }

    reverse_iterator rbegin() {
        return reverse_iterator(data, top);
    }

    reverse_iterator rend() {
        return reverse_iterator(data, 0);
    }
    
private:
    void ensure() {
        if (top >= size) {
            realloc();
        }
    }

    void realloc() {
        auto new_size = size * 2;
        if (new_size == 0) { new_size = 4; }
        auto new_data = new T[new_size];
        if (size != 0) {
            std::memcpy(new_data, data, size);
            remove();
        }
        size = new_size;
        data = new_data;
    }

    void remove() {
        luna_assert(data != nullptr);
        delete[] data;
        data = nullptr;
    }

private:
    uint64_t size;
    uint64_t top;
    T* data;
};

}