use super::StackMaps;

pub fn collect_roots(stack_maps: &StackMaps, fp: usize) -> Vec<usize> {
    // This dereferences what the FP register is pointing to,
    // which should be the previous frame pointer
    let mut lr = unsafe { *((fp + 8) as *const usize) };
    let mut fp = unsafe { *((fp) as *const usize) };
    
    let mut stack_roots = Vec::new();

    while fp != 0 {
        if let Some(stack_map) = stack_maps.lr_map.get(&(lr as usize)) {
            for offset in &stack_map.map {
                let addr = fp - stack_map.frame_to_fp_offset + (*offset as usize);
                let value = unsafe { *((addr) as *const usize) };
                stack_roots.push(value);
            }
        } else {
        }

        lr = unsafe { *((fp + 8) as *const usize) };
        fp = unsafe { *((fp) as *const usize) };
    }

    stack_roots
}