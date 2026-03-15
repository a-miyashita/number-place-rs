//! Difficulty evaluation for number-place puzzles.

mod human_solver;
mod scorer;
pub(crate) mod techniques;

use crate::puzzle::PuzzleDefinition;
use crate::types::Board;

/// Difficulty rank of a generated puzzle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DifficultyRank {
    /// Solvable using only Basic Singles (Naked/Hidden Single).
    Beginner,
    /// Requires Locked Candidates or Pair techniques.
    Intermediate,
    /// Requires Triple, Fish (size 2), Quad, Fish (size 3), or Wing techniques.
    Advanced,
    /// Cannot be solved by any of the above techniques; trial-and-error required.
    Expert,
}

/// Detailed difficulty evaluation result.
#[derive(Debug, Clone)]
pub struct DifficultyResult {
    /// Overall difficulty rank.
    pub rank: DifficultyRank,
    /// Technique score (TS): 0, 25, 45, 65, 80, or 100.
    pub technique_score: u32,
    /// Clue count score (CCS): 0.0..=100.0. Higher means fewer clues (harder).
    pub clue_count_score: f64,
    /// Total score = TS × 0.7 + CCS × 0.3.
    pub total_score: f64,
}

/// Evaluate the difficulty of an initial board (puzzle).
pub fn evaluate_difficulty(puzzle: &PuzzleDefinition, board: &Board) -> DifficultyResult {
    let (_solved, ts) = human_solver::human_solve(puzzle, board);
    let ccs = scorer::clue_count_score(puzzle, board);
    let total = scorer::total_score(ts, ccs);
    let rank = scorer::rank_from_technique_score(ts);

    DifficultyResult {
        rank,
        technique_score: ts,
        clue_count_score: ccs,
        total_score: total,
    }
}
