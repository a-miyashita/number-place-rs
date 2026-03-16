//! Puzzle generator: produces a solvable initial board from a puzzle definition.

mod independent;
mod full_board;
mod removal;

use crate::difficulty::DifficultyRank;
use crate::puzzle::PuzzleDefinition;
use crate::types::{Board, GeneratorError};
use crate::solver::solve;

/// Symmetry mode for generated puzzles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symmetry {
    /// No symmetry requirement.
    None,
    /// 180-degree rotational symmetry.
    Rotation180,
    /// Left-right mirror symmetry.
    HorizontalMirror,
    /// Top-bottom mirror symmetry.
    VerticalMirror,
}

/// Constraints passed to the puzzle generator.
#[derive(Debug, Clone)]
pub struct GeneratorConstraints {
    /// Symmetry mode for hint positions.
    pub symmetry: Symmetry,
    /// Minimum number of hints (clues) in the generated puzzle.
    pub min_clues: Option<usize>,
    /// Maximum number of hints (clues) in the generated puzzle.
    pub max_clues: Option<usize>,
    /// Target difficulty rank (best-effort; may not always be achievable).
    pub target_difficulty: Option<DifficultyRank>,
}

impl Default for GeneratorConstraints {
    fn default() -> Self {
        GeneratorConstraints {
            symmetry: Symmetry::None,
            min_clues: None,
            max_clues: None,
            target_difficulty: None,
        }
    }
}

/// Generate a solvable puzzle satisfying the given constraints.
///
/// Pass any `rand::Rng` implementation; use a seeded RNG for reproducibility.
pub fn generate(
    puzzle: &PuzzleDefinition,
    constraints: &GeneratorConstraints,
    rng: &mut impl rand::Rng,
) -> Result<Board, GeneratorError> {
    let seed_groups = independent::find_max_independent_groups(puzzle);

    const MAX_RETRIES: usize = 1000;

    for _ in 0..MAX_RETRIES {
        if let Some(board) = attempt_once(puzzle, constraints, &seed_groups, rng) {
            return Ok(board);
        }
    }
    Err(GeneratorError::GenerationFailed)
}

/// Single generation attempt. Returns `Some(board)` on success, `None` on failure.
fn attempt_once(
    puzzle: &PuzzleDefinition,
    constraints: &GeneratorConstraints,
    seed_groups: &[usize],
    rng: &mut impl rand::Rng,
) -> Option<Board> {
    // Step 1: generate a full board
    let full_board = full_board::generate_full_board(puzzle, seed_groups, rng)?;

    // Step 2: remove cells
    let puzzle_board = removal::remove_cells(&full_board, puzzle, constraints, rng);

    // Step 3: check clue count constraints
    let clue_count = puzzle_board.len();
    if let Some(min) = constraints.min_clues {
        if clue_count < min {
            return None;
        }
    }
    if let Some(max) = constraints.max_clues {
        if clue_count > max {
            return None;
        }
    }

    // Step 4: verify uniqueness via solver
    solve(puzzle, &puzzle_board).ok().map(|_| puzzle_board)
}

