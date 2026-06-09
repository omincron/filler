use crate::board::{Board, Piece};

/// Parse the player id from the first line sent by the game engine.
///
/// Expected format: `$$$ exec p<N> : [<path>]`
/// Returns 1 or 2, or panics if the format is unexpected.
pub fn parse_player_id(line: &str) -> u8 {
    // Find "p1" or "p2" in the line
    for token in line.split_whitespace() {
        if token == "p1" {
            return 1;
        }
        if token == "p2" {
            return 2;
        }
    }
    panic!("Could not determine player id from line: {:?}", line);
}

/// Parse an Anfield block from a slice of lines.
///
/// `header` is the `Anfield <cols> <rows>:` line.
/// `lines` is the remaining lines iterator; it will consume exactly
/// 1 (column-index header) + rows grid lines.
///
/// Returns a partially-constructed Board (without player_id set yet;
/// caller must supply it via `Board::new`).
pub fn parse_anfield<'a, I>(header: &str, lines: &mut I, player_id: u8) -> Board
where
    I: Iterator<Item = &'a str>,
{
    // Parse "Anfield <cols> <rows>:"
    let (width, height) = parse_dimensions(header, "Anfield");

    // Skip the column-index header line (e.g. "    01234567890123456789")
    lines.next();

    let mut grid: Vec<Vec<char>> = Vec::with_capacity(height);
    for _ in 0..height {
        let row_line = lines.next().expect("Unexpected end of Anfield data");
        // Strip the 4-character prefix "NNN " (3 digits + space)
        let stripped = if row_line.len() >= 4 { &row_line[4..] } else { row_line };
        let row: Vec<char> = stripped.chars().take(width).collect();
        grid.push(row);
    }

    Board::new(grid, width, height, player_id)
}

/// Parse a Piece block from a slice of lines.
///
/// `header` is the `Piece <cols> <rows>:` line.
/// `lines` is the remaining lines iterator; it will consume exactly `rows` lines.
pub fn parse_piece<'a, I>(header: &str, lines: &mut I) -> Piece
where
    I: Iterator<Item = &'a str>,
{
    let (width, height) = parse_dimensions(header, "Piece");

    let mut cells: Vec<Vec<char>> = Vec::with_capacity(height);
    for _ in 0..height {
        let piece_line = lines.next().expect("Unexpected end of Piece data");
        let row: Vec<char> = piece_line.chars().take(width).collect();
        cells.push(row);
    }

    Piece::new(cells, width, height)
}

/// Parse `<keyword> <cols> <rows>:` and return (cols, rows).
fn parse_dimensions(header: &str, keyword: &str) -> (usize, usize) {
    // Remove the keyword prefix and trailing ':'
    let rest = header
        .trim_start_matches(keyword)
        .trim_start()
        .trim_end_matches(':')
        .trim();
    let mut parts = rest.split_whitespace();
    let cols: usize = parts
        .next()
        .unwrap_or("0")
        .parse()
        .expect("Invalid column count");
    let rows: usize = parts
        .next()
        .unwrap_or("0")
        .parse()
        .expect("Invalid row count");
    (cols, rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Input Parsing tests (required area 1) ---

    #[test]
    fn test_parse_player_id_p1() {
        let line = "$$$ exec p1 : [robots/bender]";
        assert_eq!(parse_player_id(line), 1);
    }

    #[test]
    fn test_parse_player_id_p2() {
        let line = "$$$ exec p2 : [robots/terminator]";
        assert_eq!(parse_player_id(line), 2);
    }

    #[test]
    fn test_parse_dimensions_anfield() {
        let (w, h) = super::parse_dimensions("Anfield 20 15:", "Anfield");
        assert_eq!(w, 20);
        assert_eq!(h, 15);
    }

    #[test]
    fn test_parse_dimensions_piece() {
        let (w, h) = super::parse_dimensions("Piece 4 1:", "Piece");
        assert_eq!(w, 4);
        assert_eq!(h, 1);
    }

    #[test]
    fn test_parse_anfield_dimensions_and_cells() {
        // A small 5×3 Anfield
        let raw = "\
Anfield 5 3:
    01234
000 .....
001 ..@..
002 .....";
        let mut lines = raw.lines();
        let header = lines.next().unwrap();
        let board = parse_anfield(header, &mut lines, 1);

        assert_eq!(board.width, 5);
        assert_eq!(board.height, 3);
        assert_eq!(board.grid[0], vec!['.', '.', '.', '.', '.']);
        assert_eq!(board.grid[1], vec!['.', '.', '@', '.', '.']);
        assert_eq!(board.grid[2], vec!['.', '.', '.', '.', '.']);
        assert_eq!(board.player_id, 1);
    }

    #[test]
    fn test_parse_anfield_with_opponent() {
        let raw = "\
Anfield 5 3:
    01234
000 .....
001 ..$..
002 ..@..";
        let mut lines = raw.lines();
        let header = lines.next().unwrap();
        let board = parse_anfield(header, &mut lines, 1);

        assert_eq!(board.grid[1][2], '$');
        assert_eq!(board.grid[2][2], '@');
    }

    #[test]
    fn test_parse_piece_hash_filled() {
        let raw = "\
Piece 2 2:
.#
#.";
        let mut lines = raw.lines();
        let header = lines.next().unwrap();
        let piece = parse_piece(header, &mut lines);

        assert_eq!(piece.width, 2);
        assert_eq!(piece.height, 2);
        assert!(!piece.is_filled(0, 0));
        assert!(piece.is_filled(0, 1));
        assert!(piece.is_filled(1, 0));
        assert!(!piece.is_filled(1, 1));
    }

    #[test]
    fn test_parse_piece_o_filled() {
        let raw = "\
Piece 4 1:
.OO.";
        let mut lines = raw.lines();
        let header = lines.next().unwrap();
        let piece = parse_piece(header, &mut lines);

        assert_eq!(piece.width, 4);
        assert_eq!(piece.height, 1);
        assert!(!piece.is_filled(0, 0));
        assert!(piece.is_filled(0, 1));
        assert!(piece.is_filled(0, 2));
        assert!(!piece.is_filled(0, 3));
    }

    #[test]
    fn test_parse_anfield_full_example() {
        // Mirrors the example in the instructions
        let raw = "\
Anfield 20 15:
    01234567890123456789
000 ....................
001 ....................
002 .........@..........
003 ....................
004 ....................
005 ....................
006 ....................
007 ....................
008 ....................
009 ....................
010 ....................
011 ....................
012 .........$..........
013 ....................
014 ....................";
        let mut lines = raw.lines();
        let header = lines.next().unwrap();
        let board = parse_anfield(header, &mut lines, 1);

        assert_eq!(board.width, 20);
        assert_eq!(board.height, 15);
        // Player 1 starting cell
        assert_eq!(board.grid[2][9], '@');
        // Player 2 starting cell
        assert_eq!(board.grid[12][9], '$');
        // Empty cell
        assert_eq!(board.grid[0][0], '.');
    }
}
