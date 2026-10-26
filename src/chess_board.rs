use std::{cmp::min, fmt};

use crate::chess_board::{ChessErr::*, ChessPiece::*, PieceColor::*};

pub mod fen;

const BOARD_W: usize = 8;
const BOARD_H: usize = 8;

#[derive(Copy, Clone, Debug)]
pub enum ChessPiece {
    Empty,

    WhitePawn,
    WhiteKnight,
    WhiteBishop,
    WhiteRook,
    WhiteQueen,
    WhiteKing,

    BlackPawn,
    BlackKnight,
    BlackBishop,
    BlackRook,
    BlackQueen,
    BlackKing,
}

#[derive(Copy, Clone, Debug)]
pub enum PieceColor {
    White = 0,
    Black = 1,
}

pub enum CastlingRights {
    WhiteKing = 0b0001,
    WhiteQueen = 0b0010,
    BlackKing = 0b0100,
    BlackQueen = 0b1000,
}

enum MoveType {
    LongStraight,
    LongDiagonal,
    LongAll,
    WhiteForward,
    BlackForward,
    Single,
    Knight,
}

#[derive(Debug)]
pub enum ChessErr {
    InvalidMove,
}

#[allow(dead_code)]
pub struct ChessBoard {
    tiles: [ChessPiece; BOARD_W * BOARD_H],
    active_color: PieceColor,
    castling_rights: u8,
    half_moves: u16,
    move_count: u16,
    en_passant: isize,
}

impl ChessBoard {
    fn san_to_idx(&self, m: &str) -> Result<usize, ChessErr> {
        let m = m.as_bytes();
        if m[0] < 'a' as u8 || m[0] > 'h' as u8 {
            // hardcoded board size
            return Err(InvalidMove);
        }
        if m[1] < '1' as u8 || m[1] > '8' as u8 {
            // hardcoded board size
            return Err(InvalidMove);
        }

        let x = (m[0] - 'a' as u8) as usize;
        let y = (m[1] - '1' as u8) as usize;
        let y = BOARD_H - y - 1;

        Ok(x + y * BOARD_W)
    }

    fn get_squares(&self, idx: usize, move_type: MoveType) -> [isize; 64] {
        let mut moves = [-1_isize; BOARD_W * BOARD_H];
        let mut moves_count = 0;
        let mut moves_push = |x| {
            moves[moves_count] = x as isize;
            moves_count += 1;
        };

        match move_type {
            MoveType::LongStraight => {
                for i in 1..BOARD_W - idx % BOARD_W {
                    if let Some(tile) = self.tiles.get(idx + i) {
                        match tile {
                            Empty => moves_push(idx + i),
                            _ => break,
                        }
                    } else {
                        break;
                    }
                }
                for i in 1..idx % BOARD_W + 1 {
                    if let Some(tile) = self.tiles.get(idx - i) {
                        match tile {
                            Empty => moves_push(idx - i),
                            _ => break,
                        }
                    } else {
                        break;
                    }
                }
                for i in 1..BOARD_H - idx / BOARD_W {
                    if let Some(tile) = self.tiles.get(idx + i * BOARD_W) {
                        match tile {
                            Empty => moves_push(idx + i * BOARD_W),
                            _ => break,
                        }
                    } else {
                        break;
                    }
                }
                for i in 1..idx / BOARD_W + 1 {
                    if let Some(tile) = self.tiles.get(idx - i * BOARD_W) {
                        match tile {
                            Empty => moves_push(idx - i * BOARD_W),
                            _ => break,
                        }
                    } else {
                        break;
                    }
                }
            }
            MoveType::LongDiagonal => {
                for i in 1..min(BOARD_W - idx % BOARD_W, BOARD_H - idx / BOARD_W) {
                    if let Some(tile) = self.tiles.get(idx + i * (BOARD_W + 1)) {
                        match tile {
                            Empty => moves_push(idx + i * (BOARD_W + 1)),
                            _ => break,
                        }
                    } else {
                        break;
                    }
                }
                for i in 1..min(idx % BOARD_W + 1, BOARD_H - idx / BOARD_W) {
                    if let Some(tile) = self.tiles.get(idx + i * (BOARD_W - 1)) {
                        match tile {
                            Empty => moves_push(idx + i * (BOARD_W - 1)),
                            _ => break,
                        }
                    } else {
                        break;
                    }
                }
                for i in 1..min(idx % BOARD_W + 1, idx / BOARD_W + 1) {
                    println!("i={i}, idx={idx}");
                    if let Some(tile) = self.tiles.get(idx - i * (BOARD_W + 1)) {
                        match tile {
                            Empty => moves_push(idx - i * (BOARD_W + 1)),
                            _ => break,
                        }
                    } else {
                        break;
                    }
                }
                for i in 1..min(BOARD_W - idx % BOARD_W, idx / BOARD_W + 1) {
                    if let Some(tile) = self.tiles.get(idx - i * (BOARD_W - 1)) {
                        match tile {
                            Empty => moves_push(idx - i * (BOARD_W - 1)),
                            _ => break,
                        }
                    } else {
                        break;
                    }
                }
            }
            MoveType::WhiteForward => {
                if idx / BOARD_W == BOARD_H - 2 {
                    for i in 0..2 {
                        if let Empty = self.tiles[idx - BOARD_W * (i + 1)] {
                            moves[i] = (idx - BOARD_W * (i + 1)) as isize;
                        } else {
                            break;
                        }
                    }
                } else if idx >= BOARD_W {
                    if let Empty = self.tiles[idx - BOARD_W] {
                        moves[0] = (idx - BOARD_W) as isize;
                    }
                }
            }
            MoveType::BlackForward => {
                if idx / BOARD_W == 1 {
                    for i in 0..2 {
                        if let Empty = self.tiles[idx + BOARD_W * (i + 1)] {
                            moves[i] = (idx + BOARD_W * (i + 1)) as isize;
                        } else {
                            break;
                        }
                    }
                } else if idx < BOARD_W * (BOARD_H - 1) {
                    if let Empty = self.tiles[idx + BOARD_W] {
                        moves[0] = (idx + BOARD_W) as isize;
                    }
                }
            }
            MoveType::Single => {
                if idx % BOARD_W > 0 && idx / BOARD_W > 0 {
                    moves_push(idx - BOARD_W - 1);
                }
                if idx % BOARD_W > 0 {
                    moves_push(idx - 1);
                }
                if idx / BOARD_W > 0 {
                    moves_push(idx - BOARD_W);
                }
                if idx % BOARD_W < BOARD_W - 1 && idx / BOARD_W > 0 {
                    moves_push(idx - BOARD_W + 1);
                }
                if idx % BOARD_W < BOARD_W - 1 {
                    moves_push(idx + 1);
                }
                if idx % BOARD_W < BOARD_W - 1 && idx / BOARD_W < BOARD_H - 1 {
                    moves_push(idx + BOARD_W + 1);
                }
                if idx / BOARD_W < BOARD_H - 1 {
                    moves_push(idx + BOARD_W);
                }
                if idx % BOARD_W > 0 && idx / BOARD_W < BOARD_H - 1 {
                    moves_push(idx + BOARD_W - 1);
                }
            }
            MoveType::Knight => {
                if let Empty = self.tiles[idx + 2 + 1 * BOARD_W] {
                    moves_push(idx + 2 + 1 * BOARD_W);
                }
                if let Empty = self.tiles[idx - 2 + 1 * BOARD_W] {
                    moves_push(idx - 2 + 1 * BOARD_W);
                }
                if let Empty = self.tiles[idx - 2 - 1 * BOARD_W] {
                    moves_push(idx - 2 - 1 * BOARD_W);
                }
                if let Empty = self.tiles[idx + 2 - 1 * BOARD_W] {
                    moves_push(idx + 2 - 1 * BOARD_W);
                }
                if let Empty = self.tiles[idx + 1 + 2 * BOARD_W] {
                    moves_push(idx + 1 + 2 * BOARD_W);
                }
                if let Empty = self.tiles[idx - 1 + 2 * BOARD_W] {
                    moves_push(idx - 1 + 2 * BOARD_W);
                }
                if let Empty = self.tiles[idx - 1 - 2 * BOARD_W] {
                    moves_push(idx - 1 - 2 * BOARD_W);
                }
                if let Empty = self.tiles[idx + 1 - 2 * BOARD_W] {
                    moves_push(idx + 1 - 2 * BOARD_W);
                }
            }
            MoveType::LongAll => {
                let straight = self.get_squares(idx, MoveType::LongStraight);
                let diagonal = self.get_squares(idx, MoveType::LongDiagonal);
                for m in straight {
                    if m == -1 {
                        break;
                    }
                    moves_push(m as usize);
                }
                for m in diagonal {
                    moves_push(m as usize);
                    if m == -1 {
                        break;
                    }
                }
            }
        }

        return moves;
    }
    // supported only up to board sizes 9x9
    pub fn make_move(&mut self, m: &str) -> Result<(), ChessErr> {
        let target = self.san_to_idx(m).unwrap();

        let sqrs = self.get_squares(target, MoveType::LongStraight);
        for s in sqrs {
            println!("{s}");
            if s < 0 {
                break;
            }
            self.tiles[s as usize] = ChessPiece::WhiteKing;
        }
        // todo!();
        Ok(())
    }
}

impl fmt::Display for ChessBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..BOARD_W * BOARD_H {
            let piece = self.tiles[i];
            if i % BOARD_W == BOARD_W - 1 && i != BOARD_W * BOARD_H - 1 {
                write!(f, " {} \n", piece)?;
            } else {
                write!(f, " {} ", piece)?;
            }
        }

        // write!(f, "\ncastling_rights = {}\n", self.castling_rights)?;
        // write!(f, "move_count = {}\n", self.move_count)?;
        // write!(f, "half_moves = {}\n", self.half_moves)?;
        // write!(f, "active_color = {}\n", self.active_color as u8)?;
        // write!(f, "en_passant = {}\n", self.en_passant)?;

        Ok(())
    }
}

impl fmt::Display for ChessPiece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Empty => write!(f, "."),

            WhitePawn => write!(f, "P"),
            WhiteKnight => write!(f, "N"),
            WhiteBishop => write!(f, "B"),
            WhiteRook => write!(f, "R"),
            WhiteQueen => write!(f, "Q"),
            WhiteKing => write!(f, "K"),

            BlackPawn => write!(f, "p"),
            BlackKnight => write!(f, "n"),
            BlackBishop => write!(f, "b"),
            BlackRook => write!(f, "r"),
            BlackQueen => write!(f, "q"),
            BlackKing => write!(f, "k"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starting_position() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

        let board = ChessBoard::from_fen(fen).expect("Invalid FEN string");

        let expected = " r  n  b  q  k  b  n  r 
 p  p  p  p  p  p  p  p 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 P  P  P  P  P  P  P  P 
 R  N  B  Q  K  B  N  R ";

        assert_eq!(format!("{board}"), expected);
    }

    #[test]
    fn test_empty_board() {
        let fen = "8/8/8/8/8/8/8/8 w - - 0 1";
        let board = ChessBoard::from_fen(fen).expect("Invalid FEN string");

        let expected = " .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . ";

        assert_eq!(format!("{board}"), expected);
    }

    #[test]
    fn test_black_to_move() {
        let fen = "8/8/8/3k4/8/4K3/8/8 b - - 0 25";
        let board = ChessBoard::from_fen(fen).expect("Invalid FEN string");

        let expected = " .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  k  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  K  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . ";

        assert_eq!(format!("{board}"), expected);
    }

    #[test]
    fn test_white_kingside_castling() {
        let fen = "4k3/8/8/8/8/8/8/4K2R w K - 0 1";
        let board = ChessBoard::from_fen(fen).expect("Invalid FEN string");

        let expected = " .  .  .  .  k  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  K  .  .  R ";

        assert_eq!(format!("{board}"), expected);
    }

    #[test]
    fn test_white_queenside_castling() {
        let fen = "4k3/8/8/8/8/8/8/R3K3 w Q - 0 1";
        let board = ChessBoard::from_fen(fen).expect("Invalid FEN string");

        let expected = " .  .  .  .  k  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 R  .  .  .  K  .  .  . ";

        assert_eq!(format!("{board}"), expected);
    }

    #[test]
    fn test_all_castling_rights() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
        let board = ChessBoard::from_fen(fen).expect("Invalid FEN string");

        let expected = " r  .  .  .  k  .  .  r 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 R  .  .  .  K  .  .  R ";

        assert_eq!(format!("{board}"), expected);
    }

    #[test]
    fn test_en_passant_e3() {
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let board = ChessBoard::from_fen(fen).expect("Invalid FEN string");

        let expected = " r  n  b  q  k  b  n  r 
 p  p  p  p  p  p  p  p 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  P  .  .  . 
 .  .  .  .  .  .  .  . 
 P  P  P  P  .  P  P  P 
 R  N  B  Q  K  B  N  R ";

        assert_eq!(format!("{board}"), expected);
    }

    #[test]
    fn test_en_passant_d6() {
        let fen = "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3";
        let board = ChessBoard::from_fen(fen).expect("Invalid FEN string");

        let expected = " r  n  b  q  k  b  n  r 
 p  p  p  .  p  p  p  p 
 .  .  .  .  .  .  .  . 
 .  .  .  p  P  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 P  P  P  P  .  P  P  P 
 R  N  B  Q  K  B  N  R ";

        assert_eq!(format!("{board}"), expected);
    }

    #[test]
    fn test_halfmove_and_fullmove_counters() {
        let fen = "4k3/8/8/8/3Q4/8/8/4K3 b - - 99 42";
        let board = ChessBoard::from_fen(fen).expect("Invalid FEN string");

        let expected = " .  .  .  .  k  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  Q  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  .  .  .  . 
 .  .  .  .  K  .  .  . ";

        assert_eq!(format!("{board}"), expected);
    }

    #[test]
    fn test_all_piece_types() {
        let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
        let board = ChessBoard::from_fen(fen).expect("Invalid FEN string");

        let expected = " r  .  .  .  k  .  .  r 
 P  p  p  p  .  p  p  p 
 .  b  .  .  .  n  b  N 
 n  P  .  .  .  .  .  . 
 B  B  P  .  P  .  .  . 
 q  .  .  .  .  N  .  . 
 P  p  .  P  .  .  P  P 
 R  .  .  Q  .  R  K  . ";

        assert_eq!(format!("{board}"), expected);
    }

    #[test]
    fn test_invalid_fens() {
        let invalid_fens = [
            "",
            "8/8/8/8/8/8/8 w - - 0 1",
            "8/8/8/8/8/8/8/8/8 w - - 0 1",
            "8/8/8/8/8/8/8/7 w - - 0 1",
            "8/8/8/8/8/8/8/9 w - - 0 1",
            "8/8/8/8/8/8/8/8 x - - 0 1",
            "8/8/8/8/8/8/8/8 w ABCD - 0 1",
            "8/8/8/8/8/8/8/8 w - z9 0 1",
            "8/8/8/8/8/8/8/8 w - - -1 1",
            "8/8/8/8/8/8/8/8 w - - 0 0",
            "8/8/8/8/8/8/8/8 w - - 0",
            "8/8/8/8/8/8/1P7/8 w KQkq - 0 1",
            "8/8/8/8/8/8/PPPPPPPPP/8 w KQkq - 0 1",
        ];

        for fen in invalid_fens {
            assert!(
                ChessBoard::from_fen(fen).is_err(),
                "Expected invalid FEN: {fen}"
            );
        }
    }
}
