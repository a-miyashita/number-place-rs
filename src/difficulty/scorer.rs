//! Score calculation for difficulty evaluation.

use crate::puzzle::PuzzleDefinition;
use crate::types::Board;
use super::DifficultyRank;

/// Theoretical minimum clues for known puzzle sizes.
fn theoretical_min_clues(total_cells: usize) -> usize {
    if total_cells == 81 {
        17 // 9×9
    } else if total_cells == 256 {
        55 // 16×16
    } else {
        0 // fallback: use 0 so CCS = 100 * (total - clues) / total
    }
}

/// Compute the Clue Count Score (CCS).
///
/// `CCS = clamp((total_cells - clue_count) / (total_cells - min_clues) * 100, 0, 100)`
pub(crate) fn clue_count_score(puzzle: &PuzzleDefinition, board: &Board) -> f64 {
    // Count unique cells in the puzzle
    let mut cell_set: std::collections::HashSet<crate::types::Coordinate> =
        std::collections::HashSet::new();
    for g in &puzzle.groups {
        for &c in g {
            cell_set.insert(c);
        }
    }
    let total_cells = cell_set.len();
    let clue_count = board.len();
    let min_clues = theoretical_min_clues(total_cells);

    let denom = total_cells.saturating_sub(min_clues) as f64;
    if denom <= 0.0 {
        return 100.0;
    }

    let ccs = (total_cells as f64 - clue_count as f64) / denom * 100.0;
    ccs.clamp(0.0, 100.0)
}

/// Compute total score from technique score and CCS.
pub(crate) fn total_score(ts: u32, ccs: f64) -> f64 {
    ts as f64 * 0.7 + ccs * 0.3
}

/// Determine difficulty rank from technique score.
pub(crate) fn rank_from_technique_score(ts: u32) -> DifficultyRank {
    match ts {
        0 => DifficultyRank::Beginner,
        25 | 45 => DifficultyRank::Intermediate,
        65 | 80 => DifficultyRank::Advanced,
        _ => DifficultyRank::Expert, // 100 = Beyond
    }
}
