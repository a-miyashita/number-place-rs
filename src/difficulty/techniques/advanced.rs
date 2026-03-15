//! T4: Naked/Hidden Quad, Fish (size=3, Swordfish), and Wing patterns.

use crate::solver::{PuzzleContext, SolverState};
use super::Technique;

/// T4: Advanced techniques (Naked/Hidden Quad + Fish size=3 + Wing patterns).
pub(crate) struct Advanced;

impl Technique for Advanced {
    fn apply(&self, ctx: &PuzzleContext, state: &mut SolverState) -> bool {
        naked_quad(ctx, state)
            || hidden_quad(ctx, state)
            || fish3(ctx, state)
            || xy_wing(ctx, state)
    }
}

fn naked_quad(ctx: &PuzzleContext, state: &mut SolverState) -> bool {
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
                    for l in (k + 1)..m {
                        let ci = empties[i];
                        let cj = empties[j];
                        let ck = empties[k];
                        let cl = empties[l];

                        let union = state.masks[ci]
                            | state.masks[cj]
                            | state.masks[ck]
                            | state.masks[cl];
                        if union.count_ones() == 4 {
                            for &co in &empties {
                                if co != ci && co != cj && co != ck && co != cl {
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
    }

    progress
}

fn hidden_quad(ctx: &PuzzleContext, state: &mut SolverState) -> bool {
    let mut progress = false;
    let n = ctx.group_size as u8;

    for gi in 0..ctx.groups.len() {
        let group = ctx.groups[gi].clone();

        for a in 1..=n {
            for b in (a + 1)..=n {
                for c in (b + 1)..=n {
                    for d in (c + 1)..=n {
                        let mask_abcd = (1u32 << a) | (1u32 << b) | (1u32 << c) | (1u32 << d);

                        let positions: Vec<usize> = group
                            .iter()
                            .copied()
                            .filter(|&ci| {
                                state.values[ci] == 0 && (state.masks[ci] & mask_abcd) != 0
                            })
                            .collect();

                        if positions.len() == 4 {
                            for &ci in &positions {
                                let old = state.masks[ci];
                                let new_mask = old & mask_abcd;
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
    }

    progress
}

/// Swordfish (Fish size=3): 3 base groups, 3 cover groups.
fn fish3(ctx: &PuzzleContext, state: &mut SolverState) -> bool {
    let mut progress = false;
    let n = ctx.group_size as u8;

    for v in 1..=n {
        let vbit = 1u32 << v;

        // Collect groups that have 2 or 3 candidates for v
        let base_candidates: Vec<(usize, Vec<usize>)> = (0..ctx.groups.len())
            .filter_map(|gi| {
                let pos: Vec<usize> = ctx.groups[gi]
                    .iter()
                    .copied()
                    .filter(|&ci| state.values[ci] == 0 && (state.masks[ci] & vbit) != 0)
                    .collect();
                if pos.len() >= 2 && pos.len() <= 3 {
                    Some((gi, pos))
                } else {
                    None
                }
            })
            .collect();

        let m = base_candidates.len();
        for i in 0..m {
            for j in (i + 1)..m {
                for k in (j + 1)..m {
                    let (gi1, ref pos1) = base_candidates[i];
                    let (gi2, ref pos2) = base_candidates[j];
                    let (gi3, ref pos3) = base_candidates[k];

                    let base_pos: std::collections::HashSet<usize> =
                        pos1.iter().chain(pos2.iter()).chain(pos3.iter()).copied().collect();

                    if base_pos.len() > 3 {
                        continue; // need at most 3 cover groups
                    }

                    // Find cover groups
                    let mut cover_groups: Vec<usize> = Vec::new();
                    for gc in 0..ctx.groups.len() {
                        if gc == gi1 || gc == gi2 || gc == gi3 {
                            continue;
                        }
                        let gc_v_cells: Vec<usize> = ctx.groups[gc]
                            .iter()
                            .copied()
                            .filter(|&ci| state.values[ci] == 0 && (state.masks[ci] & vbit) != 0)
                            .collect();

                        if !gc_v_cells.is_empty()
                            && gc_v_cells.iter().all(|ci| base_pos.contains(ci))
                        {
                            cover_groups.push(gc);
                        }
                    }

                    if cover_groups.len() == 3 {
                        // Swordfish elimination
                        for &gc in &cover_groups {
                            for &ci in &ctx.groups[gc] {
                                if !base_pos.contains(&ci) && state.values[ci] == 0 {
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
    }

    progress
}

/// XY-Wing pattern.
fn xy_wing(ctx: &PuzzleContext, state: &mut SolverState) -> bool {
    let mut progress = false;

    // Find pivot cells with exactly 2 candidates
    for pivot in 0..ctx.n_cells {
        if state.values[pivot] != 0 || state.masks[pivot].count_ones() != 2 {
            continue;
        }

        let pivot_mask = state.masks[pivot];
        // Extract the two candidate values from the pivot mask
        let mut bits = Vec::new();
        for val in 1..=(ctx.group_size as u8) {
            if pivot_mask & (1u32 << val) != 0 {
                bits.push(val);
            }
        }
        if bits.len() != 2 {
            continue;
        }
        let (va, vb) = (bits[0], bits[1]);

        // Find wing1: a cell in pivot's groups with candidates {va, vc} for some vc
        let pivot_peers: Vec<usize> = ctx.peers[pivot].clone();

        for &w1 in &pivot_peers {
            if state.values[w1] != 0 || state.masks[w1].count_ones() != 2 {
                continue;
            }
            let w1_mask = state.masks[w1];
            // w1 must share exactly one value with pivot: either va or vb
            let shared_with_pivot = w1_mask & pivot_mask;
            if shared_with_pivot.count_ones() != 1 {
                continue;
            }

            let shared_val = shared_with_pivot.trailing_zeros() as u8;
            let wing_val = {
                let mut v = 0u8;
                for val in 1..=(ctx.group_size as u8) {
                    if w1_mask & (1u32 << val) != 0 && val != shared_val {
                        v = val;
                        break;
                    }
                }
                v
            };
            if wing_val == 0 {
                continue;
            }

            // The other value pivot shares with w1 that is NOT shared_val
            let other_pivot_val = if shared_val == va { vb } else { va };

            // Find wing2: a cell in pivot's groups (different group from w1) with candidates {other_pivot_val, wing_val}
            let w2_mask_expected = (1u32 << other_pivot_val) | (1u32 << wing_val);

            for &w2 in &pivot_peers {
                if w2 == w1 {
                    continue;
                }
                if state.values[w2] != 0 {
                    continue;
                }
                if state.masks[w2] != w2_mask_expected {
                    continue;
                }

                // XY-Wing found: eliminate wing_val from cells that see both w1 and w2
                let wing_bit = 1u32 << wing_val;
                let w1_peers: std::collections::HashSet<usize> =
                    ctx.peers[w1].iter().copied().collect();
                let w2_peers: std::collections::HashSet<usize> =
                    ctx.peers[w2].iter().copied().collect();

                for &ci in w1_peers.intersection(&w2_peers) {
                    if ci != w1 && ci != w2 && state.values[ci] == 0 {
                        if state.masks[ci] & wing_bit != 0 {
                            state.masks[ci] &= !wing_bit;
                            progress = true;
                        }
                    }
                }
            }
        }

    }

    progress
}
