#pragma once

#include <vector>

namespace luna::runtime {

class Cell {
public:
    enum CellKind {
        KindString,
        KindObject,
    } kind;
};

class String : public Cell {
public:
    String(const std::string& str);
    inline const char* c_str() { return m_string.c_str(); }
private:
    std::string m_string;
};

class Object : public Cell {
public:
    Object();
private:
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