//! Candidate bitmask management and the solver's internal context/state types.

use std::collections::HashMap;
use crate::types::Coordinate;

/// Immutable puzzle context built once from a `PuzzleDefinition`.
pub(crate) struct PuzzleContext {
    /// All cells in the puzzle, sorted.
    pub cells: Vec<Coordinate>,
    /// Maps cell coordinate to its index in `cells`.
    pub cell_index: HashMap<Coordinate, usize>,
    /// For each cell index: the indices of its peer cells (all cells sharing a group).
    pub peers: Vec<Vec<usize>>,
    /// For each group index: the cell indices in that group.
    pub groups: Vec<Vec<usize>>,
    /// For each cell index: the group indices it belongs to.
    pub cell_groups: Vec<Vec<usize>>,
    /// The number of symbols (group size N).
    pub group_size: usize,
    /// Total number of cells.
    pub n_cells: usize,
}

/// Mutable solver state, cloned for backtracking.
#[derive(Clone)]
pub(crate) struct SolverState {
    /// Value at each cell (0 = empty, 1..=N = placed).
    pub values: Vec<u8>,
    /// Candidate bitmask for each cell.
    /// Bit k set (k >= 1) means value k is a candidate.
    /// When a cell is filled, its mask is 0 (irrelevant).
    pub masks: Vec<u32>,
    /// Number of filled cells.
    pub n_filled: usize,
}

impl PuzzleContext {
    /// Build a `PuzzleContext` from a list of groups and the group size.
    pub(crate) fn new(groups: &[Vec<Coordinate>], group_size: usize) -> Self {
        // Collect all cells
        let mut cell_set: std::collections::BTreeSet<Coordinate> = std::collections::BTreeSet::new();
        for g in groups {
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

        // Convert groups to index-based
        let groups_idx: Vec<Vec<usize>> = groups
            .iter()
            .map(|g| g.iter().map(|c| cell_index[c]).collect())
            .collect();

        // cell_groups[i] = list of group indices containing cell i
        let mut cell_groups: Vec<Vec<usize>> = vec![Vec::new(); n_cells];
        for (gi, g) in groups_idx.iter().enumerate() {
            for &ci in g {
                cell_groups[ci].push(gi);
            }
        }

        // peers[i] = set of cell indices that share at least one group with cell i
        let mut peers: Vec<Vec<usize>> = vec![Vec::new(); n_cells];
        for (ci, cgs) in cell_groups.iter().enumerate() {
            let mut peer_set: std::collections::HashSet<usize> = std::collections::HashSet::new();
            for &gi in cgs {
                for &peer in &groups_idx[gi] {
                    if peer != ci {
                        peer_set.insert(peer);
                    }
                }
            }
            peers[ci] = peer_set.into_iter().collect();
            peers[ci].sort_unstable();
        }

        PuzzleContext {
            cells,
            cell_index,
            peers,
            groups: groups_idx,
            cell_groups,
            group_size,
            n_cells,
        }
    }
}

/// Compute the initial candidate mask for N symbols: bits 1..=N are set.
///
/// # Panics (debug) / wraps (release)
///
/// `n` must satisfy `n <= 30`. The shift `1u32 << (n + 1)` overflows for `n >= 31`.
/// In practice the library supports up to N = 25 (a practical upper bound given
/// the u8 value type and the complexity of larger puzzles). The bitmask design
/// requires `N < 32`; this is documented on [`PuzzleDefinition::group_size`].
pub(crate) fn initial_mask(n: usize) -> u32 {
    // bits 1..=n
    (1u32 << (n + 1)) - 2
}

impl SolverState {
    /// Create an initial state from the given board (partial placement).
    /// Returns `None` if the board contains an immediate conflict.
    pub(crate) fn new(ctx: &PuzzleContext, board: &crate::types::Board) -> Option<Self> {
        let full_mask = initial_mask(ctx.group_size);
        let mut values = vec![0u8; ctx.n_cells];
        let mut masks = vec![full_mask; ctx.n_cells];
        let mut n_filled = 0;

        // Place pre-filled cells
        for (&coord, &val) in board {
            let ci = *ctx.cell_index.get(&coord)?;
            if val < 1 || val as usize > ctx.group_size {
                return None; // out-of-range value
            }
            if values[ci] != 0 {
                return None; // duplicate
            }
            values[ci] = val;
            masks[ci] = 0;
            n_filled += 1;
        }

        let mut state = SolverState { values, masks, n_filled };

        // Remove placed values from peers
        let placed: Vec<(usize, u8)> = board
            .iter()
            .filter_map(|(&coord, &val)| ctx.cell_index.get(&coord).map(|&ci| (ci, val)))
            .collect();

        for (ci, val) in placed {
            let bit = 1u32 << val;
            for &peer in &ctx.peers[ci] {
                if state.values[peer] == 0 {
                    state.masks[peer] &= !bit;
                    if state.masks[peer] == 0 {
                        return None; // conflict
                    }
                }
            }
        }

        Some(state)
    }
}
