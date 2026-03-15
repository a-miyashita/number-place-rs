//! Builds a DLX matrix from a `PuzzleDefinition` and a set of fixed (hint) cells.

use std::collections::HashMap;
use crate::puzzle::PuzzleDefinition;
use crate::types::{Board, Coordinate};
use super::{Dlx, DlxNode, ROOT};

/// Build a [`Dlx`] instance ready to count solutions for the given puzzle and fixed cells.
pub(crate) fn build_dlx(puzzle: &PuzzleDefinition, fixed: &Board) -> Dlx {
    let n = puzzle.group_size;

    // Collect and sort all cells
    let mut cell_set: std::collections::BTreeSet<Coordinate> = std::collections::BTreeSet::new();
    for g in &puzzle.groups {
        for &c in g {
            cell_set.insert(c);
        }
    }
    let cells: Vec<Coordinate> = cell_set.into_iter().collect();
    let n_cells = cells.len();
    let cell_index: HashMap<Coordinate, usize> = cells
        .iter()
        .enumerate()
        .map(|(i, &c)| (c, i))
        .collect();

    let n_groups = puzzle.groups.len();

    // Column layout:
    //   0         : root (special)
    //   1..=n_cells           : cell constraints
    //   n_cells+1 .. n_cells + n_groups*n + 1 : group-value constraints
    let n_cols = n_cells + n_groups * n;

    // Allocate nodes: root + column headers + data nodes
    // Upper bound: n_cells * n rows, each row covers at most (1 + n_groups_per_cell) cols
    // We'll push nodes dynamically.
    let mut nodes: Vec<DlxNode> = Vec::with_capacity(1 + n_cols + n_cells * n * 4);
    let mut sizes: Vec<usize> = vec![0; n_cols + 1]; // indexed by col header index

    // Node 0: root
    nodes.push(DlxNode {
        left: n_cols,   // root.left = last col header
        right: 1,       // root.right = first col header
        up: ROOT,
        down: ROOT,
        col: ROOT,
        row_id: 0,
    });

    // Column header nodes 1..=n_cols
    for i in 1..=n_cols {
        nodes.push(DlxNode {
            left: i - 1,
            right: if i == n_cols { ROOT } else { i + 1 },
            up: i,   // circular: up/down point to self initially
            down: i,
            col: i,
            row_id: 0,
        });
    }

    // Determine which columns to pre-cover for fixed cells
    // For a fixed cell (ci, val): cover cell-col ci+1, and for each group g containing ci,
    // cover group-value col n_cells + g*n + (val-1) + 1.

    // For each cell group membership
    let mut cell_groups: Vec<Vec<usize>> = vec![Vec::new(); n_cells];
    for (gi, g) in puzzle.groups.iter().enumerate() {
        for &coord in g {
            let ci = cell_index[&coord];
            cell_groups[ci].push(gi);
        }
    }

    // Track which cells are fixed
    let mut fixed_ci_val: Vec<(usize, u8)> = Vec::new();
    for (&coord, &val) in fixed {
        if let Some(&ci) = cell_index.get(&coord) {
            if val >= 1 && val as usize <= n {
                fixed_ci_val.push((ci, val));
            }
        }
    }

    // Determine which cols are pre-covered
    let mut covered_cols: std::collections::HashSet<usize> = std::collections::HashSet::new();

    // We need to: for each fixed (ci, val), select the row (ci, val):
    //   cover cell col (ci+1)
    //   for each group g of ci: cover group-val col n_cells + g*n + (val-1) + 1
    // Then also eliminate all other rows that conflict.
    // The easiest correct approach: build the full matrix, then pre-select fixed rows.

    // Build all rows for unfixed cells and the selected row for fixed cells.
    // Strategy:
    //   1. Add all rows (ci, v) for unfixed ci.
    //   2. For each fixed (ci, val), add row (ci, val) and mark it as selected.
    //   3. After building, run "cover" for each selected row's columns to remove conflicts.

    // For simplicity: build all rows, then pre-cover fixed cells.
    // We'll track which rows belong to fixed cells as "selected".

    let mut selected_rows: Vec<Vec<usize>> = Vec::new(); // node indices per selected row

    for ci in 0..n_cells {
        let coord = cells[ci];
        let fixed_val = fixed.get(&coord).copied();

        for v in 1..=n as u8 {
            if let Some(fv) = fixed_val {
                if fv != v {
                    continue; // skip non-matching rows for fixed cells
                }
            }

            // Build the row for (ci, v)
            // Columns covered: cell col (ci+1), and group-val cols
            let mut row_cols: Vec<usize> = Vec::new();

            // Cell constraint col
            let cell_col = ci + 1;
            row_cols.push(cell_col);

            // Group-value constraint cols
            for &gi in &cell_groups[ci] {
                let gv_col = n_cells + gi * n + (v as usize - 1) + 1;
                row_cols.push(gv_col);
            }

            // Determine row_id
            let row_id = ci * n + (v as usize - 1);

            // Insert the nodes for this row, linking them horizontally
            let row_node_start = nodes.len();
            let row_len = row_cols.len();

            for (k, &col) in row_cols.iter().enumerate() {
                let node_idx = nodes.len();
                // Vertical link: insert above the column header
                let col_header = col;
                let up = nodes[col_header].up;
                let down = col_header;

                let left = if k == 0 { row_node_start + row_len - 1 } else { node_idx - 1 };
                let right = if k == row_len - 1 { row_node_start } else { node_idx + 1 };

                nodes.push(DlxNode {
                    left,
                    right,
                    up,
                    down,
                    col: col_header,
                    row_id,
                });

                // Fix vertical links
                nodes[up].down = node_idx;
                nodes[down].up = node_idx;
                sizes[col_header] += 1;
            }

            if fixed_val.is_some() {
                // This is a selected row
                let row_nodes: Vec<usize> = (row_node_start..nodes.len()).collect();
                selected_rows.push(row_nodes);
            }
        }
    }

    // Pre-cover selected rows (applying fixed hints)
    // For each selected row, we cover the column of every node in that row.
    // We need to do this in the DLX structure.
    // The standard approach: for each selected row, cover each column in the row.

    let mut dlx = Dlx { nodes, sizes, n_cols };

    for row_nodes in selected_rows {
        // Cover each column in this row
        let cols: Vec<usize> = row_nodes.iter().map(|&ni| dlx.nodes[ni].col).collect();
        for col in cols {
            if !covered_cols.contains(&col) {
                covered_cols.insert(col);
                dlx.cover(col);
            }
        }
    }

    dlx
}
