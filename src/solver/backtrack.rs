//! MRV-based backtracking search with optional randomised candidate ordering.

use super::candidates::{PuzzleContext, SolverState};
use super::propagate::place;

/// Pick the unfilled cell with the fewest candidates (MRV heuristic).
/// Returns `None` if all cells are filled.
fn pick_mrv(ctx: &PuzzleContext, state: &SolverState) -> Option<usize> {
    let mut best_ci: Option<usize> = None;
    let mut best_count = u32::MAX;

    for ci in 0..ctx.n_cells {
        if state.values[ci] == 0 {
            let cnt = state.masks[ci].count_ones();
            if cnt == 0 {
                return None; // contradiction
            }
            if cnt < best_count {
                best_count = cnt;
                best_ci = Some(ci);
            }
        }
    }
    best_ci
}

/// Non-randomised backtracking. Returns up to `limit` solutions.
pub(crate) fn backtrack_det(
    ctx: &PuzzleContext,
    state: SolverState,
    limit: usize,
    solutions: &mut Vec<Vec<u8>>,
) {
    if solutions.len() >= limit {
        return;
    }

    if state.n_filled == ctx.n_cells {
        solutions.push(state.values.clone());
        return;
    }

    let ci = match pick_mrv(ctx, &state) {
        Some(ci) => ci,
        None => return, // contradiction
    };

    let mask = state.masks[ci];
    let mut bit = 1u32;
    for v in 1..=(ctx.group_size as u8) {
        bit <<= 1; // bit = 1 << v
        if mask & bit != 0 {
            let mut new_state = state.clone();
            if place(ctx, &mut new_state, ci, v) {
                backtrack_det(ctx, new_state, limit, solutions);
                if solutions.len() >= limit {
                    return;
                }
            }
        }
    }
}

/// Randomised backtracking for puzzle generation (shuffles candidate order).
pub(crate) fn backtrack_rand(
    ctx: &PuzzleContext,
    state: SolverState,
    rng: &mut impl rand::Rng,
) -> Option<Vec<u8>> {
    if state.n_filled == ctx.n_cells {
        return Some(state.values);
    }

    let ci = pick_mrv(ctx, &state)?;

    let mask = state.masks[ci];
    let mut candidates: Vec<u8> = (1..=(ctx.group_size as u8))
        .filter(|&v| mask & (1u32 << v) != 0)
        .collect();

    // Shuffle candidate order for randomness
    use rand::seq::SliceRandom;
    candidates.shuffle(rng);

    for v in candidates {
        let mut new_state = state.clone();
        if place(ctx, &mut new_state, ci, v) {
            if let Some(result) = backtrack_rand(ctx, new_state, rng) {
                return Some(result);
            }
        }
    }
    None
}
