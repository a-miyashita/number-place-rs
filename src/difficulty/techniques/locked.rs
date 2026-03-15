//! T1: Locked Candidates — pointing pairs / box-line reduction.

use crate::solver::{PuzzleContext, SolverState};
use super::Technique;

/// T1: Locked Candidates technique.
pub(crate) struct LockedCandidates;

impl Technique for LockedCandidates {
    fn apply(&self, ctx: &PuzzleContext, state: &mut SolverState) -> bool {
        let mut progress = false;

        for gi1 in 0..ctx.groups.len() {
            let group1 = ctx.groups[gi1].clone();
            for v in 1..=(ctx.group_size as u8) {
                let vbit = 1u32 << v;
                // Cells in group1 that can hold v
                let possible: Vec<usize> = group1
                    .iter()
                    .copied()
                    .filter(|&ci| state.values[ci] == 0 && (state.masks[ci] & vbit) != 0)
                    .collect();

                if possible.is_empty() || possible.len() > ctx.group_size {
                    continue;
                }

                // Check if all possible cells are also in some other group gi2
                for gi2 in 0..ctx.groups.len() {
                    if gi2 == gi1 {
                        continue;
                    }
                    let group2 = &ctx.groups[gi2];
                    let group2_set: std::collections::HashSet<usize> =
                        group2.iter().copied().collect();

                    if possible.iter().all(|ci| group2_set.contains(ci)) {
                        // Remove v from other cells in group2
                        let possible_set: std::collections::HashSet<usize> =
                            possible.iter().copied().collect();
                        for &ci in group2 {
                            if !possible_set.contains(&ci) && state.values[ci] == 0 {
                                if state.masks[ci] & vbit != 0 {
                                    state.masks[ci] &= !vbit;
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
}
