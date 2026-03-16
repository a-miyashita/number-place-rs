//! Built-in preset puzzle definitions: 9×9 standard, 9×9 diagonal, and 16×16.

use super::{BorderSegment, DrawConfig, Group, PuzzleDefinition};

/// Returns the standard 9×9 Sudoku puzzle definition.
///
/// Contains 27 groups (9 rows + 9 columns + 9 3×3 boxes) with `group_size = 9`.
pub fn preset_9x9() -> PuzzleDefinition {
    let mut groups: Vec<Group> = Vec::new();

    // Row groups
    for y in 0..9i32 {
        let row: Group = (0..9i32).map(|x| (x, y)).collect();
        groups.push(row);
    }

    // Column groups
    for x in 0..9i32 {
        let col: Group = (0..9i32).map(|y| (x, y)).collect();
        groups.push(col);
    }

    // 3×3 box groups
    for by in 0..3i32 {
        for bx in 0..3i32 {
            let mut box_group: Group = Vec::new();
            for dy in 0..3i32 {
                for dx in 0..3i32 {
                    box_group.push((bx * 3 + dx, by * 3 + dy));
                }
            }
            groups.push(box_group);
        }
    }

    let draw_config = draw_config_9x9();

    PuzzleDefinition {
        groups,
        group_size: 9,
        draw_config,
    }
}

/// Returns a 9×9 Sudoku puzzle definition with two extra diagonal groups.
///
/// Contains 29 groups (9 rows + 9 columns + 9 3×3 boxes + main diagonal + anti-diagonal)
/// with `group_size = 9`.
pub fn preset_9x9_diagonal() -> PuzzleDefinition {
    let mut base = preset_9x9();

    // Main diagonal: (0,0), (1,1), ..., (8,8)
    let main_diag: Group = (0..9i32).map(|i| (i, i)).collect();
    // Anti-diagonal: (8,0), (7,1), ..., (0,8)
    let anti_diag: Group = (0..9i32).map(|i| (8 - i, i)).collect();

    base.groups.push(main_diag.clone());
    base.groups.push(anti_diag.clone());

    // Set per-cell style bits for the diagonals.
    // Bit 0 (0x01): main diagonal, Bit 1 (0x02): anti-diagonal.
    for cell in &main_diag {
        *base.draw_config.cell_styles.entry(*cell).or_insert(0) |= 0x01;
    }
    for cell in &anti_diag {
        *base.draw_config.cell_styles.entry(*cell).or_insert(0) |= 0x02;
    }

    base
}

/// Returns the 16×16 Sudoku puzzle definition.
///
/// Contains 48 groups (16 rows + 16 columns + 16 4×4 boxes) with `group_size = 16`.
pub fn preset_16x16() -> PuzzleDefinition {
    let mut groups: Vec<Group> = Vec::new();

    // Row groups
    for y in 0..16i32 {
        let row: Group = (0..16i32).map(|x| (x, y)).collect();
        groups.push(row);
    }

    // Column groups
    for x in 0..16i32 {
        let col: Group = (0..16i32).map(|y| (x, y)).collect();
        groups.push(col);
    }

    // 4×4 box groups
    for by in 0..4i32 {
        for bx in 0..4i32 {
            let mut box_group: Group = Vec::new();
            for dy in 0..4i32 {
                for dx in 0..4i32 {
                    box_group.push((bx * 4 + dx, by * 4 + dy));
                }
            }
            groups.push(box_group);
        }
    }

    let draw_config = draw_config_16x16();

    PuzzleDefinition {
        groups,
        group_size: 16,
        draw_config,
    }
}

// --- Internal helpers ---

fn draw_config_9x9() -> DrawConfig {
    let mut segs: Vec<BorderSegment> = Vec::new();

    // Vertical thick lines at x = 0, 3, 6, 9
    for gx in [0u32, 3, 6, 9] {
        segs.push(BorderSegment {
            from: (gx, 0),
            to: (gx, 9),
        });
    }

    // Horizontal thick lines at y = 0, 3, 6, 9
    for gy in [0u32, 3, 6, 9] {
        segs.push(BorderSegment {
            from: (0, gy),
            to: (9, gy),
        });
    }

    DrawConfig {
        border_segments: segs,
        cell_styles: std::collections::HashMap::new(),
    }
}

fn draw_config_16x16() -> DrawConfig {
    let mut segs: Vec<BorderSegment> = Vec::new();

    // Vertical thick lines at x = 0, 4, 8, 12, 16
    for gx in [0u32, 4, 8, 12, 16] {
        segs.push(BorderSegment {
            from: (gx, 0),
            to: (gx, 16),
        });
    }

    // Horizontal thick lines at y = 0, 4, 8, 12, 16
    for gy in [0u32, 4, 8, 12, 16] {
        segs.push(BorderSegment {
            from: (0, gy),
            to: (16, gy),
        });
    }

    DrawConfig {
        border_segments: segs,
        cell_styles: std::collections::HashMap::new(),
    }
}
