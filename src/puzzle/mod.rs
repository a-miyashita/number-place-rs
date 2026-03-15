//! Puzzle definition types and the [`PuzzleDefinition`] struct.

pub mod presets;

use crate::types::Coordinate;

/// A group is a set of cell coordinates where each symbol must appear exactly once.
pub type Group = Vec<Coordinate>;

/// The complete definition of a number-place puzzle variant.
#[derive(Debug, Clone)]
pub struct PuzzleDefinition {
    /// The list of groups. Each group's cells must contain each symbol 1..=N exactly once.
    pub groups: Vec<Group>,
    /// The common size N of all groups (also the maximum symbol value).
    ///
    /// # Constraint
    ///
    /// Must satisfy `group_size < 32`. The solver represents candidate sets as `u32` bitmasks
    /// where bit k indicates that value k is a candidate; values 1..=N occupy bits 1..=N.
    /// In practice the library is designed for N ≤ 25 (e.g. 9×9 or 16×16 puzzles).
    pub group_size: usize,
    /// Visual rendering information for the puzzle.
    pub draw_config: DrawConfig,
}

/// Visual rendering configuration attached to a [`PuzzleDefinition`].
#[derive(Debug, Clone)]
pub struct DrawConfig {
    /// Border segments that should be drawn with a thick line.
    pub border_segments: Vec<BorderSegment>,
    /// Regions of cells that should receive a background colour.
    pub shade_regions: Vec<ShadeRegion>,
}

/// A segment of the grid border described in grid-point coordinates.
///
/// Grid point `(gx, gy)` is the top-left corner of cell `(gx, gy)`.
/// A horizontal segment from `(0, 3)` to `(9, 3)` describes the thick line
/// separating the first three rows from the next three.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BorderSegment {
    /// Start grid point (inclusive).
    pub from: (u32, u32),
    /// End grid point (inclusive).
    pub to: (u32, u32),
}

/// A set of cells that should be drawn with a coloured background.
#[derive(Debug, Clone)]
pub struct ShadeRegion {
    /// Cells belonging to this region.
    pub cells: Vec<Coordinate>,
    /// Background colour as `[R, G, B, A]` with each component in `0..=255`.
    pub color: [u8; 4],
}
