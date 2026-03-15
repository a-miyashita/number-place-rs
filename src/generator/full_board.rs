//! Generate a complete (fully filled) valid board using randomised backtracking.

use rand::seq::SliceRandom;
use crate::puzzle::PuzzleDefinition;
use crate::solver::{PuzzleContext, SolverState, backtrack_rand};
use crate::types::Board;

/// Generate a fully solved board using seed groups for diversity + randomised backtracking.
///
/// Seeds the independent groups with random permutations of 1..=N (subject to validity),
/// then fills the rest with randomised backtracking. Returns `None` on failure (retry).
pub(super) fn generate_full_board(
    puzzle: &PuzzleDefinition,
    seed_group_indices: &[usize],
    rng: &mut impl rand::Rng,
) -> Option<Board> {
    let ctx = PuzzleContext::new(&puzzle.groups, puzzle.group_size);
    let n = puzzle.group_size;

    // Try to seed the independent groups with random permutations.
    // If the seed produces an invalid board (duplicate in a row/col), retry.
    let mut seed_board: Board = Board::new();
    let mut symbols: Vec<u8> = (1..=(n as u8)).collect();

    for &gi in seed_group_indices {
        symbols.shuffle(rng);
        let group = &ctx.groups[gi];
        for (&ci, &v) in group.iter().zip(symbols.iter()) {
            let coord = ctx.cells[ci];
            seed_board.insert(coord, v);
        }
    }

    // Validate seed board — random permutations may conflict across groups.
    // Build state (also validates through peer removal).
    let state = SolverState::new(&ctx, &seed_board)?;

    // Check for validity: if the seed board has duplicates in any group, return None.
    if !crate::solver::validate_board(puzzle, &seed_board) {
        return None;
    }

    // Use randomised backtracking to fill remaining cells
    let result_vals = backtrack_rand(&ctx, state, rng)?;

    let board: Board = ctx
        .cells
        .iter()
        .enumerate()
        .map(|(i, &coord)| (coord, result_vals[i]))
        .collect();

    Some(board)
}
