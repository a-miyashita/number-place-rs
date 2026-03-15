//! Core type definitions for the number-place library.

/// A cell coordinate `(x, y)`. Negative values are allowed for irregular puzzle shapes.
pub type Coordinate = (i32, i32);

/// A board mapping filled cell coordinates to their values (1..=N).
/// Empty cells have no entry in the map.
pub type Board = std::collections::HashMap<Coordinate, u8>;

/// Errors returned by the solver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolverError {
    /// No solution exists.
    NoSolution,
    /// More than one solution exists.
    MultipleSolutions,
    /// The board already contains a rule violation: an out-of-range value or
    /// a duplicate within the same group.
    InvalidBoard,
}

/// Errors returned by the generator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeneratorError {
    /// Could not generate a puzzle satisfying the given constraints.
    GenerationFailed,
}
