//! Technique trait and shared types for human-solver techniques.

pub mod singles;
pub mod locked;
pub mod pairs;
pub mod triples;
pub mod advanced;

use crate::solver::{PuzzleContext, SolverState};

/// A human solving technique that can be applied to the current state.
pub(crate) trait Technique {
    /// Apply the technique once. Returns `true` if any progress was made.
    fn apply(&self, ctx: &PuzzleContext, state: &mut SolverState) -> bool;
}
