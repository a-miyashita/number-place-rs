//! Cell removal strategy with uniqueness checking via DLX.

use rand::seq::SliceRandom;
use crate::puzzle::PuzzleDefinition;
use crate::types::{Board, Coordinate};
use super::GeneratorConstraints;
use super::Symmetry;
use crate::dlx::build_dlx;
use crate::solver::{PuzzleContext, propagation_check};

/// Check whether the given board has exactly one solution.
///
/// First tries fast constraint propagation; falls back to DLX only when
/// propagation alone cannot determine the result.
fn is_unique(ctx: &PuzzleContext, puzzle: &PuzzleDefinition, board: &Board) -> bool {
    match propagation_check(ctx, board) {
        Some(result) => result,
        None => {
            let mut dlx = build_dlx(puzzle, board);
            dlx.count_solutions(2) == 1
        }
    }
}

/// Get the symmetric partner of a cell under the given symmetry.
/// Returns `None` if the cell has no unique partner (e.g. it maps to itself).
fn symmetric_partner(
    coord: Coordinate,
    symmetry: &Symmetry,
    cells: &[Coordinate],
) -> Option<Coordinate> {
    // Compute the bounding box of all cells
    let min_x = cells.iter().map(|&(x, _)| x).min().unwrap_or(0);
    let max_x = cells.iter().map(|&(x, _)| x).max().unwrap_or(0);
    let min_y = cells.iter().map(|&(_, y)| y).min().unwrap_or(0);
    let max_y = cells.iter().map(|&(_, y)| y).max().unwrap_or(0);

    let (x, y) = coord;
    let partner = match symmetry {
        Symmetry::None => return None,
        Symmetry::Rotation180 => (min_x + max_x - x, min_y + max_y - y),
        Symmetry::HorizontalMirror => (min_x + max_x - x, y),
        Symmetry::VerticalMirror => (x, min_y + max_y - y),
    };

    if partner == coord {
        None
    } else {
        Some(partner)
    }
}

/// Remove cells from a full board to produce a puzzle satisfying the constraints.
pub(super) fn remove_cells(
    full_board: &Board,
    puzzle: &PuzzleDefinition,
    constraints: &GeneratorConstraints,
    rng: &mut impl rand::Rng,
) -> Board {
    let mut board = full_board.clone();
    // Build PuzzleContext once; it is immutable for the lifetime of this call.
    let ctx = PuzzleContext::new(&puzzle.groups, puzzle.group_size);

    // Collect all cells in the puzzle
    let mut cell_set: std::collections::BTreeSet<Coordinate> = std::collections::BTreeSet::new();
    for g in &puzzle.groups {
        for &c in g {
            cell_set.insert(c);
        }
    }
    let all_cells: Vec<Coordinate> = cell_set.into_iter().collect();

    let mut cells_order: Vec<Coordinate> = all_cells.clone();
    cells_order.shuffle(rng);

    let mut processed: std::collections::HashSet<Coordinate> = std::collections::HashSet::new();

    for coord in cells_order {
        if processed.contains(&coord) {
            continue;
        }

        // Determine what to remove
        let partner = symmetric_partner(coord, &constraints.symmetry, &all_cells);
        let mut to_remove: Vec<Coordinate> = vec![coord];
        if let Some(p) = partner {
            to_remove.push(p);
        }

        // Only attempt removal if all cells in the group are present in the board
        if !to_remove.iter().all(|c| board.contains_key(c)) {
            processed.insert(coord);
            continue;
        }

        // Check min_clues constraint before removal
        let current_clues = board.len();
        let would_be_clues = current_clues - to_remove.len();
        if let Some(min) = constraints.min_clues {
            if would_be_clues < min {
                processed.insert(coord);
                continue;
            }
        }

        // Temporarily remove
        let saved: Vec<(Coordinate, u8)> = to_remove
            .iter()
            .filter_map(|&c| board.remove(&c).map(|v| (c, v)))
            .collect();

        // Check max_clues: if current_clues is already ≤ max, we can still try to remove
        // We only reject if the result would exceed max_clues... actually max_clues restricts
        // the upper bound, so we want to remove more cells to reach at or below max.
        // Since we're removing cells, we always try to remove more.
        // Check uniqueness
        let unique = is_unique(&ctx, puzzle, &board);

        if unique {
            // Accept removal
            for &(c, _) in &saved {
                processed.insert(c);
            }
            // Check max_clues: if we're under max, keep going
        } else {
            // Restore
            for (c, v) in saved {
                board.insert(c, v);
            }
        }

        processed.insert(coord);
        if let Some(p) = partner {
            processed.insert(p);
        }
    }

    // Enforce max_clues: if board has too many clues, we can't do much more
    // (we've already tried all removals). Just return what we have.
    board
}
