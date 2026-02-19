use super::{StackMap, StackMaps};

const RETURN_ADDRESS_LOOKBACK_WINDOW: usize = 16;

fn read_word(addr: usize) -> Option<usize> {
    if addr == 0 || !addr.is_multiple_of(core::mem::align_of::<usize>()) {
        return None;
    }

    Some(unsafe { *(addr as *const usize) })
}

fn find_stack_map<'a>(stack_maps: &'a StackMaps, return_address: usize) -> Option<&'a StackMap> {
    if let Some(stack_map) = stack_maps.lr_map.get(&return_address) {
        return Some(stack_map);
    }

    for delta in 1..=RETURN_ADDRESS_LOOKBACK_WINDOW {
        let key = return_address.checked_sub(delta)?;
        if let Some(stack_map) = stack_maps.lr_map.get(&key) {
            return Some(stack_map);
        }
    }

    None
}

pub fn collect_roots(stack_maps: &StackMaps, fp: usize) -> Vec<usize> {
    let Some(mut return_address) = read_word(fp + 8) else {
        return Vec::new();
    };
    let Some(mut fp) = read_word(fp) else {
        return Vec::new();
    };

    let mut stack_roots = Vec::new();

    while fp != 0 {
        if let Some(stack_map) = find_stack_map(stack_maps, return_address) {
            for offset in &stack_map.map {
                let addr = fp - stack_map.frame_to_fp_offset + (*offset as usize);
                if let Some(value) = read_word(addr) {
                    stack_roots.push(value);
                }
            }
        }

        let Some(next_return_address) = read_word(fp + 8) else {
            break;
        };

        let Some(next_fp) = read_word(fp) else {
            break;
        };

        return_address = next_return_address;
        fp = next_fp;
    }

    stack_roots
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{find_stack_map, StackMap, StackMaps};

    #[test]
    fn finds_stack_map_with_exact_return_address_match() {
        let mut lr_map = HashMap::new();
        lr_map.insert(
            0x1000,
            StackMap {
                map: vec![8],
                frame_to_fp_offset: 16,
            },
        );
        let stack_maps = StackMaps { lr_map };

        assert!(find_stack_map(&stack_maps, 0x1000).is_some());
    }

    #[test]
    fn finds_stack_map_with_small_return_address_delta() {
        let mut lr_map = HashMap::new();
        lr_map.insert(
            0x1000,
            StackMap {
                map: vec![8],
                frame_to_fp_offset: 16,
            },
        );
        let stack_maps = StackMaps { lr_map };

        assert!(find_stack_map(&stack_maps, 0x1005).is_some());
    }

    #[test]
    fn does_not_match_when_return_address_delta_is_too_large() {
        let mut lr_map = HashMap::new();
        lr_map.insert(
            0x1000,
            StackMap {
                map: vec![8],
                frame_to_fp_offset: 16,
            },
        );
        let stack_maps = StackMaps { lr_map };

        assert!(find_stack_map(&stack_maps, 0x1011).is_none());
    }
}
