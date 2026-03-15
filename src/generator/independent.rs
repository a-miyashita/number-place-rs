//! Finding maximum independent group sets for seed-based board generation.

use std::collections::HashSet;
use crate::puzzle::PuzzleDefinition;
use crate::types::Coordinate;

/// Find a set of mutually independent groups (no shared cells) for seeding the board generator.
///
/// The returned groups are additionally chosen so that their cells span different
/// rows and columns (no two groups share a row or column value). This ensures that
/// seeding each group with a random permutation of 1..=N produces a locally valid
/// board with no duplicates in any row or column.
pub(super) fn find_max_independent_groups(puzzle: &PuzzleDefinition) -> Vec<usize> {
    let groups = &puzzle.groups;
    let n = groups.len();

    // Compute total unique cells in the puzzle
    let mut all_cells: HashSet<Coordinate> = HashSet::new();
    for g in groups {
        for &c in g {
            all_cells.insert(c);
        }
    }
    let total_cells = all_cells.len();

    // Target: seed at most 1/3 of all cells
    let max_seed_cells = (total_cells / 3).max(puzzle.group_size);

    // Build conflict matrix: conflict[i][j] = true if groups i and j share a cell
    let mut conflict = vec![vec![false; n]; n];
    for i in 0..n {
        for j in i + 1..n {
            let set_i: HashSet<_> = groups[i].iter().copied().collect();
            let shares = groups[j].iter().any(|c| set_i.contains(c));
            if shares {
                conflict[i][j] = true;
                conflict[j][i] = true;
            }
        }
    }

    // Helper: do two groups share any row (same y) or column (same x) value?
    let groups_share_axis = |gi: usize, gj: usize| -> bool {
        let xs_i: HashSet<i32> = groups[gi].iter().map(|&(x, _)| x).collect();
        let ys_i: HashSet<i32> = groups[gi].iter().map(|&(_, y)| y).collect();
        groups[gj].iter().any(|&(x, y)| xs_i.contains(&x) || ys_i.contains(&y))
    };

    let mut best: Vec<usize> = Vec::new();

    for start in 0..n {
        let mut candidate: Vec<usize> = vec![start];
        let mut covered_cells: HashSet<Coordinate> = groups[start].iter().copied().collect();
        let mut covered_xs: HashSet<i32> = groups[start].iter().map(|&(x, _)| x).collect();
        let mut covered_ys: HashSet<i32> = groups[start].iter().map(|&(_, y)| y).collect();

        for g in 0..n {
            if g == start {
                continue;
            }
            // Must not conflict (shared cells)
            if candidate.iter().any(|&c| conflict[g][c]) {
                continue;
            }
            // Must not share any row or column axis with existing candidates
            if candidate.iter().any(|&c| groups_share_axis(g, c)) {
                continue;
            }
            // Cell limit
            let new_cells: usize = groups[g]
                .iter()
                .filter(|c| !covered_cells.contains(c))
                .count();
            if covered_cells.len() + new_cells > max_seed_cells {
                continue;
            }
            // Add group
            for &c in &groups[g] {
                covered_cells.insert(c);
            }
            for &(x, _) in &groups[g] {
                covered_xs.insert(x);
            }
            for &(_, y) in &groups[g] {
                covered_ys.insert(y);
            }
            candidate.push(g);
        }

        if candidate.len() > best.len() {
            best = candidate;
        }
    }

    // Fallback: if we couldn't find even one valid group, use the first group
    if best.is_empty() {
        best.push(0);
    }

    best
}
