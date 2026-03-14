use std::slice::Iter;

use crate::ir::{Block, Inst};

pub struct BlockIter<'a> {
    block: &'a Block,
    ins_iter: Iter<'a, Inst>,
    source_loc_iter: Iter<'a, (usize, usize)>,
    idx: usize,
    current_source_loc: Option<(usize, usize)>,
    next_source_loc: Option<(usize, usize)>
}

impl<'a> BlockIter<'a> {
    pub(super) fn new(block: &'a Block) -> Self {
        let ins_iter = block.ins.iter();
        let mut source_loc_iter = block.source_locs.iter();
        let current_source_loc = source_loc_iter.next().cloned();
        let next_source_loc = source_loc_iter.next().cloned();

        Self {
            block,
            ins_iter,
            source_loc_iter,
            idx: 0,
            current_source_loc,
            next_source_loc
        }
    }
}

impl<'a> Iterator for BlockIter<'a> {
    type Item = (&'a Inst, Option<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ins) = self.ins_iter.next() {
            if let Some((idx, _)) = self.next_source_loc {
                if self.idx >= idx {
                    self.current_source_loc = self.next_source_loc;
                    self.next_source_loc = self.source_loc_iter.next().cloned();
                }
            } 
            self.idx += 1; 
            Some((ins, self.current_source_loc.map(|(_, i)| i)))
        } else {
            None
        }
    }
}

mod tests {
    
    #[test]
    fn test_iter() {
        use crate::ir::{Inst, Signature, SourceLoc, SourceLocs, builder::FuncBuilder};
        
        let mut source_locs = SourceLocs { ..Default::default() };
        let mut builder = FuncBuilder::new("id".into(), Signature{
            parameters: Vec::new(),
            ret_types: Vec::new()
        }, &mut source_locs);

        let block = builder.new_block();
        builder.switch_to_block(block);
        builder.source_loc(SourceLoc {
            file: 0,
            line: 1,
            col: 0,
        });
        builder.load_const_int(10);
        builder.load_const_int(11);
        builder.load_const_int(12);
        builder.load_const_int(13);
        builder.source_loc(SourceLoc {
            file: 0,
            line: 2,
            col: 1,
        });
        builder.load_const_int(20);
        builder.load_const_int(21);
        builder.load_const_int(22);
        builder.load_const_int(23);

        let func = builder.finish();

        let iter = func.blocks[0].iter();

        let expected = vec![
            (Inst::LoadConstInt(10), SourceLoc{file: 0, line: 1, col: 0}),
            (Inst::LoadConstInt(11), SourceLoc{file: 0, line: 1, col: 0}),
            (Inst::LoadConstInt(12), SourceLoc{file: 0, line: 1, col: 0}),
            (Inst::LoadConstInt(13), SourceLoc{file: 0, line: 1, col: 0}),
            (Inst::LoadConstInt(20), SourceLoc{file: 0, line: 2, col: 1}),
            (Inst::LoadConstInt(21), SourceLoc{file: 0, line: 2, col: 1}),
            (Inst::LoadConstInt(22), SourceLoc{file: 0, line: 2, col: 1}),
            (Inst::LoadConstInt(23), SourceLoc{file: 0, line: 2, col: 1}),
        ];

        for ((g_i, g_sl), (e_i, e_sl)) in iter.zip(expected.iter()) {
            assert_eq!(g_i, e_i);
            assert_eq!(source_locs.locations[g_sl.unwrap()], *e_sl);
        }
    }
}