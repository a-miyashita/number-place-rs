//! Solver: constraint propagation + MRV backtracking.

mod candidates;
mod propagate;
mod backtrack;

pub(crate) use candidates::{PuzzleContext, SolverState};
pub(crate) use backtrack::backtrack_rand;

use crate::puzzle::PuzzleDefinition;
use crate::types::{Board, SolverError};

/// Validate that all placed values are in range and conflict-free.
pub fn validate_board(puzzle: &PuzzleDefinition, board: &Board) -> bool {
    let n = puzzle.group_size;
    // Check value range
    for &v in board.values() {
        if v < 1 || v as usize > n {
            return false;
        }
    }
    // Check each group for duplicates
    for group in &puzzle.groups {
        let mut seen = 0u32;
        for coord in group {
            if let Some(&v) = board.get(coord) {
                let bit = 1u32 << v;
                if seen & bit != 0 {
                    return false; // duplicate
                }
                seen |= bit;
            }
        }
    }
    true
}

/// Try to determine solution uniqueness via constraint propagation alone.
///
/// Returns:
/// - `Some(true)`  – propagation fully solved the board (unique solution confirmed)
/// - `Some(false)` – propagation found a contradiction (no solution exists)
/// - `None`        – propagation could not finish; a full search is required
pub(crate) fn propagation_check(ctx: &PuzzleContext, board: &Board) -> Option<bool> {
    let mut state = SolverState::new(ctx, board)?;

    if !run_propagation(ctx, &mut state) {
        return Some(false);
    }

    if state.n_filled == ctx.n_cells {
        Some(true)
    } else {
        None
    }
}

/// Run naked-singles + hidden-singles propagation to a fixpoint.
/// Returns `false` on contradiction, `true` otherwise.
fn run_propagation(ctx: &PuzzleContext, state: &mut SolverState) -> bool {
    loop {
        let before = state.n_filled;

        // Naked singles: place cells whose candidate mask has exactly one bit set.
        for ci in 0..ctx.n_cells {
            if state.values[ci] != 0 {
                continue;
            }
            let m = state.masks[ci];
            if m == 0 {
                return false; // no candidates left
            }
            if m.count_ones() == 1 {
                let v = m.trailing_zeros() as u8;
                if !propagate::place(ctx, state, ci, v) {
                    return false;
                }
            }
        }

        // Hidden singles: for each group, find values with exactly one candidate cell.
        for gi in 0..ctx.groups.len() {
            let group = &ctx.groups[gi];
            for w in 1..=(ctx.group_size as u8) {
                let wbit = 1u32 << w;
                if group.iter().any(|&c| state.values[c] == w) {
                    continue; // already placed in this group
                }
                let mut only = None;
                let mut cnt = 0usize;
                for &c in group {
                    if state.values[c] == 0 && state.masks[c] & wbit != 0 {
                        cnt += 1;
                        only = Some(c);
                        if cnt > 1 {
                            break;
                        }
                    }
                }
                if cnt == 0 {
                    return false; // value has no valid cell
                }
                if cnt == 1 {
                    let c = only.unwrap();
                    if state.values[c] == 0 {
                        if !propagate::place(ctx, state, c, w) {
                            return false;
                        }
                    }
                }
            }
        }

        // Naked pairs: two cells in a group share the same two candidates →
        // eliminate those two values from every other cell in that group.
        for gi in 0..ctx.groups.len() {
            let group = &ctx.groups[gi];
            // Collect (mask, cell_index) for empty cells with exactly 2 candidates.
            let mut two_cands: [(u32, usize); 16] = [(0, 0); 16];
            let mut n_two = 0usize;
            for &c in group {
                if state.values[c] != 0 { continue; }
                let m = state.masks[c];
                if m.count_ones() == 2 {
                    two_cands[n_two] = (m, c);
                    n_two += 1;
                }
            }
            'pair: for i in 0..n_two {
                for j in (i + 1)..n_two {
                    if two_cands[i].0 != two_cands[j].0 { continue; }
                    let pair_mask = two_cands[i].0;
                    let ci = two_cands[i].1;
                    let cj = two_cands[j].1;
                    for &ck in group {
                        if ck == ci || ck == cj || state.values[ck] != 0 { continue; }
                        let old = state.masks[ck];
                        let new_m = old & !pair_mask;
                        if new_m == old { continue; }
                        if new_m == 0 { return false; }
                        state.masks[ck] = new_m;
                        if new_m.count_ones() == 1 {
                            let v = new_m.trailing_zeros() as u8;
                            if !propagate::place(ctx, state, ck, v) { return false; }
                            // Masks may have changed; restart this group's pair scan.
                            continue 'pair;
                        }
                    }
                }
            }
        }

        // Hidden pairs: two values in a group can only go in the same two cells →
        // eliminate all other candidates from those two cells.
        for gi in 0..ctx.groups.len() {
            let group = &ctx.groups[gi];
            // pos_mask[w] = bitmask of group-positions (0-based) where value w is a candidate.
            let mut pos_mask = [0u32; 32];
            for (pos, &c) in group.iter().enumerate() {
                if state.values[c] != 0 { continue; }
                let m = state.masks[c];
                let mut bits = m >> 1; // bit k-1 set means value k is candidate
                let mut w = 1u8;
                while bits != 0 {
                    if bits & 1 != 0 {
                        pos_mask[w as usize] |= 1u32 << pos;
                    }
                    bits >>= 1;
                    w += 1;
                }
            }
            for v1 in 1..=(ctx.group_size as u8) {
                let pm1 = pos_mask[v1 as usize];
                if pm1.count_ones() != 2 { continue; }
                for v2 in (v1 + 1)..=(ctx.group_size as u8) {
                    if pos_mask[v2 as usize] != pm1 { continue; }
                    // Hidden pair (v1, v2) occupies exactly the two positions in pm1.
                    let keep_mask = (1u32 << v1) | (1u32 << v2);
                    for (pos, &c) in group.iter().enumerate() {
                        if pm1 & (1u32 << pos) == 0 { continue; }
                        if state.values[c] != 0 { continue; }
                        let old = state.masks[c];
                        let new_m = old & keep_mask;
                        if new_m == old { continue; }
                        if new_m == 0 { return false; }
                        state.masks[c] = new_m;
                        if new_m.count_ones() == 1 {
                            let v = new_m.trailing_zeros() as u8;
                            if !propagate::place(ctx, state, c, v) { return false; }
                        }
                    }
                }
            }
        }

        if state.n_filled == before {
            break; // fixpoint reached
        }
    }
    true
}

/// Solve a puzzle. Returns `Ok(solution)` for a unique solution, or an error.
pub fn solve(puzzle: &PuzzleDefinition, board: &Board) -> Result<Board, SolverError> {
    // Quick structural validation
    if !validate_board(puzzle, board) {
        return Err(SolverError::InvalidBoard);
    }

    let ctx = PuzzleContext::new(&puzzle.groups, puzzle.group_size);

    let state = match SolverState::new(&ctx, board) {
        Some(s) => s,
        None => return Err(SolverError::InvalidBoard),
    };

    // Search for up to 2 solutions to distinguish Ok / MultipleSolutions
    let mut solutions: Vec<Vec<u8>> = Vec::new();
    backtrack::backtrack_det(&ctx, state, 2, &mut solutions);

    match solutions.len() {
        0 => Err(SolverError::NoSolution),
        1 => {
            let vals = &solutions[0];
            let result: Board = ctx
                .cells
                .iter()
                .enumerate()
                .map(|(i, &coord)| (coord, vals[i]))
                .collect();
            Ok(result)
        }
        _ => Err(SolverError::MultipleSolutions),
    }
}
