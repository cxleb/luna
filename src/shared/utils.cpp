#include "utils.h"
#include <stdio.h>

std::optional<std::vector<char>> slerp(const std::string& path) {
    FILE* file;
#if WIN32
    auto err = fopen_s(&file, path.c_str(), "rb");
    if(err != 0) {
        return std::nullopt;
    }
    if (!file) {
        return std::nullopt;
    }
#else 
    file = fopen(path.c_str(), "rb");
    
    if (!file) {
        return std::nullopt;
    }
#endif
    fseek(file, 0, SEEK_END);
    auto size = ftell(file);
    rewind(file);

    std::vector<char> contents(size);

    auto read = fread(&contents[0], 1, size, file);
    
    return std::move(contents);
}