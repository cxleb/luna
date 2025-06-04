#include "utils.h"
#include <stdio.h>

std::optional<std::vector<char>> slerp(const std::string& path) {
    FILE* file;
    auto err = fopen_s(&file, path.c_str(), "rb");
    if(err != 0) {
        return std::nullopt;
    }
    if (!file) {
        return std::nullopt;
    }
    
    fseek(file, 0, SEEK_END);
    auto size = ftell(file);
    rewind(file);

    std::vector<char> contents(size);

    auto read = fread(&contents[0], 1, size, file);
    
    return std::move(contents);
}