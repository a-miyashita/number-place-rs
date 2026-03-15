# number-place-rs

A generalised number-place (Sudoku) library written in Rust.

Provides three core capabilities for **any** puzzle variant that can be expressed as a list of groups:

| Component | Description |
|-----------|-------------|
| **Solver** | Returns the unique solution or an appropriate error |
| **Generator** | Produces a solvable puzzle from a definition and generation constraints |
| **Difficulty evaluator** | Classifies a puzzle by the hardest human solving technique required |

## Supported puzzle types (built-in presets)

| Preset | Groups | Notes |
|--------|--------|-------|
| `preset_9x9()` | 27 (9 rows + 9 cols + 9 boxes) | Standard 9×9 Sudoku |
| `preset_9x9_diagonal()` | 29 (+ main diagonal + anti-diagonal) | Diagonal Sudoku |
| `preset_16x16()` | 48 (16 rows + 16 cols + 16 boxes) | 16×16 Sudoku |

Custom puzzle types (Samurai, irregular boxes, etc.) can be defined as a list of `Group` values — no code changes to the core algorithms are needed.

## Getting started

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
number-place = { path = "..." }
rand = "0.9"
```

### Generate and solve a puzzle

```rust
use number_place::puzzle::presets::preset_9x9;
use number_place::generator::{GeneratorConstraints, Symmetry};
use rand::SeedableRng;

let puzzle = preset_9x9();
let constraints = GeneratorConstraints::default();
let mut rng = rand::rngs::StdRng::seed_from_u64(42);

// Generate a solvable puzzle
let board = number_place::generate(&puzzle, &constraints, &mut rng)?;

// Solve it
let solution = number_place::solve(&puzzle, &board)?;

// Validate the solution
assert!(number_place::validate_board(&puzzle, &solution));
```

### Evaluate difficulty

```rust
let result = number_place::evaluate_difficulty(&puzzle, &board);
println!("Rank: {:?}", result.rank);           // Beginner / Intermediate / Advanced / Expert
println!("Score: {:.1}", result.total_score);  // 0.0 – 100.0
```

### Custom puzzle definition

Any variant that can be expressed as a list of groups works out of the box:

```rust
use number_place::puzzle::{PuzzleDefinition, DrawConfig};

// Minimal 4×4 puzzle: rows, columns, and 2×2 boxes
let groups: Vec<Vec<(i32, i32)>> = vec![
    // rows
    vec![(0,0),(1,0),(2,0),(3,0)],
    vec![(0,1),(1,1),(2,1),(3,1)],
    vec![(0,2),(1,2),(2,2),(3,2)],
    vec![(0,3),(1,3),(2,3),(3,3)],
    // columns
    vec![(0,0),(0,1),(0,2),(0,3)],
    vec![(1,0),(1,1),(1,2),(1,3)],
    vec![(2,0),(2,1),(2,2),(2,3)],
    vec![(3,0),(3,1),(3,2),(3,3)],
    // 2×2 boxes
    vec![(0,0),(1,0),(0,1),(1,1)],
    vec![(2,0),(3,0),(2,1),(3,1)],
    vec![(0,2),(1,2),(0,3),(1,3)],
    vec![(2,2),(3,2),(2,3),(3,3)],
];

let definition = PuzzleDefinition {
    groups,
    group_size: 4,
    draw_config: DrawConfig::default(),
};
```

## API reference

### `solve`

```rust
pub fn solve(puzzle: &PuzzleDefinition, board: &Board) -> Result<Board, SolverError>
```

Returns `Ok(solution)` when there is exactly one solution.

| Error | Meaning |
|-------|---------|
| `SolverError::InvalidBoard` | Current placements violate a group constraint or are out of range |
| `SolverError::NoSolution` | No completion exists (illegal board) |
| `SolverError::MultipleSolutions` | More than one completion exists (legal but not solvable) |

### `validate_board`

```rust
pub fn validate_board(puzzle: &PuzzleDefinition, board: &Board) -> bool
```

Returns `true` when all placed values are in `1..=group_size` and no group contains a duplicate. Does **not** check for uniqueness of solution.

### `generate`

```rust
pub fn generate(
    puzzle: &PuzzleDefinition,
    constraints: &GeneratorConstraints,
    rng: &mut impl rand::Rng,
) -> Result<Board, GeneratorError>
```

Generates a solvable puzzle. The `rng` parameter is taken explicitly, so results are reproducible given the same seed.

`GeneratorConstraints` fields:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `symmetry` | `Symmetry` | `None` | Clue pattern symmetry |
| `min_clues` | `Option<usize>` | `None` | Minimum number of hints |
| `max_clues` | `Option<usize>` | `None` | Maximum number of hints |
| `target_difficulty` | `Option<DifficultyRank>` | `None` | Target difficulty rank |

`Symmetry` variants: `None`, `Rotation180`, `HorizontalMirror`, `VerticalMirror`.

### `evaluate_difficulty`

```rust
pub fn evaluate_difficulty(puzzle: &PuzzleDefinition, board: &Board) -> DifficultyResult
```

Simulates a human solver and classifies the puzzle by the hardest technique required.

`DifficultyResult` fields:

| Field | Description |
|-------|-------------|
| `rank` | `Beginner` / `Intermediate` / `Advanced` / `Expert` |
| `technique_score` | Score of the hardest required technique (0 / 25 / 45 / 65 / 80 / 100) |
| `clue_count_score` | Difficulty from clue count (0.0–100.0, higher = fewer clues) |
| `total_score` | `technique_score × 0.7 + clue_count_score × 0.3` |

Difficulty ranks correspond to the following solving techniques:

| Rank | Required technique |
|------|--------------------|
| Beginner | Basic Singles (Naked + Hidden Single) |
| Intermediate | Locked Candidates or Naked/Hidden Pair |
| Advanced | Naked/Hidden Triple, X-Wing equivalent, Wing patterns |
| Expert | Not solvable by the above techniques (trial-and-error required) |

## Drawing information

Each `PuzzleDefinition` carries a `DrawConfig` that a game application can use to render the board:

- **`border_segments`** — thick-line edges expressed as pairs of grid points `(from, to)`. Default cell borders are thin; only the exceptions need to be listed.
- **`shade_regions`** — groups of cells with a background colour (RGBA), used for diagonal highlights, Samurai overlaps, etc.

The board width and height are not stored explicitly; the application derives them from the coordinate ranges present in the group list.

## Architecture

```
src/
  lib.rs                      Public API (solve, validate_board, generate, evaluate_difficulty)
  types.rs                    Coordinate, Board, SolverError, GeneratorError
  puzzle/
    mod.rs                    PuzzleDefinition, DrawConfig, BorderSegment, ShadeRegion
    presets.rs                Built-in presets (9×9, 9×9 diagonal, 16×16)
  solver/
    mod.rs                    Public solve() and validate_board()
    candidates.rs             PuzzleContext + SolverState (bitmask candidate tracking)
    propagate.rs              Naked/Hidden Single constraint propagation
    backtrack.rs              MRV + forward-checking backtracking
  dlx/
    mod.rs                    Algorithm X + Dancing Links (uniqueness check)
    builder.rs                PuzzleDefinition → DLX exact-cover matrix
  generator/
    mod.rs                    Public generate()
    independent.rs            Maximum independent group set (greedy)
    full_board.rs             Complete board generation (seed + randomised backtracking)
    removal.rs                Cell removal with uniqueness check
  difficulty/
    mod.rs                    Public evaluate_difficulty()
    human_solver.rs           Human-solver simulation
    scorer.rs                 CCS / TS scoring, DifficultyRank mapping
    techniques/               T0 Basic Singles … T4 Advanced (Quad / Fish / Wings)
```

### Key design decisions

- **Group abstraction**: all puzzle variants are represented as a flat list of groups. The solver, generator, and difficulty evaluator contain no variant-specific logic.
- **Bitmask candidates**: each cell's candidate set is stored as a `u32` bitmask (bit *k* = value *k* is a candidate), making clone and `popcount` cheap during backtracking.
- **DLX for uniqueness**: the cell-removal step uses Algorithm X (Dancing Links) to count solutions up to 2, stopping as soon as a second solution is found.
- **Independent group seeding**: before backtracking, a maximum set of mutually independent groups is filled with random permutations, reducing the search space and producing diverse complete boards.
- **RNG injection**: `generate` accepts `impl rand::Rng` explicitly, enabling reproducible tests with a seeded RNG.

## License

MIT
