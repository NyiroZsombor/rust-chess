use std::fmt;

use crate::chess_board::{ChessErr::*, ChessPiece::*, PieceColor::*};

mod fen;

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

#[derive(Debug)]
pub enum ChessErr {
    InvalidBoard,
    SpaceExpected,
    EOFError,
    ActiveColorError,
    CastlingRightError,
    HalfMovesError,
    MoveCountError,
    EnPassantError,
}

pub struct ChessBoard {
    tiles: [ChessPiece; BOARD_W * BOARD_H],
    #[allow(dead_code)]
    active_color: PieceColor,
    #[allow(dead_code)]
    castling_rights: u8,
    #[allow(dead_code)]
    half_moves: u16,
    #[allow(dead_code)]
    move_count: u16,
    #[allow(dead_code)]
    en_passant: isize,
}

impl ChessPiece {
    fn from_char(c: char) -> Result<ChessPiece, ChessErr> {
        Ok(match c {
            'P' => WhitePawn,
            'N' => WhiteKnight,
            'B' => WhiteBishop,
            'R' => WhiteRook,
            'Q' => WhiteQueen,
            'K' => WhiteKing,

            'p' => BlackPawn,
            'n' => BlackKnight,
            'b' => BlackBishop,
            'r' => BlackRook,
            'q' => BlackQueen,
            'k' => BlackKing,

            _ => return Err(InvalidBoard),
        })
    }
}

impl CastlingRights {
    fn from_char(c: char) -> Result<CastlingRights, ChessErr> {
        Ok(match c {
            'K' => CastlingRights::WhiteKing,
            'Q' => CastlingRights::WhiteQueen,
            'k' => CastlingRights::BlackKing,
            'q' => CastlingRights::BlackQueen,
            _ => return Err(CastlingRightError),
        })
    }
}

impl ChessBoard {
    fn get_by_idx(&self, idx: usize) -> ChessPiece {
        self.tiles[idx]
    }
}

impl fmt::Display for ChessBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..BOARD_W * BOARD_H {
            let piece = self.get_by_idx(i as usize);
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
        ];

        for fen in invalid_fens {
            assert!(
                ChessBoard::from_fen(fen).is_err(),
                "Expected invalid FEN: {fen}"
            );
        }
    }
}
