//! T2: Naked Pair and Hidden Pair techniques.

use crate::solver::{PuzzleContext, SolverState};
use super::Technique;

/// T2: Naked/Hidden Pair technique.
pub(crate) struct Pairs;

impl Technique for Pairs {
    fn apply(&self, ctx: &PuzzleContext, state: &mut SolverState) -> bool {
        naked_pair(ctx, state) || hidden_pair(ctx, state)
    }
}

fn naked_pair(ctx: &PuzzleContext, state: &mut SolverState) -> bool {
    let mut progress = false;

    for gi in 0..ctx.groups.len() {
        let group = ctx.groups[gi].clone();
        let empties: Vec<usize> = group
            .iter()
            .copied()
            .filter(|&ci| state.values[ci] == 0)
            .collect();

        for i in 0..empties.len() {
            let ci = empties[i];
            let mask_i = state.masks[ci];
            if mask_i.count_ones() != 2 {
                continue;
            }
            for j in (i + 1)..empties.len() {
                let cj = empties[j];
                if state.masks[cj] == mask_i {
                    // Naked pair found: remove both bits from all other cells in group
                    for &ck in &empties {
                        if ck != ci && ck != cj {
                            let old = state.masks[ck];
                            let new_mask = old & !mask_i;
                            if new_mask != old {
                                state.masks[ck] = new_mask;
                                progress = true;
                            }
                        }
                    }
                }
            }
        }
    }

    progress
}

fn hidden_pair(ctx: &PuzzleContext, state: &mut SolverState) -> bool {
    let mut progress = false;
    let n = ctx.group_size as u8;

    for gi in 0..ctx.groups.len() {
        let group = ctx.groups[gi].clone();

        for a in 1..=n {
            for b in (a + 1)..=n {
                let abit = 1u32 << a;
                let bbit = 1u32 << b;

                // Cells that can hold a or b
                let positions: Vec<usize> = group
                    .iter()
                    .copied()
                    .filter(|&ci| {
                        state.values[ci] == 0
                            && ((state.masks[ci] & abit) != 0 || (state.masks[ci] & bbit) != 0)
                    })
                    .collect();

                if positions.len() == 2 {
                    // Both a and b are confined to exactly these 2 cells
                    let pair_mask = abit | bbit;
                    for &ci in &positions {
                        let old = state.masks[ci];
                        let new_mask = old & pair_mask;
                        if new_mask != old && new_mask != 0 {
                            state.masks[ci] = new_mask;
                            progress = true;
                        }
                    }
                }
            }
        }
    }

    progress
}
