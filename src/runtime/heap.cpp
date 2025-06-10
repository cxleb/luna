#include "heap.h"
#include "runtime/value.h"
#include <functional>

namespace luna::runtime {

String::String(const std::string& str) : m_string(str) {
    kind = Cell::KindString;
}

uint64_t String::hash() {
    return std::hash<std::string>()(m_string);
};

Object::Object() {
    kind = Cell::KindString;
}

uint64_t Object::hash() {
    uint64_t hash = 0;
    for(auto& a : map) {
        auto val_hash = value_hash(a.second);
        hash = hash ^ (val_hash << 1);
    }
    return hash;
};

void Object::set(Value key, Value eq) {
    map[key] = eq;
}

Value Object::get(Value key) {
    return map[key];
}

Cell* Heap::alloc_string(const std::string& str) {
    auto* cell = new String(str);
    m_cells.push_back(cell);
    return cell;
}

Cell* Heap::alloc_object() {
    auto* cell = new Object();
    m_cells.push_back(cell);
    return cell;
}


}