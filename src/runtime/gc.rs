use std::collections::HashSet;

use libc::{malloc, free};

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
enum Allocation {
    Array(usize, usize),
    Object(usize, usize)
}

impl Allocation {
    pub fn get_address(&self) -> usize {
        match self {
            &Allocation::Array(p, _) => p,
            &Allocation::Object(p, _) => p,
        }
    }
}

pub struct GarbageCollector {
    allocations: HashSet<Allocation>
}

impl GarbageCollector {
    pub fn new() -> Self {
        Self {
            allocations: HashSet::new()
        }
    }

    pub fn should_collect(&mut self) -> bool {
        true
    }

    fn find_allocation(&self, ptr: usize) -> Option<&Allocation> {
        self.allocations.iter().filter(|&alloc| alloc.get_address() == ptr).next()
    }
    
    fn mark_allocation(&self, allocation: &Allocation, marks: &mut Vec<Allocation>) {
        // mark this allocation 
        marks.push(allocation.clone());

        // find all sub allocations and mark them
        match allocation {
            &Allocation::Array(p, s) => {
                for i in 0..s {
                    let v = (p + 8 + (i * 8)) as *mut usize;
                    let v = unsafe { *v };
                    if let Some(a) = self.find_allocation(v) {
                        self.mark_allocation(a, marks);
                    }
                }
            },
            &Allocation::Object(p, s) => {
                for i in 0..s {
                    let v = (p + (i * 8)) as *mut usize;
                    let v = unsafe { *v };
                    if let Some(a) = self.find_allocation(v) {
                        self.mark_allocation(a, marks);
                    }
                }
            },
        }
    }

    pub fn collect(&mut self, stack_roots: &Vec<usize>) {
        let mut marks = Vec::new();
        for &root in stack_roots.iter() {
            let alloc = self.find_allocation(root).expect("Stack root was not an allocation!");
            self.mark_allocation(alloc, &mut marks);
        }
        let to_remove = self.allocations.iter().filter(|a| !marks.contains(a)).cloned().collect::<Vec<_>>();
        
        for a in to_remove {
            self.allocations.remove(&a);
            Self::free_allocation(a);
        }
    }

    fn free_allocation(a: Allocation) {
        unsafe {
            free(a.get_address() as *mut libc::c_void);
        }
    }

    pub fn create_array(&mut self, size: usize) -> *const i64 {
        // Placeholder implementation
        // +8 for the size
        unsafe { 
            let ptr = malloc((8 * size) + 8) as *mut i64;
            *ptr = size as i64;
            self.allocations.insert(Allocation::Array(ptr as usize, size));
            ptr
        }
    }

    pub fn create_object(&mut self, size: usize) -> *const i64 {
        // Placeholder implementation
        // No need to encode size as it is fixed.
        unsafe { 
            let ptr = malloc(8 * size) as *const i64;
            self.allocations.insert(Allocation::Object(ptr as usize, size));
            ptr
        }
    }

}