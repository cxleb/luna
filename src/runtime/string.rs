pub fn convert_to_interal_string(s: &str) -> Box<[u8]> {
    // A string is length + data contiguously in memory
    let mut bytes = Vec::with_capacity(8 + s.len());
    let len = s.len() as u64;
    bytes.extend_from_slice(&len.to_le_bytes());
    bytes.extend_from_slice(s.as_bytes());
    bytes.into_boxed_slice()
}

// Warning: extremely unsafe and no checking of any form.
pub fn convert_from_internal_string<'a>(ptr: *const u8) -> &'a str {
    unsafe {
        let length_slice = std::slice::from_raw_parts(ptr, 8);
        let length = usize::from_le_bytes(length_slice.try_into().unwrap());
        let value = ptr.add(8);
        let slice = std::slice::from_raw_parts(value, length);
        std::str::from_utf8_unchecked(slice)
    }
}