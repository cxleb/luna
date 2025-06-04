#include "heap.h"

namespace luna::runtime {

String::String(const std::string& str) : m_string(str) {
    kind = Cell::KindString;
}

Object::Object() {
    kind = Cell::KindString;
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