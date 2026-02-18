use super::StackMaps;

fn read_word(addr: usize) -> Option<usize> {
    if addr == 0 || !addr.is_multiple_of(core::mem::align_of::<usize>()) {
        return None;
    }

    Some(unsafe { *(addr as *const usize) })
}

pub fn collect_roots(stack_maps: &StackMaps, fp: usize) -> Vec<usize> {
    let Some(mut lr) = read_word(fp + 8) else {
        return Vec::new();
    };
    let Some(mut fp) = read_word(fp) else {
        return Vec::new();
    };

    let mut stack_roots = Vec::new();

    while fp != 0 {
        if let Some(stack_map) = stack_maps.lr_map.get(&lr) {
            for offset in &stack_map.map {
                let addr = fp - stack_map.frame_to_fp_offset + (*offset as usize);
                if let Some(value) = read_word(addr) {
                    stack_roots.push(value);
                }
            }
        }

        let Some(next_lr) = read_word(fp + 8) else {
            break;
        };

        let Some(next_fp) = read_word(fp) else {
            break;
        };

        lr = next_lr;
        fp = next_fp;
    }

    stack_roots
}
