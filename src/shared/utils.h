#pragma once

#include <memory>
#include <optional>
#include <vector>
#include <string>
#include <stdint.h>

using u8 = uint8_t;
using u16 = uint16_t;
using u32 = uint32_t;
using u64 = uint64_t;
using i8 = int8_t;
using i16 = int16_t;
using i32 = int32_t;
using i64 = int64_t;

template<typename T>
using ref = std::shared_ptr<T>;

template<typename T, typename ... Args>
constexpr ref<T> make_ref(Args&& ... args)
{
    return std::make_shared<T>(std::forward<Args>(args)...);
}

template<typename T, typename U>
constexpr ref<T> static_ref_cast(ref<U> ptr)
{
    return std::static_pointer_cast<T>(ptr);
}

std::optional<std::vector<char>> slerp(const std::string& path);