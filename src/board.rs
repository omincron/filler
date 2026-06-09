/// Represents the full game board (Anfield).
#[derive(Debug, Clone)]
pub struct Board {
    /// Grid cells stored in row-major order: grid[row][col].
    pub grid: Vec<Vec<char>>,
    /// Number of columns.
    pub width: usize,
    /// Number of rows.
    pub height: usize,
    /// 1 for player 1, 2 for player 2.
    pub player_id: u8,
}

impl Board {
    /// Create a Board from a flat grid (row-major), dimensions, and player id.
    pub fn new(grid: Vec<Vec<char>>, width: usize, height: usize, player_id: u8) -> Self {
        Board { grid, width, height, player_id }
    }

    /// Return the set of characters that belong to the current player.
    pub fn own_chars(&self) -> [char; 2] {
        if self.player_id == 1 {
            ['@', 'a']
        } else {
            ['$', 's']
        }
    }

    /// Return the set of characters that belong to the opponent.
    pub fn opponent_chars(&self) -> [char; 2] {
        if self.player_id == 1 {
            ['$', 's']
        } else {
            ['@', 'a']
        }
    }

    /// Returns the cell at (row, col), or None if out of bounds.
    pub fn get(&self, row: usize, col: usize) -> Option<char> {
        self.grid.get(row)?.get(col).copied()
    }

    /// Find the approximate centroid of the player's own territory.
    /// Returns (row, col) as floats.
    pub fn own_centroid(&self) -> (f64, f64) {
        let own = self.own_chars();
        let mut sum_r = 0usize;
        let mut sum_c = 0usize;
        let mut count = 0usize;
        for r in 0..self.height {
            for c in 0..self.width {
                if own.contains(&self.grid[r][c]) {
                    sum_r += r;
                    sum_c += c;
                    count += 1;
                }
            }
        }
        if count == 0 {
            (self.height as f64 / 2.0, self.width as f64 / 2.0)
        } else {
            (sum_r as f64 / count as f64, sum_c as f64 / count as f64)
        }
    }

    /// Find the approximate centroid of the opponent's territory.
    /// Returns (row, col) as floats.
    pub fn opponent_centroid(&self) -> (f64, f64) {
        let opp = self.opponent_chars();
        let mut sum_r = 0usize;
        let mut sum_c = 0usize;
        let mut count = 0usize;
        for r in 0..self.height {
            for c in 0..self.width {
                if opp.contains(&self.grid[r][c]) {
                    sum_r += r;
                    sum_c += c;
                    count += 1;
                }
            }
        }
        if count == 0 {
            // Opponent not found: use opposite corner from own centroid
            let (or_, oc) = self.own_centroid();
            (self.height as f64 - 1.0 - or_, self.width as f64 - 1.0 - oc)
        } else {
            (sum_r as f64 / count as f64, sum_c as f64 / count as f64)
        }
    }
}

/// Represents one game piece.
#[derive(Debug, Clone)]
pub struct Piece {
    /// Cells of the piece stored in row-major order.
    pub cells: Vec<Vec<char>>,
    /// Number of columns.
    pub width: usize,
    /// Number of rows.
    pub height: usize,
}

impl Piece {
    pub fn new(cells: Vec<Vec<char>>, width: usize, height: usize) -> Self {
        Piece { cells, width, height }
    }

    /// Returns true if the cell at (row, col) in the piece is filled.
    /// Filled cells are `O`, `#`, or any non-`.` character.
    pub fn is_filled(&self, row: usize, col: usize) -> bool {
        match self.cells.get(row).and_then(|r| r.get(col)) {
            Some(&c) => c != '.',
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_board(rows: Vec<&str>, player_id: u8) -> Board {
        let height = rows.len();
        let width = if height > 0 { rows[0].len() } else { 0 };
        let grid: Vec<Vec<char>> = rows.iter().map(|r| r.chars().collect()).collect();
        Board::new(grid, width, height, player_id)
    }

    #[test]
    fn test_own_chars_p1() {
        let b = make_board(vec!["..."], 1);
        assert_eq!(b.own_chars(), ['@', 'a']);
    }

    #[test]
    fn test_own_chars_p2() {
        let b = make_board(vec!["..."], 2);
        assert_eq!(b.own_chars(), ['$', 's']);
    }

    #[test]
    fn test_opponent_chars_p1() {
        let b = make_board(vec!["..."], 1);
        assert_eq!(b.opponent_chars(), ['$', 's']);
    }

    #[test]
    fn test_opponent_chars_p2() {
        let b = make_board(vec!["..."], 2);
        assert_eq!(b.opponent_chars(), ['@', 'a']);
    }

    #[test]
    fn test_get_in_bounds() {
        let b = make_board(vec!["@..", "..."], 1);
        assert_eq!(b.get(0, 0), Some('@'));
        assert_eq!(b.get(0, 1), Some('.'));
    }

    #[test]
    fn test_get_out_of_bounds() {
        let b = make_board(vec!["@.."], 1);
        assert_eq!(b.get(5, 0), None);
        assert_eq!(b.get(0, 10), None);
    }

    #[test]
    fn test_own_centroid() {
        // Player 1 with '@' at (0,0) and (0,2) → centroid row=0, col=1
        let b = make_board(vec!["@.@", "..."], 1);
        let (r, c) = b.own_centroid();
        assert!((r - 0.0).abs() < 1e-9);
        assert!((c - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_opponent_centroid_fallback() {
        // No opponent cells → fallback to opposite corner
        let b = make_board(vec!["@..", "..."], 1);
        let (r, c) = b.opponent_centroid();
        // Should be the opposite area (non-zero, non-negative)
        assert!(r >= 0.0);
        assert!(c >= 0.0);
    }

    #[test]
    fn test_piece_is_filled() {
        let cells = vec![vec!['.', 'O'], vec!['#', '.']];
        let p = Piece::new(cells, 2, 2);
        assert!(!p.is_filled(0, 0));
        assert!(p.is_filled(0, 1));
        assert!(p.is_filled(1, 0));
        assert!(!p.is_filled(1, 1));
    }

    #[test]
    fn test_piece_is_filled_out_of_bounds() {
        let cells = vec![vec!['O']];
        let p = Piece::new(cells, 1, 1);
        assert!(!p.is_filled(5, 5));
    }
}
