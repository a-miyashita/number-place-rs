//! T0: Basic Singles — Naked Single and Hidden Single.

use crate::solver::{PuzzleContext, SolverState};
use super::Technique;

/// T0: Basic Singles technique (Naked Single + Hidden Single).
pub(crate) struct BasicSingles;

impl Technique for BasicSingles {
    fn apply(&self, ctx: &PuzzleContext, state: &mut SolverState) -> bool {
        let mut progress = false;

        loop {
            let mut step = false;

            // Naked Singles: cell with exactly one candidate
            for ci in 0..ctx.n_cells {
                if state.values[ci] == 0 && state.masks[ci].count_ones() == 1 {
                    let v = state.masks[ci].trailing_zeros() as u8;
                    if place_value(ctx, state, ci, v) {
                        step = true;
                        progress = true;
                    }
                }
            }

            // Hidden Singles: a value has only one candidate cell in a group
            'outer: for gi in 0..ctx.groups.len() {
                let group = ctx.groups[gi].clone();
                for v in 1..=(ctx.group_size as u8) {
                    let vbit = 1u32 << v;
                    let candidates: Vec<usize> = group
                        .iter()
                        .copied()
                        .filter(|&ci| state.values[ci] == 0 && (state.masks[ci] & vbit) != 0)
                        .collect();
                    if candidates.len() == 1 {
                        let ci = candidates[0];
                        if place_value(ctx, state, ci, v) {
                            step = true;
                            progress = true;
                            break 'outer; // restart after any placement
                        }
                    }
                }
            }

            if !step {
                break;
            }
        }

        progress
    }
}

/// Place a value and update masks/peers.
fn place_value(ctx: &PuzzleContext, state: &mut SolverState, ci: usize, v: u8) -> bool {
    state.values[ci] = v;
    state.masks[ci] = 0;
    state.n_filled += 1;
    let bit = 1u32 << v;
    for &peer in &ctx.peers[ci] {
        if state.values[peer] == 0 {
            state.masks[peer] &= !bit;
        }
    }
    true
}
