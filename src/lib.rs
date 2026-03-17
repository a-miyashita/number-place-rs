//! # number-place
//!
//! A generalised number-place (Sudoku) library providing:
//!
//! - **Solver**: returns the unique solution or an appropriate error.
//! - **Generator**: produces a solvable puzzle from a definition and constraints.
//! - **Difficulty evaluator**: classifies a puzzle by the hardest technique required.
//!
//! ## Quick start
//!
//! ```rust
//! use number_place_rs::puzzle::presets::preset_9x9;
//! use number_place_rs::generator::{GeneratorConstraints, Symmetry};
//! use rand::SeedableRng;
//!
//! let puzzle = preset_9x9();
//! let constraints = GeneratorConstraints::default();
//! let mut rng = rand::rngs::StdRng::seed_from_u64(42);
//! let result = number_place_rs::generate(&puzzle, &constraints, &mut rng).unwrap();
//! assert!(number_place_rs::validate_board(&puzzle, &result.solution));
//! ```

pub mod types;
pub mod puzzle;
pub mod solver;
pub mod generator;
pub mod difficulty;
pub(crate) mod dlx;

pub use types::{Board, Coordinate, GeneratorError, SolverError};
pub use puzzle::{PuzzleDefinition, DrawConfig, BorderSegment};
pub use difficulty::{DifficultyRank, DifficultyResult};
pub use generator::{GeneratedPuzzle, GeneratorConstraints, Symmetry};

/// Solve a puzzle. Returns `Ok(solution)` for a unique solution.
///
/// # Errors
///
/// - [`SolverError::InvalidBoard`] if the board contains duplicates or out-of-range values.
/// - [`SolverError::NoSolution`] if no solution exists.
/// - [`SolverError::MultipleSolutions`] if more than one solution exists.
#[must_use]
pub fn solve(puzzle: &PuzzleDefinition, board: &Board) -> Result<Board, SolverError> {
    solver::solve(puzzle, board)
}

/// Check that all placed values are in range and no group contains duplicates.
///
/// Returns `true` if the board is locally valid (does not check for uniqueness of solution).
#[must_use]
pub fn validate_board(puzzle: &PuzzleDefinition, board: &Board) -> bool {
    solver::validate_board(puzzle, board)
}

/// Generate a solvable puzzle satisfying the given constraints.
///
/// Returns a [`GeneratedPuzzle`] containing both the initial board (clues only)
/// and its unique solution.
///
/// # Errors
///
/// - [`GeneratorError::GenerationFailed`] if constraints cannot be satisfied within the retry limit.
#[must_use]
pub fn generate(
    puzzle: &PuzzleDefinition,
    constraints: &GeneratorConstraints,
    rng: &mut impl rand::Rng,
) -> Result<GeneratedPuzzle, GeneratorError> {
    generator::generate(puzzle, constraints, rng)
}

/// Evaluate the difficulty of an initial board.
#[must_use]
pub fn evaluate_difficulty(puzzle: &PuzzleDefinition, board: &Board) -> DifficultyResult {
    difficulty::evaluate_difficulty(puzzle, board)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use puzzle::presets::{preset_9x9, preset_9x9_diagonal, preset_16x16};
    use rand::SeedableRng;

    // -----------------------------------------------------------------------
    // Helper: build a Board from a row-major array (0 = empty)
    // -----------------------------------------------------------------------

    fn board_from_rows(rows: &[[u8; 9]; 9]) -> Board {
        let mut b = Board::new();
        for (y, row) in rows.iter().enumerate() {
            for (x, &v) in row.iter().enumerate() {
                if v != 0 {
                    b.insert((x as i32, y as i32), v);
                }
            }
        }
        b
    }

    #[allow(dead_code)]
    fn board_from_rows_16(rows: &[[u8; 16]; 16]) -> Board {
        let mut b = Board::new();
        for (y, row) in rows.iter().enumerate() {
            for (x, &v) in row.iter().enumerate() {
                if v != 0 {
                    b.insert((x as i32, y as i32), v);
                }
            }
        }
        b
    }

    // Known 9×9 puzzle and solution
    fn known_puzzle() -> Board {
        board_from_rows(&[
            [5, 3, 0, 0, 7, 0, 0, 0, 0],
            [6, 0, 0, 1, 9, 5, 0, 0, 0],
            [0, 9, 8, 0, 0, 0, 0, 6, 0],
            [8, 0, 0, 0, 6, 0, 0, 0, 3],
            [4, 0, 0, 8, 0, 3, 0, 0, 1],
            [7, 0, 0, 0, 2, 0, 0, 0, 6],
            [0, 6, 0, 0, 0, 0, 2, 8, 0],
            [0, 0, 0, 4, 1, 9, 0, 0, 5],
            [0, 0, 0, 0, 8, 0, 0, 7, 9],
        ])
    }

    fn known_solution() -> Board {
        board_from_rows(&[
            [5, 3, 4, 6, 7, 8, 9, 1, 2],
            [6, 7, 2, 1, 9, 5, 3, 4, 8],
            [1, 9, 8, 3, 4, 2, 5, 6, 7],
            [8, 5, 9, 7, 6, 1, 4, 2, 3],
            [4, 2, 6, 8, 5, 3, 7, 9, 1],
            [7, 1, 3, 9, 2, 4, 8, 5, 6],
            [9, 6, 1, 5, 3, 7, 2, 8, 4],
            [2, 8, 7, 4, 1, 9, 6, 3, 5],
            [3, 4, 5, 2, 8, 6, 1, 7, 9],
        ])
    }

    // -----------------------------------------------------------------------
    // validate_board tests
    // -----------------------------------------------------------------------

    #[test]
    fn validate_empty_board() {
        let puzzle = preset_9x9();
        assert!(validate_board(&puzzle, &Board::new()));
    }

    #[test]
    fn validate_legal_partial() {
        let puzzle = preset_9x9();
        let board = known_puzzle();
        assert!(validate_board(&puzzle, &board));
    }

    #[test]
    fn validate_duplicate_in_group() {
        let puzzle = preset_9x9();
        let mut board = Board::new();
        board.insert((0, 0), 5);
        board.insert((1, 0), 5); // duplicate 5 in row 0
        assert!(!validate_board(&puzzle, &board));
    }

    #[test]
    fn validate_out_of_range_value() {
        let puzzle = preset_9x9();
        let mut board = Board::new();
        board.insert((0, 0), 10); // 10 > 9
        assert!(!validate_board(&puzzle, &board));
    }

    #[test]
    fn validate_zero_value() {
        let puzzle = preset_9x9();
        let mut board = Board::new();
        board.insert((0, 0), 0); // 0 is out of range
        assert!(!validate_board(&puzzle, &board));
    }

    #[test]
    fn validate_complete_board() {
        let puzzle = preset_9x9();
        assert!(validate_board(&puzzle, &known_solution()));
    }

    // -----------------------------------------------------------------------
    // solve tests
    // -----------------------------------------------------------------------

    #[test]
    fn solve_known_puzzle() {
        let puzzle = preset_9x9();
        let result = solve(&puzzle, &known_puzzle());
        assert_eq!(result, Ok(known_solution()));
    }

    #[test]
    fn solve_empty_board_multiple_solutions() {
        let puzzle = preset_9x9();
        assert_eq!(solve(&puzzle, &Board::new()), Err(SolverError::MultipleSolutions));
    }

    #[test]
    fn solve_duplicate_board_invalid() {
        let puzzle = preset_9x9();
        let mut board = Board::new();
        board.insert((0, 0), 5);
        board.insert((1, 0), 5); // duplicate in row 0
        assert_eq!(solve(&puzzle, &board), Err(SolverError::InvalidBoard));
    }

    #[test]
    fn solve_no_solution() {
        // Verify that the solver returns NoSolution for a locally-valid board that has
        // no valid completion.
        //
        // Construction: start from known_puzzle() (24 clues, unique solution) and add
        // one extra clue at (0,2)=2. The unique solution requires (0,2)=1, so no
        // completion exists. The value 2 is locally valid at (0,2):
        //   - row 2 clues: {8, 9} — 2 absent ✓
        //   - col 0 clues: {4, 5, 6, 7, 8} — 2 absent ✓
        //   - box (0,0)-(2,2) clues: {3, 5, 6, 8, 9} — 2 absent ✓
        // The contradiction is global (only detectable by backtracking), so the solver
        // must return NoSolution rather than InvalidBoard.
        let puzzle = preset_9x9();
        let mut board = known_puzzle();
        board.insert((0, 2), 2); // locally valid but contradicts the unique solution
        let result = solve(&puzzle, &board);
        assert_eq!(result, Err(SolverError::NoSolution));
    }

    #[test]
    fn solve_deterministic() {
        let puzzle = preset_9x9();
        let board = known_puzzle();
        let r1 = solve(&puzzle, &board);
        let r2 = solve(&puzzle, &board);
        assert_eq!(r1, r2);
    }

    #[test]
    fn solve_9x9_diagonal() {
        // Verify that the solver correctly handles the diagonal variant by generating
        // a puzzle under diagonal constraints and confirming the solution satisfies them.
        let puzzle = preset_9x9_diagonal();
        let constraints = GeneratorConstraints::default();
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let result = generate(&puzzle, &constraints, &mut rng)
            .expect("diagonal puzzle generation failed");
        assert!(
            validate_board(&puzzle, &result.solution),
            "Solution must satisfy all diagonal constraints"
        );
    }

    // -----------------------------------------------------------------------
    // generate tests
    // -----------------------------------------------------------------------

    #[test]
    fn generate_9x9_default() {
        let puzzle = preset_9x9();
        let constraints = GeneratorConstraints::default();
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let result = generate(&puzzle, &constraints, &mut rng).expect("generation failed");
        assert!(
            validate_board(&puzzle, &result.solution),
            "Solution must be valid"
        );
    }

    #[test]
    fn generate_same_seed_same_result() {
        let puzzle = preset_9x9();
        let constraints = GeneratorConstraints::default();
        let mut rng1 = rand::rngs::StdRng::seed_from_u64(99);
        let mut rng2 = rand::rngs::StdRng::seed_from_u64(99);
        let b1 = generate(&puzzle, &constraints, &mut rng1).expect("gen1 failed");
        let b2 = generate(&puzzle, &constraints, &mut rng2).expect("gen2 failed");
        assert_eq!(b1.board, b2.board, "Same seed must produce same board");
    }

    #[test]
    fn generate_min_clues_respected() {
        let puzzle = preset_9x9();
        let constraints = GeneratorConstraints {
            min_clues: Some(30),
            ..Default::default()
        };
        let mut rng = rand::rngs::StdRng::seed_from_u64(7);
        let result = generate(&puzzle, &constraints, &mut rng).expect("generation failed");
        assert!(result.board.len() >= 30, "Expected >= 30 clues, got {}", result.board.len());
    }

    #[test]
    fn generate_max_clues_respected() {
        let puzzle = preset_9x9();
        let constraints = GeneratorConstraints {
            max_clues: Some(50),
            ..Default::default()
        };
        let mut rng = rand::rngs::StdRng::seed_from_u64(13);
        let result = generate(&puzzle, &constraints, &mut rng).expect("generation failed");
        assert!(result.board.len() <= 50, "Expected <= 50 clues, got {}", result.board.len());
    }

    #[test]
    fn generate_rotation180_symmetry() {
        let puzzle = preset_9x9();
        let constraints = GeneratorConstraints {
            symmetry: Symmetry::Rotation180,
            ..Default::default()
        };
        let mut rng = rand::rngs::StdRng::seed_from_u64(21);
        let result = generate(&puzzle, &constraints, &mut rng).expect("generation failed");

        // Verify 180-degree rotational symmetry: if (x, y) is a hint, (8-x, 8-y) must also be.
        for &(x, y) in result.board.keys() {
            let partner = (8 - x, 8 - y);
            assert!(
                result.board.contains_key(&partner),
                "Missing symmetric partner for ({}, {}): expected ({}, {})",
                x, y, 8 - x, 8 - y
            );
        }
    }

    // -----------------------------------------------------------------------
    // evaluate_difficulty tests
    // -----------------------------------------------------------------------

    #[test]
    fn difficulty_score_in_range() {
        let puzzle = preset_9x9();
        let board = known_puzzle();
        let result = evaluate_difficulty(&puzzle, &board);
        assert!(
            result.total_score >= 0.0 && result.total_score <= 100.0,
            "total_score out of range: {}",
            result.total_score
        );
    }

    #[test]
    fn difficulty_score_consistent() {
        let puzzle = preset_9x9();
        let board = known_puzzle();
        let result = evaluate_difficulty(&puzzle, &board);
        let expected_total = result.technique_score as f64 * 0.7 + result.clue_count_score * 0.3;
        assert!(
            (result.total_score - expected_total).abs() < 1e-9,
            "total_score inconsistency: {} vs {}",
            result.total_score, expected_total
        );
    }

    #[test]
    fn difficulty_beginner_for_nearly_complete_board() {
        // A board with almost all cells filled (only singles needed) should be Beginner.
        let puzzle = preset_9x9();
        let solution = known_solution();

        // Remove just one cell from the complete solution → only one candidate → Naked Single
        let mut easy = solution.clone();
        easy.remove(&(8, 8));

        let result = evaluate_difficulty(&puzzle, &easy);
        assert_eq!(
            result.rank, DifficultyRank::Beginner,
            "Expected Beginner for near-complete board, got {:?}", result.rank
        );
    }

    #[test]
    fn difficulty_ccs_in_range() {
        let puzzle = preset_9x9();
        let board = known_puzzle();
        let result = evaluate_difficulty(&puzzle, &board);
        assert!(
            result.clue_count_score >= 0.0 && result.clue_count_score <= 100.0,
            "CCS out of range: {}",
            result.clue_count_score
        );
    }

    // -----------------------------------------------------------------------
    // Preset tests
    // -----------------------------------------------------------------------

    #[test]
    fn preset_9x9_groups() {
        let puzzle = preset_9x9();
        assert_eq!(puzzle.groups.len(), 27, "9x9 must have 27 groups");
        assert_eq!(puzzle.group_size, 9, "9x9 group_size must be 9");
    }

    #[test]
    fn preset_9x9_all_groups_correct_size() {
        let puzzle = preset_9x9();
        for (i, g) in puzzle.groups.iter().enumerate() {
            assert_eq!(g.len(), 9, "Group {} has wrong size: {}", i, g.len());
        }
    }

    #[test]
    fn preset_9x9_diagonal_groups() {
        let puzzle = preset_9x9_diagonal();
        assert_eq!(puzzle.groups.len(), 29, "9x9 diagonal must have 29 groups");
        assert_eq!(puzzle.group_size, 9);
    }

    #[test]
    fn preset_9x9_diagonal_all_groups_correct_size() {
        let puzzle = preset_9x9_diagonal();
        for (i, g) in puzzle.groups.iter().enumerate() {
            assert_eq!(g.len(), 9, "Group {} has wrong size: {}", i, g.len());
        }
    }

    #[test]
    fn preset_16x16_groups() {
        let puzzle = preset_16x16();
        assert_eq!(puzzle.groups.len(), 48, "16x16 must have 48 groups");
        assert_eq!(puzzle.group_size, 16);
    }

    #[test]
    fn preset_16x16_all_groups_correct_size() {
        let puzzle = preset_16x16();
        for (i, g) in puzzle.groups.iter().enumerate() {
            assert_eq!(g.len(), 16, "Group {} has wrong size: {}", i, g.len());
        }
    }

    #[test]
    fn preset_groups_no_duplicates_within_group() {
        for puzzle in [preset_9x9(), preset_9x9_diagonal(), preset_16x16()] {
            for (gi, group) in puzzle.groups.iter().enumerate() {
                let mut seen = std::collections::HashSet::new();
                for &cell in group {
                    assert!(seen.insert(cell), "Duplicate cell in group {}: {:?}", gi, cell);
                }
            }
        }
    }
}

