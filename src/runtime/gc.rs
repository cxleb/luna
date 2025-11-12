use std::sync::{Mutex, MutexGuard};

static GC: Mutex<GarbageCollector> = Mutex::new(GarbageCollector::new());

trait Glob<'a> {
    fn access(&'a self) -> MutexGuard<'a, GarbageCollector>;
} 

impl<'a> Glob<'a> for Mutex<GarbageCollector> {
    fn access(&'a self) -> MutexGuard<'a, GarbageCollector> {
        self.lock().unwrap()
    }
}

struct GarbageCollector {

}

impl GarbageCollector {
    const fn new() -> Self {
        Self {

        }
    }
}

fn gc_init() {
    
}

fn gc_deinit() {
}

fn gc_collect() {
    //let gc =GC.access();
}

fn gc_alloc() {

}