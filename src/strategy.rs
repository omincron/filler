use crate::board::{Board, Piece};

/// Result of checking whether placing `piece` at column `x`, row `y` is valid.
#[derive(Debug, PartialEq)]
pub enum PlacementResult {
    /// Placement is valid; contains the number of own-overlaps (always 1).
    Valid,
    /// Piece would go out of Anfield bounds.
    OutOfBounds,
    /// Piece overlaps opponent territory.
    OpponentOverlap,
    /// Number of own-cell overlaps is not exactly 1.
    WrongOwnOverlap(usize),
}

/// Check whether placing `piece` with its top-left corner at board column `x`,
/// board row `y` is a valid move for the player described in `board`.
///
/// Validity requires:
/// 1. All filled piece cells are within the Anfield.
/// 2. Exactly one filled cell overlaps the player's own territory.
/// 3. Zero filled cells overlap the opponent's territory.
pub fn check_placement(board: &Board, piece: &Piece, x: usize, y: usize) -> PlacementResult {
    let own = board.own_chars();
    let opp = board.opponent_chars();

    let mut own_overlaps = 0usize;

    for pr in 0..piece.height {
        for pc in 0..piece.width {
            if !piece.is_filled(pr, pc) {
                continue;
            }
            let br = y + pr;
            let bc = x + pc;

            // Bounds check
            if br >= board.height || bc >= board.width {
                return PlacementResult::OutOfBounds;
            }

            let cell = board.grid[br][bc];

            if opp.contains(&cell) {
                return PlacementResult::OpponentOverlap;
            }

            if own.contains(&cell) {
                own_overlaps += 1;
                if own_overlaps > 1 {
                    // Early-exit: already invalid
                    return PlacementResult::WrongOwnOverlap(own_overlaps);
                }
            }
        }
    }

    if own_overlaps == 1 {
        PlacementResult::Valid
    } else {
        PlacementResult::WrongOwnOverlap(own_overlaps)
    }
}

/// Find the best valid placement for `piece` on `board`.
///
/// Strategy:
///   - Enumerate all (x, y) positions where the piece fits.
///   - Filter to valid placements (exactly 1 own-overlap, 0 opponent-overlap,
///     fully within bounds).
///   - Score each valid placement by how far its piece center moves toward
///     the opponent's centroid (i.e., minimise distance to opponent centroid).
///   - Return the best (x, y), or None if no valid placement exists.
pub fn find_placement(board: &Board, piece: &Piece) -> Option<(usize, usize)> {
    let (opp_r, opp_c) = board.opponent_centroid();

    let mut best: Option<(usize, usize)> = None;
    let mut best_score = f64::MAX;

    // y iterates rows, x iterates cols for the top-left corner of the piece.
    // The piece must fit fully within the board, so the maximum top-left position
    // is (width - piece.width, height - piece.height).
    // We use saturating arithmetic: if piece is larger than board, no placements exist.
    let max_y = board.height.saturating_sub(piece.height);
    let max_x = board.width.saturating_sub(piece.width);

    for y in 0..=max_y {
        for x in 0..=max_x {
            if check_placement(board, piece, x, y) == PlacementResult::Valid {
                // Compute the center of this piece placement on the board
                let center_r = y as f64 + piece.height as f64 / 2.0;
                let center_c = x as f64 + piece.width as f64 / 2.0;

                // Distance from piece center to opponent centroid (lower = closer = better)
                let dr = center_r - opp_r;
                let dc = center_c - opp_c;
                let dist = dr * dr + dc * dc;

                if dist < best_score {
                    best_score = dist;
                    best = Some((x, y));
                }
            }
        }
    }

    best
}

/// Format the placement output as required by the game engine: `X Y\n`.
pub fn format_output(x: usize, y: usize) -> String {
    format!("{} {}\n", x, y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Board, Piece};

    // Helper: build a Board from string rows.
    fn board_from(rows: &[&str], player_id: u8) -> Board {
        let height = rows.len();
        let width = if height > 0 { rows[0].len() } else { 0 };
        let grid: Vec<Vec<char>> = rows.iter().map(|r| r.chars().collect()).collect();
        Board::new(grid, width, height, player_id)
    }

    // Helper: build a Piece from string rows.
    fn piece_from(rows: &[&str]) -> Piece {
        let height = rows.len();
        let width = if height > 0 { rows[0].len() } else { 0 };
        let cells: Vec<Vec<char>> = rows.iter().map(|r| r.chars().collect()).collect();
        Piece::new(cells, width, height)
    }

    // --- Valid Placement Logic tests (required area 2) ---

    #[test]
    fn test_valid_placement_exactly_one_own_overlap() {
        // 5×5 board, player 1 has '@' at (2,2)
        let board = board_from(
            &[
                ".....",
                ".....",
                "..@..",
                ".....",
                ".....",
            ],
            1,
        );
        // Single-cell piece 'O'
        let piece = piece_from(&["O"]);
        // Placing at (2,2) means the piece (row=2, col=2) lands on '@' → 1 own-overlap, valid
        assert_eq!(check_placement(&board, &piece, 2, 2), PlacementResult::Valid);
    }

    #[test]
    fn test_valid_placement_two_own_overlaps_rejected() {
        // Board with two own cells adjacent
        let board = board_from(
            &[
                ".....",
                "..@@.",
                ".....",
            ],
            1,
        );
        // 1×2 horizontal piece covering both '@' cells
        let piece = piece_from(&["OO"]);
        // Placing at col=2, row=1 covers grid[1][2]='@' and grid[1][3]='@' → 2 overlaps
        let result = check_placement(&board, &piece, 2, 1);
        assert!(matches!(result, PlacementResult::WrongOwnOverlap(2)));
    }

    #[test]
    fn test_valid_placement_opponent_overlap_rejected() {
        // Board with opponent '$' at (1,2)
        let board = board_from(
            &[
                ".....",
                "..$...",
                "..@..",
                ".....",
            ],
            1,
        );
        // Piece that would cover the opponent cell
        let piece = piece_from(&["O"]);
        // Place at (2,1) → lands on '$'
        assert_eq!(
            check_placement(&board, &piece, 2, 1),
            PlacementResult::OpponentOverlap
        );
    }

    #[test]
    fn test_valid_placement_no_own_overlap_rejected() {
        let board = board_from(
            &[
                ".....",
                "..@..",
                ".....",
            ],
            1,
        );
        let piece = piece_from(&["O"]);
        // Placing on an empty cell → 0 own overlaps
        let result = check_placement(&board, &piece, 0, 0);
        assert!(matches!(result, PlacementResult::WrongOwnOverlap(0)));
    }

    // --- Boundary Conditions tests (required area 3) ---

    #[test]
    fn test_placement_out_of_bounds_right() {
        let board = board_from(&["@....", "....."], 1);
        // 3-wide piece placed at col=4 on a 5-wide board: col 4+2=6 >= 5 → out of bounds
        let piece = piece_from(&["OOO"]);
        assert_eq!(check_placement(&board, &piece, 4, 0), PlacementResult::OutOfBounds);
    }

    #[test]
    fn test_placement_out_of_bounds_bottom() {
        let board = board_from(&["@....", "....."], 1);
        // 2-tall piece placed at row=2 on a 2-tall board: row 2+1=3 >= 2 → out of bounds
        let piece = piece_from(&["O", "O"]);
        assert_eq!(check_placement(&board, &piece, 0, 2), PlacementResult::OutOfBounds);
    }

    #[test]
    fn test_placement_exactly_at_edge_is_valid() {
        // 3-wide board, player 1 '@' at (0,2). 1-wide piece placed at col=2, row=0.
        let board = board_from(&["..@", "..."], 1);
        let piece = piece_from(&["O"]);
        assert_eq!(check_placement(&board, &piece, 2, 0), PlacementResult::Valid);
    }

    #[test]
    fn test_piece_wider_than_board_no_valid_placement() {
        let board = board_from(&["@.."], 1);
        let piece = piece_from(&["OOOO"]); // 4 wide, board is 3 wide
        // max_x = 3.saturating_sub(4) = 0, loop 0..=0 → only x=0 checked, out-of-bounds
        let result = find_placement(&board, &piece);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_placement_returns_valid_move() {
        // 5×5 board, p1 '@' at centre (2,2), opponent '$' at (0,0)
        let board = board_from(
            &[
                "$....",
                ".....",
                "..@..",
                ".....",
                ".....",
            ],
            1,
        );
        let piece = piece_from(&["O"]);
        let result = find_placement(&board, &piece);
        assert!(result.is_some());
        let (x, y) = result.unwrap();
        // Must be valid
        assert_eq!(check_placement(&board, &piece, x, y), PlacementResult::Valid);
    }

    #[test]
    fn test_find_placement_no_valid_move() {
        // Completely filled board except own cell → no room
        let board = board_from(
            &[
                "@$$",
                "$$$",
            ],
            1,
        );
        let piece = piece_from(&["OO"]);
        // Every 2-wide placement either goes out of bounds or hits opponent
        let result = find_placement(&board, &piece);
        assert!(result.is_none());
    }

    #[test]
    fn test_strategy_moves_toward_opponent() {
        // Board: p1 has a 2-cell territory at rows 3-4, cols 8-9 (bottom-right area).
        // p2 '$' is at top-left corner (0,0).
        // A 1×2 piece can be placed either at (8,3) — overlapping (8,4) = '@',
        // extending toward top    →  center ≈ (3.5, 9.0), dist to (0,0) smaller
        // or at (8,4) — overlapping (8,4) = '@', extending toward bottom.
        // Strategy should prefer the placement whose center is closer to (0,0).
        let board = board_from(
            &[
                "$.........",  // row 0
                "..........",  // row 1
                "..........",  // row 2
                "........@.",  // row 3  — own cell at (3, 8)
                "........@.",  // row 4  — own cell at (4, 8)
            ],
            1,
        );
        // Vertical 2-tall piece
        let piece = piece_from(&["O", "O"]);
        let result = find_placement(&board, &piece);
        assert!(result.is_some());
        let (x, y) = result.unwrap();
        // Placement must be valid
        assert_eq!(check_placement(&board, &piece, x, y), PlacementResult::Valid);
        // The centre of the chosen placement (in row-space) must be ≤ 3.5
        // (i.e. the piece is placed toward the top, not hanging below row 4).
        // Valid placements with exactly 1 own-overlap:
        //   (8, 2): covers (2,8)='.' and (3,8)='@' → 1 overlap, valid, center_r=3.0
        //   (8, 3): covers (3,8)='@' and (4,8)='@' → 2 overlaps, INVALID
        //   (8, 4): covers (4,8)='@' and (5,8)=OOB → out of bounds
        // So only (8, 2) is valid — it moves toward the opponent at top-left.
        assert_eq!(x, 8);
        assert_eq!(y, 2);
    }

    // --- Coordinate Output tests (required area 4) ---

    #[test]
    fn test_format_output_basic() {
        assert_eq!(format_output(7, 2), "7 2\n");
    }

    #[test]
    fn test_format_output_zero_zero() {
        assert_eq!(format_output(0, 0), "0 0\n");
    }

    #[test]
    fn test_format_output_large_coords() {
        assert_eq!(format_output(100, 200), "100 200\n");
    }

    #[test]
    fn test_format_output_ends_with_newline() {
        let out = format_output(3, 5);
        assert!(out.ends_with('\n'));
    }

    #[test]
    fn test_format_output_has_space_separator() {
        let out = format_output(3, 5);
        let parts: Vec<&str> = out.trim().split(' ').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "3");
        assert_eq!(parts[1], "5");
    }
}
