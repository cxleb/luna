#pragma once

#include <unordered_map>
#include <vector>
#include <string>
#include "value.h"

namespace luna::runtime {

class Cell {
public:
    enum CellKind {
        KindString,
        KindObject,
    } kind;
    //virtual uint64_t hash() = 0;
    virtual bool equal(Cell* other) = 0;
};

class String : public Cell {
public:
    String(const std::string& str);
    inline const char* c_str() { return m_string.c_str(); }
    //virtual uint64_t hash() override;
    virtual bool equal(Cell* other) override;
private:
    std::string m_string;
};

class Object : public Cell {
public:
    Object();
    //virtual uint64_t hash() override;
    virtual bool equal(Cell* other) override;
    void set(int64_t key, Value eq);
    Value get(int64_t key);
private:
    std::unordered_map<int64_t, Value> map;
    /// ???
};

class Heap {
public:
    Cell* alloc_string(const std::string& str);
    Cell* alloc_object();
private:
    std::vector<Cell*> m_cells; 
};

}