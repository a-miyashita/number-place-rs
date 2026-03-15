//! Human solver simulation for difficulty evaluation.

use crate::puzzle::PuzzleDefinition;
use crate::solver::{PuzzleContext, SolverState};
use crate::types::Board;
use super::techniques::{
    Technique,
    singles::BasicSingles,
    locked::LockedCandidates,
    pairs::Pairs,
    triples::Triples,
    advanced::Advanced,
};

/// Simulate human solving, returning `(solved, max_technique_score)`.
///
/// The `max_technique_score` corresponds to the hardest technique level needed:
/// - 0  = T0 Basic Singles only
/// - 25 = T1 Locked Candidates
/// - 45 = T2 Pairs
/// - 65 = T3 Triples / Fish(2)
/// - 80 = T4 Quad / Fish(3) / Wings
/// - 100 = T5 Beyond (couldn't solve with techniques T0-T4)
pub(crate) fn human_solve(puzzle: &PuzzleDefinition, board: &Board) -> (bool, u32) {
    let ctx = PuzzleContext::new(&puzzle.groups, puzzle.group_size);
    let Some(mut state) = SolverState::new(&ctx, board) else {
        return (false, 100);
    };

    let techniques: &[(u32, &dyn Technique)] = &[
        (0, &BasicSingles),
        (25, &LockedCandidates),
        (45, &Pairs),
        (65, &Triples),
        (80, &Advanced),
    ];

    let mut max_level = 0u32;

    loop {
        let mut progress = false;

        for &(score, technique) in techniques {
            if technique.apply(&ctx, &mut state) {
                if score > max_level {
                    max_level = score;
                }
                progress = true;
                break; // restart with lowest-level technique
            }
        }

        if !progress {
            break;
        }
    }

    let solved = state.n_filled == ctx.n_cells;
    let technique_score = if solved { max_level } else { 100 };

    (solved, technique_score)
}
