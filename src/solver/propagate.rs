//! Constraint propagation: Naked Singles and Hidden Singles using a work queue.

use super::candidates::{PuzzleContext, SolverState};

/// Place value `v` at cell `ci` and run constraint propagation.
///
/// Returns `false` if a contradiction is detected.
pub(crate) fn place(
    ctx: &PuzzleContext,
    state: &mut SolverState,
    ci: usize,
    v: u8,
) -> bool {
    debug_assert_eq!(state.values[ci], 0, "Tried to place value in already-filled cell {}", ci);
    state.values[ci] = v;
    state.masks[ci] = 0;
    state.n_filled += 1;
    propagate(ctx, state, ci, v)
}

/// Propagate consequences of placing value `v` at cell `ci` using a work queue.
fn propagate(ctx: &PuzzleContext, state: &mut SolverState, start_ci: usize, start_v: u8) -> bool {
    // Work queue: (cell_index, value_to_place)
    let mut queue: std::collections::VecDeque<(usize, u8)> = std::collections::VecDeque::new();
    queue.push_back((start_ci, start_v));

    while let Some((ci, v)) = queue.pop_front() {
        let bit = 1u32 << v;

        // Remove v from all empty peers of ci
        for &peer in &ctx.peers[ci] {
            if state.values[peer] == 0 {
                let old = state.masks[peer];
                if old & bit == 0 {
                    continue; // v was not a candidate; nothing to do
                }
                let new_mask = old & !bit;
                if new_mask == 0 {
                    return false; // contradiction: no candidates left
                }
                state.masks[peer] = new_mask;
                // Naked Single: only one candidate remains
                if new_mask.count_ones() == 1 {
                    let nv = new_mask.trailing_zeros() as u8;
                    if !do_place(ctx, state, peer, nv, &mut queue) {
                        return false;
                    }
                }
            }
        }

        // Hidden Singles: check each group containing ci for values with only one candidate cell
        for &gi in &ctx.cell_groups[ci] {
            let group = &ctx.groups[gi];
            for w in 1..=(ctx.group_size as u8) {
                let wbit = 1u32 << w;
                // Check if w is already placed in this group
                let already = group.iter().any(|&gci| state.values[gci] == w);
                if already {
                    continue;
                }
                // Count empty cells that still have w as a candidate
                let mut only: Option<usize> = None;
                let mut count = 0usize;
                for &gci in group {
                    if state.values[gci] == 0 && (state.masks[gci] & wbit) != 0 {
                        count += 1;
                        only = Some(gci);
                        if count > 1 {
                            break;
                        }
                    }
                }
                if count == 0 {
                    return false; // no valid cell for w in this group
                }
                if count == 1 {
                    let target = only.unwrap();
                    if !do_place(ctx, state, target, w, &mut queue) {
                        return false;
                    }
                }
            }
        }
    }

    true
}

/// Place value `v` at cell `ci` (if not already placed) and add to the work queue.
fn do_place(
    _ctx: &PuzzleContext,
    state: &mut SolverState,
    ci: usize,
    v: u8,
    queue: &mut std::collections::VecDeque<(usize, u8)>,
) -> bool {
    if state.values[ci] != 0 {
        // Cell already placed — check consistency
        if state.values[ci] != v {
            return false; // contradiction: different values required
        }
        return true; // already placed with the same value, skip
    }
    // Verify v is still a candidate
    let bit = 1u32 << v;
    if state.masks[ci] & bit == 0 {
        return false; // v is no longer a candidate for ci
    }
    state.values[ci] = v;
    state.masks[ci] = 0;
    state.n_filled += 1;
    queue.push_back((ci, v));
    true
}

