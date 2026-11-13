use std::io::{Read, Write};
use std::ops::{Index, IndexMut};
use std::{cmp::min, fmt};

use crate::chess_board::{ChessErr::*, ChessPiece::*, PieceColor::*};

pub mod fen;

pub const BOARD_W: usize = 8;
pub const BOARD_H: usize = 8;
const COLOR_BIT: u8 = 0b0001;

const WHITE_KINGSIDE: u8 = 0b0001;
const WHITE_QUEENSIDE: u8 = 0b0010;
const BLACK_KINGSIDE: u8 = 0b0100;
const BLACK_QUEENSIDE: u8 = 0b1000;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PieceColor {
    White = 0,
    Black = 1,
}

#[derive(Copy, Clone, PartialEq)]
pub enum ChessPiece {
    Empty = 0,

    WhitePawn = 0b0010,
    WhiteKnight = 0b0100,
    WhiteBishop = 0b0110,
    WhiteRook = 0b1000,
    WhiteQueen = 0b1010,
    WhiteKing = 0b1100,

    BlackPawn = 0b0011,
    BlackKnight = 0b0101,
    BlackBishop = 0b0111,
    BlackRook = 0b1001,
    BlackQueen = 0b1011,
    BlackKing = 0b1101,
}

#[derive(Debug)]
pub enum ChessErr {
    InvalidMove,
    InvalidPiece,
    InvalidPromotion,
    InvalidSAN,
    IoError,
    InvalidColor,
    EmptyTile,
    CheckMove,
}

pub struct Moves([bool; BOARD_W * BOARD_H]);

#[derive(Clone, Copy)]
pub struct SAN {
    rank: u8,
    file: u8,
}

#[allow(dead_code)]
pub struct ChessBoard {
    tiles: [ChessPiece; BOARD_W * BOARD_H],
    pub active_color: PieceColor,
    castling_rights: u8,
    half_moves: u16,
    move_count: u16,
    en_passant: Option<usize>,
}

impl ChessPiece {
    fn get_color(&self) -> PieceColor {
        if *self as u8 & COLOR_BIT == White as u8 {
            White
        } else {
            Black
        }
    }

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

            _ => return Err(InvalidPiece),
        })
    }

    pub fn from_input(prompt: &str) -> Result<ChessPiece, ChessErr> {
        let mut buf = [0_u8; 1];
        std::io::stdout()
            .write(prompt.as_bytes())
            .map_err(|_| IoError)?;
        std::io::stdout().flush().map_err(|_| IoError)?;
        std::io::stdin().read_exact(&mut buf).map_err(|_| IoError)?;
        std::io::stdin()
            .read_exact(&mut [0; 1])
            .map_err(|_| IoError)?;
        ChessPiece::from_char(buf[0] as char)
    }
}

impl PieceColor {
    pub fn switch(&mut self) {
        match self {
            White => *self = Black,
            Black => *self = White,
        }
    }
}

impl Index<usize> for Moves {
    type Output = bool;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Moves {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Moves {
    pub fn new() -> Moves {
        Moves {
            0: [false; BOARD_W * BOARD_H],
        }
    }
}

impl SAN {
    pub fn new(m: [u8; 2]) -> Result<SAN, ChessErr> {
        // NOTE: hardcoded board size

        if m[0] < 'a' as u8 || m[0] > 'h' as u8 {
            return Err(InvalidSAN);
        }
        if m[1] < '1' as u8 || m[1] > '8' as u8 {
            return Err(InvalidSAN);
        }

        let file = m[0] - 'a' as u8;
        let rank = m[1] - '1' as u8;

        Ok(SAN { rank, file })
    }

    pub fn from_input(prompt: &str) -> Result<SAN, ChessErr> {
        let mut buf = [0_u8; 2];
        std::io::stdout()
            .write(prompt.as_bytes())
            .map_err(|_| IoError)?;
        std::io::stdout().flush().map_err(|_| IoError)?;
        std::io::stdin().read_exact(&mut buf).map_err(|_| IoError)?;
        std::io::stdin()
            .read_exact(&mut [0; 1])
            .map_err(|_| IoError)?;
        SAN::new(buf)
    }

    fn to_idx(&self) -> Result<usize, ChessErr> {
        let y = BOARD_H - self.rank as usize - 1;

        Ok(self.file as usize + y * BOARD_W)
    }
}

impl ChessBoard {
    fn check_continous_tiles(
        &self,
        moves: &mut Moves,
        start: usize,
        amount: usize,
        f: impl Fn(usize) -> usize,
    ) {
        let color = self.tiles[start].get_color();

        for t in 1..amount {
            let dest = f(t);
            let tile = self.tiles[dest];
            if tile == Empty {
                moves[dest] = true;
            } else if tile.get_color() != color {
                moves[dest] = true;
                break;
            } else {
                break;
            }
        }
    }

    fn check_jump_tile(
        &self,
        moves: &mut Moves,
        idx: usize,
        (dx, dy): (isize, isize),
        move_type: ChessPiece,
    ) {
        let x = idx % BOARD_W;
        let y = idx / BOARD_W;

        let dest_x = x as isize + dx;
        let dest_y = y as isize + dy;
        let o = dx + dy * BOARD_W as isize;

        if dest_x < 0 || dest_x >= BOARD_W as isize {
            return;
        }
        if dest_y < 0 || dest_y >= BOARD_H as isize {
            return;
        }
        let i = (idx as isize + o) as usize;
        let tile = self.tiles[i];
        if tile == Empty || tile.get_color() != move_type.get_color() {
            moves[i] = true;
        }
    }

    fn get_squares(&self, idx: usize, move_type: ChessPiece) -> Moves {
        let mut moves: Moves = Moves::new();

        match move_type {
            WhiteRook | BlackRook => {
                self.check_continous_tiles(
                    // right
                    &mut moves,
                    idx,
                    BOARD_W - idx % BOARD_W,
                    |i| idx + i,
                );
                self.check_continous_tiles(
                    // left
                    &mut moves,
                    idx,
                    idx % BOARD_W + 1,
                    |i| idx - i,
                );
                self.check_continous_tiles(
                    // down
                    &mut moves,
                    idx,
                    BOARD_H - idx / BOARD_W,
                    |i| idx + i * BOARD_W,
                );
                self.check_continous_tiles(
                    // up
                    &mut moves,
                    idx,
                    idx / BOARD_W + 1,
                    |i| idx - i * BOARD_W,
                );
            }
            WhiteBishop | BlackBishop => {
                self.check_continous_tiles(
                    &mut moves,
                    idx,
                    min(BOARD_W - idx % BOARD_W, BOARD_H - idx / BOARD_W),
                    |i| idx + i * (BOARD_W + 1),
                );
                self.check_continous_tiles(
                    &mut moves,
                    idx,
                    min(idx % BOARD_W + 1, BOARD_H - idx / BOARD_W),
                    |i| idx + i * (BOARD_W - 1),
                );
                self.check_continous_tiles(
                    &mut moves,
                    idx,
                    min(idx % BOARD_W + 1, idx / BOARD_W + 1),
                    |i| idx - i * (BOARD_W + 1),
                );
                self.check_continous_tiles(
                    &mut moves,
                    idx,
                    min(BOARD_W - idx % BOARD_W, idx / BOARD_W + 1),
                    |i| idx - i * (BOARD_W - 1),
                );
            }
            WhitePawn | BlackPawn => {
                let is_end_rank = match move_type.get_color() {
                    White => idx / BOARD_W == 0,
                    Black => idx / BOARD_W == BOARD_H - 1,
                };

                if is_end_rank {
                    return moves;
                }

                let is_double_move = match move_type.get_color() {
                    White => idx / BOARD_W == BOARD_H - 2,
                    Black => idx / BOARD_W == 1,
                };

                if is_double_move {
                    for i in 0..2 {
                        let target = match move_type.get_color() {
                            White => idx - BOARD_W * (i + 1),
                            Black => idx + BOARD_W * (i + 1),
                        };
                        if let Empty = self.tiles[target] {
                            moves[target] = true;
                        } else {
                            break;
                        }
                    }
                } else if idx >= BOARD_W {
                    let target = match move_type.get_color() {
                        White => idx - BOARD_W,
                        Black => idx + BOARD_W,
                    };
                    // single move
                    if let Empty = self.tiles[target] {
                        moves[target] = true;
                    }
                }

                // capture left
                if idx % BOARD_W > 0 {
                    let target_left = match move_type.get_color() {
                        White => idx - BOARD_W - 1,
                        Black => idx + BOARD_W - 1,
                    };
                    let tile = self.tiles[target_left];
                    let is_en_passant = match self.en_passant {
                        Some(en_passant) => en_passant == idx - 1,
                        None => false,
                    };
                    if (tile != Empty && tile.get_color() != move_type.get_color()) || is_en_passant
                    {
                        moves[target_left] = true;
                    }
                }

                // capture right
                if idx % BOARD_W < BOARD_W - 1 {
                    let target_right = match move_type.get_color() {
                        White => idx - BOARD_W + 1,
                        Black => idx + BOARD_W + 1,
                    };
                    let tile = self.tiles[target_right];
                    let is_en_passant = match self.en_passant {
                        Some(en_passant) => en_passant == idx + 1,
                        None => false,
                    };
                    if (tile != Empty && tile.get_color() != move_type.get_color()) || is_en_passant
                    {
                        moves[target_right] = true;
                    }
                }
            }
            WhiteKing | BlackKing => {
                let king_pos = match move_type.get_color() {
                    White => BOARD_W / 2 + BOARD_W * (BOARD_H - 1),
                    Black => BOARD_W / 2,
                };
                let king_rook = king_pos + BOARD_W / 2 - 1;
                let queen_rook = king_pos - BOARD_W / 2;
                let mut can_queenside_castle = true;
                let mut can_kingside_castle = true;

                // home rank is empty
                for i in king_pos + 1..king_rook - 1 {
                    if self.tiles[i] != Empty {
                        can_kingside_castle = false;
                        break;
                    }
                }
                for i in queen_rook + 1..king_pos - 1 {
                    if self.tiles[i] != Empty {
                        can_queenside_castle = false;
                        break;
                    }
                }

                // castling right is valid
                match move_type.get_color() {
                    White => {
                        if self.castling_rights & WHITE_KINGSIDE == 0 {
                            can_kingside_castle = false;
                        }
                        if self.castling_rights & WHITE_QUEENSIDE == 0 {
                            can_queenside_castle = false;
                        }
                    }
                    Black => {
                        if self.castling_rights & BLACK_KINGSIDE == 0 {
                            can_kingside_castle = false;
                        }
                        if self.castling_rights & BLACK_QUEENSIDE == 0 {
                            can_queenside_castle = false;
                        }
                    }
                }

                if can_kingside_castle {
                    moves[idx + 2] = true;
                }
                if can_queenside_castle {
                    moves[idx - 2] = true;
                }

                self.check_jump_tile(&mut moves, idx, (1, 0), move_type);
                self.check_jump_tile(&mut moves, idx, (1, 1), move_type);
                self.check_jump_tile(&mut moves, idx, (0, 1), move_type);
                self.check_jump_tile(&mut moves, idx, (-1, 1), move_type);
                self.check_jump_tile(&mut moves, idx, (-1, 0), move_type);
                self.check_jump_tile(&mut moves, idx, (-1, -1), move_type);
                self.check_jump_tile(&mut moves, idx, (0, -1), move_type);
                self.check_jump_tile(&mut moves, idx, (1, -1), move_type);
            }
            WhiteKnight | BlackKnight => {
                self.check_jump_tile(&mut moves, idx, (2, 1), move_type);
                self.check_jump_tile(&mut moves, idx, (1, 2), move_type);
                self.check_jump_tile(&mut moves, idx, (-1, 2), move_type);
                self.check_jump_tile(&mut moves, idx, (-2, 1), move_type);
                self.check_jump_tile(&mut moves, idx, (-2, -1), move_type);
                self.check_jump_tile(&mut moves, idx, (-1, -2), move_type);
                self.check_jump_tile(&mut moves, idx, (1, -2), move_type);
                self.check_jump_tile(&mut moves, idx, (2, -1), move_type);
            }
            WhiteQueen => {
                let straight = self.get_squares(idx, WhiteRook);
                let diagonal = self.get_squares(idx, WhiteBishop);
                for i in 0..BOARD_W * BOARD_H {
                    moves[i] |= straight[i] | diagonal[i];
                }
            }
            BlackQueen => {
                let straight = self.get_squares(idx, BlackRook);
                let diagonal = self.get_squares(idx, BlackBishop);
                for i in 0..BOARD_W * BOARD_H {
                    moves[i] |= straight[i] | diagonal[i];
                }
            }
            Empty => {}
        }

        return moves;
    }

    fn is_check(&self, color: PieceColor) -> bool {
        let king = match color {
            White => WhiteKing,
            Black => BlackKing,
        };
        let king_idx = match self.tiles.iter().position(|&x| x == king) {
            Some(idx) => idx,
            None => {
                eprintln!("what, no king?");
                return false;
            }
        };

        let move_types: [ChessPiece; 6] = match color {
            White => [
                WhitePawn,
                WhiteKnight,
                WhiteBishop,
                WhiteRook,
                WhiteQueen,
                WhiteKing,
            ],
            Black => [
                BlackPawn,
                BlackKnight,
                BlackBishop,
                BlackRook,
                BlackQueen,
                BlackKing,
            ],
        };

        for mt in move_types {
            let moves = self.get_squares(king_idx, mt);
            for i in 0..BOARD_W * BOARD_H {
                if !moves[i] {
                    continue;
                }

                if self.tiles[i] as u8 == mt as u8 ^ COLOR_BIT {
                    return true;
                }
            }
        }

        return false;
    }

    pub fn get_moves(&self, san: SAN, color: PieceColor) -> Result<Moves, ChessErr> {
        // NOTE: supported only up to board sizes 9x9
        // TODO: special moves: en-passant
        let target = san.to_idx()?;
        if let Empty = self.tiles[target] {
            return Err(EmptyTile);
        }
        if self.tiles[target].get_color() != color {
            return Err(InvalidColor);
        }

        Ok(self.get_squares(target, self.tiles[target]))
    }

    pub fn make_move_coord(
        &mut self,
        start: SAN,
        end: SAN,
        promotion_callback: impl Fn() -> ChessPiece,
    ) -> Result<(), ChessErr> {
        let moves = self.get_moves(start, self.active_color)?;
        let start = start.to_idx()?;
        let end = end.to_idx()?;

        if !moves[end] {
            return Err(InvalidMove);
        }

        let king_pos = match self.active_color {
            White => BOARD_W / 2 + BOARD_W * (BOARD_H - 1),
            Black => BOARD_W / 2,
        };
        let king = match self.active_color {
            White => WhiteKing,
            Black => BlackKing,
        };

        let is_castling = self.tiles[start] == king && (end == king_pos + 2 || end == king_pos - 2);
        let is_promotion = match self.active_color {
            White => self.tiles[start] == WhitePawn && end / BOARD_W == 0,
            Black => self.tiles[start] == BlackPawn && end / BOARD_W == BOARD_H - 1,
        };
        let is_en_passant_candidate = match self.active_color {
            White => self.tiles[start] == WhitePawn && start / BOARD_W == 6 && end / BOARD_W == 4,
            Black => self.tiles[start] == BlackPawn && start / BOARD_W == 1 && end / BOARD_W == 3,
        };
        let is_en_passant_capture = match self.active_color {
            White => self.tiles[start] == WhitePawn && Some(end + BOARD_W) == self.en_passant,
            Black => self.tiles[start] == BlackPawn && Some(end - BOARD_W) == self.en_passant,
        };

        if is_castling {
            let king_rook = king_pos + BOARD_W / 2 - 1;
            let queen_rook = king_pos - BOARD_W / 2;
            self.tiles[end] = king;
            self.tiles[king_pos] = Empty;

            if end == king_pos + 2 {
                self.tiles[king_pos + 1] = self.tiles[king_rook];
                self.tiles[king_rook] = Empty;
            } else if end == king_pos - 2 {
                self.tiles[king_pos - 1] = self.tiles[queen_rook];
                self.tiles[queen_rook] = Empty;
            } else {
                eprintln!("what, weird castling?");
            }
        } else if is_promotion {
            let piece = promotion_callback();
            if self.tiles[start].get_color() != piece.get_color() {
                return Err(InvalidColor);
            }

            match piece {
                WhitePawn | WhiteKing | BlackPawn | BlackKing | Empty => {
                    return Err(InvalidPromotion);
                }
                WhiteKnight | WhiteBishop | WhiteRook | WhiteQueen | BlackKnight | BlackBishop
                | BlackRook | BlackQueen => {
                    self.tiles[start] = Empty;
                    self.tiles[end] = piece;
                }
            }
        } else {
            let captured = self.tiles[end];
            self.tiles[end] = self.tiles[start];
            self.tiles[start] = Empty;

            if self.is_check(self.active_color) {
                self.tiles[start] = self.tiles[end];
                self.tiles[end] = captured;
                return Err(CheckMove);
            }
        }

        if is_en_passant_candidate {
            self.en_passant = Some(end);
        } else {
            self.en_passant = None;
        }

        match self.active_color {
            White => {
                if is_en_passant_capture {
                    self.tiles[end + BOARD_W] = Empty;
                }
                if self.tiles[end] == WhiteKing {
                    self.castling_rights &= !(WHITE_KINGSIDE | WHITE_QUEENSIDE);
                } else if self.tiles[end] == WhiteRook {
                    if start % BOARD_W == BOARD_W - 1 {
                        self.castling_rights &= !WHITE_KINGSIDE;
                    } else if start % BOARD_W == 0 {
                        self.castling_rights &= !WHITE_QUEENSIDE;
                    }
                }
            }
            Black => {
                if is_en_passant_capture {
                    self.tiles[end - BOARD_W] = Empty;
                }
                if self.tiles[end] == BlackKing {
                    self.castling_rights &= !(BLACK_KINGSIDE | BLACK_QUEENSIDE);
                } else if self.tiles[end] == BlackRook {
                    if start % BOARD_W == BOARD_W - 1 {
                        self.castling_rights &= !BLACK_KINGSIDE;
                    } else if start % BOARD_W == 0 {
                        self.castling_rights &= !BLACK_QUEENSIDE;
                    }
                }
            }
        }
        self.active_color.switch();
        Ok(())
    }

    pub fn display(&self, selected: Moves, color: PieceColor) {
        let mut last_move: usize = 0;
        if let Black = color {
            last_move = BOARD_W * BOARD_H - 1;
            while last_move > 0 && !selected[last_move] {
                last_move -= 1;
            }
        }

        for i in 0..BOARD_W * BOARD_H {
            let i = match color {
                White => i,
                Black => BOARD_W * BOARD_H - i - 1,
            };
            let is_end_ln = match color {
                White => i % BOARD_W == BOARD_W - 1,
                Black => i % BOARD_W == 0,
            };
            let is_start_ln = match color {
                White => i % BOARD_W == 0,
                Black => i % BOARD_W == BOARD_W - 1,
            };
            let piece = self.tiles[i];

            if is_start_ln {
                print!("{} ", BOARD_W - (i / BOARD_W));
            }

            if selected[i] {
                match color {
                    White => last_move += 1,
                    Black => {
                        if last_move > 0 {
                            last_move -= 1;
                        }
                    }
                };
                print!("({})", piece);
            } else {
                print!(" {} ", piece);
            }

            if is_end_ln {
                println!();
            }
        }

        match color {
            White => println!("   a  b  c  d  e  f  g  h "),
            Black => println!("   h  g  f  e  d  c  b  a "),
        }
        println!();
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

        Ok(())
    }
}

impl fmt::Display for ChessPiece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Empty => write!(f, "."),
            WhitePawn => write!(f, "♙"),
            WhiteKnight => write!(f, "♘"),
            WhiteBishop => write!(f, "♗"),
            WhiteRook => write!(f, "♖"),
            WhiteQueen => write!(f, "♕"),
            WhiteKing => write!(f, "♔"),

            BlackPawn => write!(f, "♟"),
            BlackKnight => write!(f, "♞"),
            BlackBishop => write!(f, "♝"),
            BlackRook => write!(f, "♜"),
            BlackQueen => write!(f, "♛"),
            BlackKing => write!(f, "♚"),
        }
    }
}

impl fmt::Debug for ChessPiece {
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
