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
