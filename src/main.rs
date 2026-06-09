use std::io::{self, BufRead, Write};

// Re-use the library modules.
use filler_lib::board::Piece;
use filler_lib::parser::{parse_anfield, parse_piece, parse_player_id};
use filler_lib::strategy::{find_placement, format_output};

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    let mut lines = stdin.lock().lines();

    // ── First line: determine player id ──────────────────────────────────────
    let first_line = lines
        .next()
        .expect("No input received")
        .expect("Failed to read first line");

    let player_id = parse_player_id(&first_line);

    // ── Main game loop ────────────────────────────────────────────────────────
    // Buffer for lines belonging to the current block (Anfield or Piece).
    let mut block_lines: Vec<String> = Vec::new();
    // Track what we are currently collecting.
    enum State {
        Idle,
        ReadingAnfield { header: String, cols: usize, rows: usize, rows_read: usize },
        ReadingPiece   { header: String, cols: usize, rows: usize, rows_read: usize },
    }

    let mut state = State::Idle;
    // Hold the most recently parsed board so we can pair it with a piece.
    let mut current_board: Option<filler_lib::board::Board> = None;

    /// Parse `<keyword> <cols> <rows>:` and return (cols, rows).
    fn extract_dims(header: &str, keyword: &str) -> (usize, usize) {
        let rest = header
            .trim_start_matches(keyword)
            .trim_start()
            .trim_end_matches(':')
            .trim();
        let mut parts = rest.split_whitespace();
        let cols: usize = parts.next().unwrap_or("0").parse().unwrap_or(0);
        let rows: usize = parts.next().unwrap_or("0").parse().unwrap_or(0);
        (cols, rows)
    }

    for line_result in lines {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => break,
        };

        // Dispatch based on current state and incoming line.
        state = match state {
            // ── Idle: look for a block header ────────────────────────────────
            State::Idle => {
                if line.starts_with("Anfield") {
                    let (cols, rows) = extract_dims(&line, "Anfield");
                    block_lines.clear();
                    block_lines.push(line.clone()); // push header
                    State::ReadingAnfield { header: line, cols, rows, rows_read: 0 }
                } else if line.starts_with("Piece") {
                    let (cols, rows) = extract_dims(&line, "Piece");
                    block_lines.clear();
                    block_lines.push(line.clone());
                    State::ReadingPiece { header: line, cols, rows, rows_read: 0 }
                } else {
                    State::Idle
                }
            }

            // ── Reading Anfield block ─────────────────────────────────────────
            State::ReadingAnfield { header, cols, rows, rows_read } => {
                block_lines.push(line.clone());
                let new_rows_read = rows_read + 1;

                // We need 1 column-index line + rows grid lines = rows+1 total after header
                if new_rows_read == rows + 1 {
                    // Build a string slice vector (skip the header which is block_lines[0])
                    let all: Vec<&str> = block_lines.iter().map(String::as_str).collect();
                    let mut iter = all[1..].iter().copied(); // skip header; parse_anfield receives the rest
                    let board = parse_anfield(&header, &mut iter, player_id);
                    current_board = Some(board);
                    block_lines.clear();
                    State::Idle
                } else {
                    State::ReadingAnfield { header, cols, rows, rows_read: new_rows_read }
                }
            }

            // ── Reading Piece block ───────────────────────────────────────────
            State::ReadingPiece { header, cols, rows, rows_read } => {
                block_lines.push(line.clone());
                let new_rows_read = rows_read + 1;

                if new_rows_read == rows {
                    // All piece rows collected.
                    let all: Vec<&str> = block_lines.iter().map(String::as_str).collect();
                    let mut iter = all[1..].iter().copied();
                    let piece: Piece = parse_piece(&header, &mut iter);

                    // Compute placement and output.
                    let output = if let Some(ref board) = current_board {
                        match find_placement(board, &piece) {
                            Some((x, y)) => format_output(x, y),
                            None => format_output(0, 0),
                        }
                    } else {
                        format_output(0, 0)
                    };

                    out.write_all(output.as_bytes()).ok();
                    out.flush().ok();

                    block_lines.clear();
                    State::Idle
                } else {
                    State::ReadingPiece { header, cols, rows, rows_read: new_rows_read }
                }
            }
        };
    }
}
