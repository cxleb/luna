use libc::malloc;

pub struct GarbageCollector {

}

impl GarbageCollector {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn collect(&mut self) {
    }

    pub fn create_array(&mut self, size: usize) -> *const i64 {
        // Placeholder implementation
        // +8 for the size
        unsafe { 
            let ptr = malloc((8 * size) + 8) as *mut i64;
            *ptr = size as i64;
            ptr
        }
    }
}