#include "heap.h"
#include "runtime/value.h"
#include <functional>

namespace luna::runtime {

String::String(const std::string& str) : m_string(str) {
    kind = Cell::KindString;
}

uint64_t String::hash() {
    auto hash = std::hash<std::string>()(m_string); 
    return hash;
}

bool String::equal(Cell* other) {
    if (other->kind != KindString) return false;
    auto* str = static_cast<String*>(other);
    return str->m_string == m_string;
}

Object::Object() {
    kind = Cell::KindObject;
}

uint64_t Object::hash() {
    uint64_t hash = 0;
    for(auto& a : map) {
        auto val_hash = value_hash(a.second);
        hash = hash ^ (val_hash << 1);
    }
    return hash;
}

bool Object::equal(Cell* other) {
    if (other->kind != KindObject) return false;
    auto* obj = static_cast<Object*>(other);
    return obj->map == map;
}


void Object::set(Value key, Value eq) {
    map.insert_or_assign(key, eq);
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