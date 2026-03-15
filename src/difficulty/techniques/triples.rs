//! T3: Naked/Hidden Triple and Fish (size=2, X-Wing equivalent).

use crate::solver::{PuzzleContext, SolverState};
use super::Technique;

/// T3: Naked/Hidden Triple + Fish (size=2).
pub(crate) struct Triples;

impl Technique for Triples {
    fn apply(&self, ctx: &PuzzleContext, state: &mut SolverState) -> bool {
        naked_triple(ctx, state) || hidden_triple(ctx, state) || fish2(ctx, state)
    }
}

fn naked_triple(ctx: &PuzzleContext, state: &mut SolverState) -> bool {
    let mut progress = false;

    for gi in 0..ctx.groups.len() {
        let group = ctx.groups[gi].clone();
        let empties: Vec<usize> = group
            .iter()
            .copied()
            .filter(|&ci| state.values[ci] == 0)
            .collect();

        let m = empties.len();
        for i in 0..m {
            for j in (i + 1)..m {
                for k in (j + 1)..m {
                    let ci = empties[i];
                    let cj = empties[j];
                    let ck = empties[k];

                    let union = state.masks[ci] | state.masks[cj] | state.masks[ck];
                    if union.count_ones() == 3 {
                        // Naked triple: remove these 3 values from other cells
                        for &co in &empties {
                            if co != ci && co != cj && co != ck {
                                let old = state.masks[co];
                                let new_mask = old & !union;
                                if new_mask != old {
                                    state.masks[co] = new_mask;
                                    progress = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    progress
}

fn hidden_triple(ctx: &PuzzleContext, state: &mut SolverState) -> bool {
    let mut progress = false;
    let n = ctx.group_size as u8;

    for gi in 0..ctx.groups.len() {
        let group = ctx.groups[gi].clone();

        for a in 1..=n {
            for b in (a + 1)..=n {
                for c in (b + 1)..=n {
                    let abit = 1u32 << a;
                    let bbit = 1u32 << b;
                    let cbit = 1u32 << c;
                    let triple_mask = abit | bbit | cbit;

                    let positions: Vec<usize> = group
                        .iter()
                        .copied()
                        .filter(|&ci| {
                            state.values[ci] == 0 && (state.masks[ci] & triple_mask) != 0
                        })
                        .collect();

                    if positions.len() == 3 {
                        // Restrict these 3 cells to only {a, b, c}
                        for &ci in &positions {
                            let old = state.masks[ci];
                            let new_mask = old & triple_mask;
                            if new_mask != old && new_mask != 0 {
                                state.masks[ci] = new_mask;
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

/// Fish pattern of size 2 (X-Wing equivalent) across any two groups.
fn fish2(ctx: &PuzzleContext, state: &mut SolverState) -> bool {
    let mut progress = false;
    let n = ctx.group_size as u8;

    // For each value v, try pairs of groups (g1, g2) as "base":
    // if all v-positions in g1 and g2 are covered by exactly 2 other "cover" groups,
    // remove v from other cells in those cover groups.
    for v in 1..=n {
        let vbit = 1u32 << v;

        for gi1 in 0..ctx.groups.len() {
            let pos1: Vec<usize> = ctx.groups[gi1]
                .iter()
                .copied()
                .filter(|&ci| state.values[ci] == 0 && (state.masks[ci] & vbit) != 0)
                .collect();

            if pos1.len() < 2 || pos1.len() > 2 {
                continue; // base group must have exactly 2 candidates for X-Wing
            }

            for gi2 in (gi1 + 1)..ctx.groups.len() {
                let pos2: Vec<usize> = ctx.groups[gi2]
                    .iter()
                    .copied()
                    .filter(|&ci| state.values[ci] == 0 && (state.masks[ci] & vbit) != 0)
                    .collect();

                if pos2.len() != 2 {
                    continue;
                }

                // Union of base positions
                let base_positions: std::collections::HashSet<usize> =
                    pos1.iter().chain(pos2.iter()).copied().collect();

                // Find cover groups: groups that contain all base positions
                // For X-Wing: we need exactly 2 cover groups
                let mut cover_groups: Vec<usize> = Vec::new();
                for gc in 0..ctx.groups.len() {
                    if gc == gi1 || gc == gi2 {
                        continue;
                    }
                    let gc_cells: std::collections::HashSet<usize> =
                        ctx.groups[gc].iter().copied().collect();
                    // A cover group must contain at least one base position
                    // AND all v-candidates in gc must be in the base positions
                    let gc_v_cells: Vec<usize> = ctx.groups[gc]
                        .iter()
                        .copied()
                        .filter(|&ci| state.values[ci] == 0 && (state.masks[ci] & vbit) != 0)
                        .collect();

                    if gc_v_cells.iter().all(|ci| base_positions.contains(ci))
                        && !gc_v_cells.is_empty()
                    {
                        cover_groups.push(gc);
                    }
                    let _ = gc_cells;
                }

                if cover_groups.len() == 2 {
                    // X-Wing: eliminate v from other cells in cover groups that are not in base
                    for &gc in &cover_groups {
                        for &ci in &ctx.groups[gc] {
                            if !base_positions.contains(&ci) && state.values[ci] == 0 {
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
    }

    progress
}
