#![allow(dead_code)]
pub use crate::engine::*;
pub use crate::utils::*;
use colored::*;

// Board position for the start of a new game
pub const DEFAULT_FEN_STRING: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/*
    Example Piece: 0b10000101 = WHITE | QUEEN
    1st bit: Color 1 = White, 0 = Black
    2-5 bit: Unused
    6-8 bit: Piece identifier
*/

pub const COLOR_MASK: u8 = 0b10000000;
pub const WHITE: u8 = 0b10000000;
pub const BLACK: u8 = 0b00000000;

pub const PIECE_MASK: u8 = 0b00000111;
pub const PAWN: u8 = 0b00000001;
pub const KNIGHT: u8 = 0b00000010;
pub const BISHOP: u8 = 0b00000011;
pub const ROOK: u8 = 0b00000100;
pub const QUEEN: u8 = 0b00000101;
pub const KING: u8 = 0b00000110;

pub const EMPTY: u8 = 0;
pub const SENTINEL: u8 = 0b11111111;

pub const BOARD_START: usize = 2;
pub const BOARD_END: usize = 10;

type Point = (usize, usize);

pub fn get_color(square: u8) -> Option<PieceColor> {
    if is_empty(square) || is_outside_board(square) {
        return None;
    }
    if square & COLOR_MASK == WHITE {
        return Some(PieceColor::White);
    }
    Some(PieceColor::Black)
}

pub fn is_white(square: u8) -> bool {
    !is_empty(square) && square & COLOR_MASK == WHITE
}

pub fn is_black(square: u8) -> bool {
    !is_empty(square) && square & COLOR_MASK == BLACK
}

pub fn is_pawn(square: u8) -> bool {
    square & PIECE_MASK == PAWN
}

pub fn is_knight(square: u8) -> bool {
    square & PIECE_MASK == KNIGHT
}

pub fn is_bishop(square: u8) -> bool {
    square & PIECE_MASK == BISHOP
}

pub fn is_rook(square: u8) -> bool {
    square & PIECE_MASK == ROOK
}

pub fn is_queen(square: u8) -> bool {
    square & PIECE_MASK == QUEEN
}

pub fn is_king(square: u8) -> bool {
    square & PIECE_MASK == KING
}

pub fn is_empty(square: u8) -> bool {
    square == EMPTY
}

pub fn is_outside_board(square: u8) -> bool {
    square == SENTINEL
}

/*
    Returns the row, col on the board when given the algebraic coordinates
*/
pub fn algebraic_pairs_to_board_position(pair: &str) -> Option<Point> {
    if pair.len() != 2 {
        return None;
    }

    let c = pair.chars().nth(0).unwrap();
    let r = pair.chars().nth(1).unwrap();
    let col = match c {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => return None,
    };

    let row = BOARD_END - (r.to_digit(10).unwrap() as usize);
    if !(BOARD_START..BOARD_END).contains(&row) {
        return None;
    }

    Some((row, col + BOARD_START))
}

pub fn board_position_to_algebraic_pair(pair: Point) -> String {
    let row = match pair.0 {
        2 => "8",
        3 => "7",
        4 => "6",
        5 => "5",
        6 => "4",
        7 => "3",
        8 => "2",
        9 => "1",
        _ => "1",
    };
    let col = match pair.1 {
        2 => "a",
        3 => "b",
        4 => "c",
        5 => "d",
        6 => "e",
        7 => "f",
        8 => "g",
        9 => "h",
        _ => "h",
    };
    col.to_string() + &row.to_string()
}

fn get_piece_character(piece: u8) -> &'static str {
    match piece & PIECE_MASK {
        PAWN => "♟︎",
        KNIGHT => "♞",
        BISHOP => "♝",
        ROOK => "♜",
        QUEEN => "♛",
        KING => "♚",
        _ => " ",
    }
}

fn get_piece_character_simple(piece: u8) -> &'static str {
    if is_white(piece) {
        match piece & PIECE_MASK {
            PAWN => "P",
            KNIGHT => "N",
            BISHOP => "B",
            ROOK => "R",
            QUEEN => "Q",
            KING => "K",
            _ => " ",
        }
    } else {
        match piece & PIECE_MASK {
            PAWN => "p",
            KNIGHT => "n",
            BISHOP => "b",
            ROOK => "r",
            QUEEN => "q",
            KING => "k",
            _ => " ",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PieceColor {
    Black,
    White,
}

impl PieceColor {
    pub fn as_mask(&self) -> u8 {
        match *self {
            PieceColor::White => WHITE,
            PieceColor::Black => BLACK,
        }
    }
}

#[derive(Clone)]
pub struct BoardState {
    pub full_move_clock: u8, // The number of the full moves. It starts at 1, and is incremented after Black's move
    pub half_move_clock: u8, // The number of half moves since the last capture or pawn advance, used for the fifty-move rule
    pub board: [[u8; 12]; 12],
    pub to_move: PieceColor,
    // if a pawn, on the last move, made a double move, this is set, otherwise this is None
    pub pawn_double_move: Option<Point>,
    pub white_king_location: Point,
    pub black_king_location: Point,
    pub white_king_side_castle: bool,
    pub white_queen_side_castle: bool,
    pub black_king_side_castle: bool,
    pub black_queen_side_castle: bool,
    pub black_total_piece_value: i32,
    pub white_total_piece_value: i32,
    pub last_move: Option<String>, // the start and last position of the last move made
}

impl BoardState {
    pub fn pretty_print_board(&self) {
        println!("a b c d e f g h");
        for i in BOARD_START..BOARD_END {
            for j in BOARD_START..BOARD_END {
                let piece = format!("{} ", get_piece_character(self.board[i][j]));
                if (i + j) % 2 != 0 {
                    if is_white(self.board[i][j]) {
                        print!("{}", piece.white().on_truecolor(158, 93, 30));
                    } else {
                        print!("{}", piece.black().on_truecolor(158, 93, 30));
                    }
                } else if is_white(self.board[i][j]) {
                    print!("{}", piece.white().on_truecolor(205, 170, 125));
                } else {
                    print!("{}", piece.black().on_truecolor(205, 170, 125));
                }
            }
            println!(" {}", 10 - i);
        }
    }

    pub fn simple_board(&self) -> String {
        let mut board = "\na b c d e f g h\n".to_string();
        for i in BOARD_START..BOARD_END {
            for j in BOARD_START..BOARD_END {
                board = format!("{}{} ", board, get_piece_character_simple(self.board[i][j]));
            }
            board = format!("{} {}\n", board, 10 - i);
        }
        board
    }

    pub fn simple_print_board(&self) {
        print!("{}", self.simple_board());
    }

    pub fn swap_color(&mut self) {
        match self.to_move {
            PieceColor::White => self.to_move = PieceColor::Black,
            PieceColor::Black => self.to_move = PieceColor::White,
        }
    }
}

/*
    Parse the standard fen string notation (en.wikipedia.org/wiki/Forsyth–Edwards_Notation) and return a board state
*/
pub fn board_from_fen(fen: &str) -> Result<BoardState, &str> {
    let mut board = [[SENTINEL; 12]; 12];
    let mut fen = fen.to_string();
    trim_newline(&mut fen);
    let fen_config: Vec<&str> = fen.split(' ').collect();
    if fen_config.len() != 6 {
        return Err("Could not parse fen string: Invalid fen string");
    }

    let to_move = match fen_config[1] {
        "w" => PieceColor::White,
        "b" => PieceColor::Black,
        _ => return Err("Could not parse fen string: Next player to move was not provided"),
    };
    let castling_privileges = fen_config[2];
    let en_passant = fen_config[3];

    let half_move_clock = fen_config[4].parse::<u8>();
    if half_move_clock.is_err() {
        return Err("Could not parse fen string: Invalid half move value");
    }

    let full_move_clock = fen_config[5].parse::<u8>();
    if full_move_clock.is_err() {
        return Err("Could not parse fen string: Invalid full move value");
    }

    let fen_rows: Vec<&str> = fen_config[0].split('/').collect();

    if fen_rows.len() != 8 {
        return Err("Could not parse fen string: Invalid number of rows provided, 8 expected");
    }

    let mut row: usize = BOARD_START;
    let mut col: usize = BOARD_START;
    let mut white_king_location = (0, 0);
    let mut black_king_location = (0, 0);
    let mut white_piece_values = 0;
    let mut black_piece_values = 0;
    for fen_row in fen_rows {
        for square in fen_row.chars() {
            if square.is_digit(10) {
                let square_skip_count = square.to_digit(10).unwrap() as usize;
                if square_skip_count + col > BOARD_END {
                    return Err("Could not parse fen string: Index out of bounds");
                }
                for _ in 0..square_skip_count {
                    board[row][col] = EMPTY;
                    col += 1;
                }
            } else {
                board[row][col] = match get_piece_from_fen_string_char(square) {
                    Some(piece) => piece,
                    None => return Err("Could not parse fen string: Invalid character found"),
                };

                if is_white(board[row][col]) {
                    white_piece_values += PIECE_VALUES[(board[row][col] & PIECE_MASK) as usize];
                    if is_king(board[row][col]) {
                        white_king_location = (row, col);
                    }
                } else {
                    black_piece_values += PIECE_VALUES[(board[row][col] & PIECE_MASK) as usize];
                    if is_king(board[row][col]) {
                        black_king_location = (row, col);
                    }
                }
                col += 1;
            }
        }
        if col != BOARD_END {
            return Err("Could not parse fen string: Complete row was not specified");
        }
        row += 1;
        col = BOARD_START;
    }

    // Deal with the en passant string
    let mut en_passant_pos: Option<Point> = None;
    if en_passant.len() != 2 {
        if en_passant != "-" {
            return Err("Could not parse fen string: En passant string not valid");
        }
    } else {
        en_passant_pos = algebraic_pairs_to_board_position(en_passant);
    }

    Ok(BoardState {
        full_move_clock: full_move_clock.unwrap(),
        half_move_clock: half_move_clock.unwrap(),
        board,
        to_move,
        white_king_location,
        black_king_location,
        pawn_double_move: en_passant_pos,
        white_king_side_castle: castling_privileges.find('K') != None,
        white_queen_side_castle: castling_privileges.find('Q') != None,
        black_king_side_castle: castling_privileges.find('k') != None,
        black_queen_side_castle: castling_privileges.find('q') != None,
        black_total_piece_value: black_piece_values,
        white_total_piece_value: white_piece_values,
        last_move: None,
    })
}

fn get_piece_from_fen_string_char(piece: char) -> Option<u8> {
    match piece {
        'r' => Some(BLACK | ROOK),
        'n' => Some(BLACK | KNIGHT),
        'b' => Some(BLACK | BISHOP),
        'q' => Some(BLACK | QUEEN),
        'k' => Some(BLACK | KING),
        'p' => Some(BLACK | PAWN),
        'R' => Some(WHITE | ROOK),
        'N' => Some(WHITE | KNIGHT),
        'B' => Some(WHITE | BISHOP),
        'Q' => Some(WHITE | QUEEN),
        'K' => Some(WHITE | KING),
        'P' => Some(WHITE | PAWN),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pieces_recognized() {
        assert!(is_white(WHITE | BISHOP));
        assert!(is_white(WHITE | ROOK));
        assert!(is_white(WHITE | KING));
        assert!(is_white(WHITE | PAWN));

        assert!(is_black(BLACK | BISHOP));
        assert!(is_black(BLACK | ROOK));
        assert!(is_black(BLACK | KING));
        assert!(is_black(BLACK | PAWN));

        assert!(is_pawn(WHITE | PAWN));
        assert!(is_pawn(BLACK | PAWN));
        assert!(!is_pawn(BLACK | ROOK));

        assert!(is_knight(WHITE | KNIGHT));
        assert!(is_knight(BLACK | KNIGHT));
        assert!(!is_knight(WHITE | QUEEN));

        assert!(is_bishop(WHITE | BISHOP));
        assert!(is_bishop(BLACK | BISHOP));
        assert!(!is_bishop(WHITE | ROOK));

        assert!(is_queen(WHITE | QUEEN));
        assert!(is_queen(BLACK | QUEEN));
        assert!(!is_queen(WHITE | PAWN));

        assert!(is_king(WHITE | KING));
        assert!(is_king(BLACK | KING));
        assert!(!is_king(WHITE | QUEEN));

        assert!(is_empty(EMPTY));
        assert!(!is_empty(WHITE | KING));

        assert!(is_outside_board(SENTINEL));
        assert!(!is_outside_board(EMPTY));
        assert!(!is_outside_board(WHITE | KING));
    }

    // Algebraic translation tests

    #[test]
    fn algebraic_translation_correct() {
        let res = algebraic_pairs_to_board_position("a8").unwrap();
        assert_eq!(res.0, BOARD_START);
        assert_eq!(res.1, BOARD_START);

        let res = algebraic_pairs_to_board_position("h1").unwrap();
        assert_eq!(res.0, BOARD_END - 1);
        assert_eq!(res.1, BOARD_END - 1);

        let res = algebraic_pairs_to_board_position("a6").unwrap();
        assert_eq!(res.0, BOARD_START + 2);
        assert_eq!(res.1, BOARD_START);

        let res = algebraic_pairs_to_board_position("c5").unwrap();
        assert_eq!(res.0, BOARD_START + 3);
        assert_eq!(res.1, BOARD_START + 2);
    }

    #[test]
    #[should_panic]
    fn algebraic_translation_panic_col() {
        algebraic_pairs_to_board_position("z1").unwrap();
    }

    #[test]
    #[should_panic]
    fn algebraic_translation_panic_long() {
        algebraic_pairs_to_board_position("a11").unwrap();
    }

    #[test]
    fn points_to_long_algebraic_position_test() {
        let res = board_position_to_algebraic_pair((2, 2));
        assert_eq!(res, "a8");

        let res = board_position_to_algebraic_pair((4, 6));
        assert_eq!(res, "e6");
    }

    // Fen string tests

    #[test]
    fn empty_board() {
        let b = board_from_fen("8/8/8/8/8/8/8/8 w KQkq - 0 1").unwrap();
        for i in BOARD_START..BOARD_END {
            for j in BOARD_START..BOARD_END {
                assert_eq!(b.board[i][j], EMPTY);
            }
        }
    }

    #[test]
    fn starting_pos() {
        let b = board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert_eq!(b.board[2][2], BLACK | ROOK);
        assert_eq!(b.board[2][3], BLACK | KNIGHT);
        assert_eq!(b.board[2][4], BLACK | BISHOP);
        assert_eq!(b.board[2][5], BLACK | QUEEN);
        assert_eq!(b.board[2][6], BLACK | KING);
        assert_eq!(b.board[2][7], BLACK | BISHOP);
        assert_eq!(b.board[2][8], BLACK | KNIGHT);
        assert_eq!(b.board[2][9], BLACK | ROOK);

        for i in BOARD_START..BOARD_END {
            assert_eq!(b.board[3][i], BLACK | PAWN);
        }

        for i in 4..8 {
            for j in BOARD_START..BOARD_END {
                assert_eq!(b.board[i][j], EMPTY);
            }
        }

        assert_eq!(b.board[9][2], WHITE | ROOK);
        assert_eq!(b.board[9][3], WHITE | KNIGHT);
        assert_eq!(b.board[9][4], WHITE | BISHOP);
        assert_eq!(b.board[9][5], WHITE | QUEEN);
        assert_eq!(b.board[9][6], WHITE | KING);
        assert_eq!(b.board[9][7], WHITE | BISHOP);
        assert_eq!(b.board[9][8], WHITE | KNIGHT);
        assert_eq!(b.board[9][9], WHITE | ROOK);

        for i in BOARD_START..BOARD_END {
            assert_eq!(b.board[8][i], WHITE | PAWN);
        }

        assert_eq!(
            b.white_total_piece_value,
            20000 + 900 + 2 * 500 + 2 * 330 + 2 * 320 + 8 * 100
        );
        assert_eq!(
            b.black_total_piece_value,
            20000 + 900 + 2 * 500 + 2 * 330 + 2 * 320 + 8 * 100
        );
    }

    #[test]
    fn correct_en_passant_privileges() {
        let b =
            board_from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e4 0 1").unwrap();
        assert_eq!(b.pawn_double_move.unwrap().0, BOARD_START + 4);
        assert_eq!(b.pawn_double_move.unwrap().1, BOARD_START + 4);
    }

    #[test]
    fn correct_en_passant_privileges_black() {
        let b =
            board_from_fen("rnbqkbnr/ppppppp1/8/7p/8/8/PPPPPPPP/RNBQKBNR w KQkq h5 0 1").unwrap();
        assert_eq!(b.pawn_double_move.unwrap().0, BOARD_START + 3);
        assert_eq!(b.pawn_double_move.unwrap().1, BOARD_START + 7);
    }

    #[test]
    fn correct_king_location() {
        let b = board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert_eq!(b.black_king_location, (2, 6));
        assert_eq!(b.white_king_location, (9, 6));
    }

    #[test]
    fn correct_king_location_two() {
        let b = board_from_fen("6rk/1b4np/5pp1/1p6/8/1P3NP1/1B3P1P/5RK1 w KQkq - 0 1").unwrap();
        assert_eq!(b.black_king_location, (2, 9));
        assert_eq!(b.white_king_location, (9, 8));
    }

    #[test]
    fn correct_starting_player() {
        let mut b =
            board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert_eq!(b.to_move, PieceColor::White);
        b = board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1").unwrap();
        assert_eq!(b.to_move, PieceColor::Black);
    }

    #[test]
    fn correct_castling_privileges() {
        let mut b = board_from_fen("6rk/1b4np/5pp1/1p6/8/1P3NP1/1B3P1P/5RK1 w KQkq - 0 1").unwrap();
        assert!(b.black_king_side_castle);
        assert!(b.black_queen_side_castle);
        assert!(b.white_king_side_castle);
        assert!(b.white_queen_side_castle);

        b = board_from_fen("6rk/1b4np/5pp1/1p6/8/1P3NP1/1B3P1P/5RK1 w - - 0 1").unwrap();
        assert!(!b.black_king_side_castle);
        assert!(!b.black_queen_side_castle);
        assert!(!b.white_king_side_castle);
        assert!(!b.white_queen_side_castle);

        b = board_from_fen("6rk/1b4np/5pp1/1p6/8/1P3NP1/1B3P1P/5RK1 w Kq - 0 1").unwrap();
        assert!(!b.black_king_side_castle);
        assert!(b.black_queen_side_castle);
        assert!(b.white_king_side_castle);
        assert!(!b.white_queen_side_castle);
    }

    #[test]
    fn random_pos() {
        let b = board_from_fen("4R1B1/1kp5/1B1Q4/1P5p/1p2p1pK/8/3pP3/4N1b1 w - - 0 1").unwrap();
        assert_eq!(b.board[2][6], WHITE | ROOK);
        assert_eq!(b.board[2][8], WHITE | BISHOP);
        assert_eq!(b.board[3][3], BLACK | KING);
        assert_eq!(b.board[3][4], BLACK | PAWN);
        assert_eq!(b.board[4][3], WHITE | BISHOP);
        assert_eq!(b.board[4][5], WHITE | QUEEN);
        assert_eq!(b.board[5][3], WHITE | PAWN);
        assert_eq!(b.board[5][9], BLACK | PAWN);
        assert_eq!(b.board[6][3], BLACK | PAWN);
        assert_eq!(b.board[6][6], BLACK | PAWN);
        assert_eq!(b.board[6][8], BLACK | PAWN);
        assert_eq!(b.board[6][9], WHITE | KING);
        assert_eq!(b.board[8][5], BLACK | PAWN);
        assert_eq!(b.board[8][6], WHITE | PAWN);
        assert_eq!(b.board[9][6], WHITE | KNIGHT);
        assert_eq!(b.board[9][8], BLACK | BISHOP);
    }

    #[test]
    #[should_panic]
    fn bad_fen_string() {
        board_from_fen("this isn't a fen string").unwrap();
    }

    #[test]
    #[should_panic]
    fn bad_fen_string_bad_char() {
        board_from_fen("rnbqkbnH/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    }

    #[test]
    #[should_panic]
    fn bad_fen_string_too_many_chars() {
        board_from_fen("rnbqkbnrrrrr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    }
}
