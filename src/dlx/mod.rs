//! Dancing Links (DLX) implementation of Algorithm X for exact-cover problems.
//! Used internally to check uniqueness of solutions during puzzle generation.

mod builder;

pub(crate) use builder::build_dlx;

/// A single node in the Dancing Links structure.
struct DlxNode {
    left: usize,
    right: usize,
    up: usize,
    down: usize,
    /// Index of the column header for data nodes; equals `self` index for headers.
    col: usize,
    /// Row identifier (index into the (cell, value) row list).
    #[allow(dead_code)]
    row_id: usize,
}

/// A Dancing Links exact-cover solver.
pub(crate) struct Dlx {
    nodes: Vec<DlxNode>,
    /// Size (number of uncovered rows) for each column header, indexed 0..=n_cols.
    sizes: Vec<usize>,
    /// Number of columns (index 0 = root, 1..=n_cols = column headers).
    #[allow(dead_code)]
    n_cols: usize,
}

const ROOT: usize = 0;

impl Dlx {
    /// Count the number of exact-cover solutions, stopping at `limit`.
    pub(crate) fn count_solutions(&mut self, limit: usize) -> usize {
        let mut count = 0;
        self.search(limit, &mut count);
        count
    }

    fn search(&mut self, limit: usize, count: &mut usize) {
        if *count >= limit {
            return;
        }

        // If root's right points to root, the matrix is empty → solution found
        if self.nodes[ROOT].right == ROOT {
            *count += 1;
            return;
        }

        // Choose the column with the fewest rows (S heuristic)
        let col = self.choose_col();
        if col == ROOT {
            return; // no columns left but matrix non-empty? shouldn't happen
        }
        if self.sizes[col] == 0 {
            return; // no rows cover this column → dead end
        }

        self.cover(col);

        let mut row = self.nodes[col].down;
        while row != col {
            // Cover all other columns in this row
            let mut j = self.nodes[row].right;
            while j != row {
                self.cover(self.nodes[j].col);
                j = self.nodes[j].right;
            }

            self.search(limit, count);

            // Uncover in reverse
            j = self.nodes[row].left;
            while j != row {
                self.uncover(self.nodes[j].col);
                j = self.nodes[j].left;
            }

            if *count >= limit {
                self.uncover(col);
                return;
            }

            row = self.nodes[row].down;
        }

        self.uncover(col);
    }

    fn choose_col(&self) -> usize {
        let mut best = ROOT;
        let mut best_size = usize::MAX;
        let mut j = self.nodes[ROOT].right;
        while j != ROOT {
            if self.sizes[j] < best_size {
                best_size = self.sizes[j];
                best = j;
            }
            j = self.nodes[j].right;
        }
        best
    }

    fn cover(&mut self, col: usize) {
        // Unlink column header
        let r = self.nodes[col].right;
        let l = self.nodes[col].left;
        self.nodes[l].right = r;
        self.nodes[r].left = l;

        // Remove all rows in this column
        let mut i = self.nodes[col].down;
        while i != col {
            let mut j = self.nodes[i].right;
            while j != i {
                let u = self.nodes[j].up;
                let d = self.nodes[j].down;
                self.nodes[u].down = d;
                self.nodes[d].up = u;
                self.sizes[self.nodes[j].col] -= 1;
                j = self.nodes[j].right;
            }
            i = self.nodes[i].down;
        }
    }

    fn uncover(&mut self, col: usize) {
        // Re-add rows in reverse order
        let mut i = self.nodes[col].up;
        while i != col {
            let mut j = self.nodes[i].left;
            while j != i {
                let u = self.nodes[j].up;
                let d = self.nodes[j].down;
                self.nodes[u].down = j;
                self.nodes[d].up = j;
                self.sizes[self.nodes[j].col] += 1;
                j = self.nodes[j].left;
            }
            i = self.nodes[i].up;
        }
        // Re-link column header
        let r = self.nodes[col].right;
        let l = self.nodes[col].left;
        self.nodes[l].right = col;
        self.nodes[r].left = col;
    }
}
