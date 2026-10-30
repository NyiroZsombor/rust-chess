use std::{cmp::min, fmt};

use crate::chess_board::{ChessErr::*, ChessPiece::*, PieceColor::*};

pub mod fen;

pub const BOARD_W: usize = 8;
pub const BOARD_H: usize = 8;
const COLOR_BIT: u8 = 0b0001;

type SAN = [u8; 2];
type Moves = [bool; BOARD_W * BOARD_H];

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

#[derive(Copy, Clone, Debug, PartialEq)]
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
    InvalidMove,
    InvalidSAN,
    InvalidColor,
    InvalidEmpty,
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

impl ChessPiece {
    fn get_color(&self) -> PieceColor {
        if *self as u8 & COLOR_BIT == White as u8 {
            White
        } else {
            Black
        }
    }
}

impl PieceColor {
    fn switch(&mut self) {
        match self {
            White => *self = Black,
            Black => *self = White,
        }
    }
}

impl ChessBoard {
    fn san_pos_to_idx(&self, m: SAN) -> Result<usize, ChessErr> {
        if m[0] < 'a' as u8 || m[0] > 'h' as u8 {
            // hardcoded board size
            return Err(InvalidSAN);
        }
        if m[1] < '1' as u8 || m[1] > '8' as u8 {
            // hardcoded board size
            return Err(InvalidSAN);
        }

        let x = (m[0] - 'a' as u8) as usize;
        let y = (m[1] - '1' as u8) as usize;
        let y = BOARD_H - y - 1;

        Ok(x + y * BOARD_W)
    }

    fn check_continous_tiles(
        &self,
        moves: &mut Moves,
        idx: usize,
        end: usize,
        f: impl Fn(usize, usize) -> usize,
        move_type: ChessPiece,
    ) {
        for i in 1..end {
            let dest = f(idx, i);
            let tile = self.tiles[dest];
            if tile == Empty {
                moves[dest] = true;
            } else if tile.get_color() != move_type.get_color() {
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
        let mut moves: Moves = [false; BOARD_W * BOARD_H];

        match move_type {
            WhiteRook | BlackRook => {
                self.check_continous_tiles(
                    // right
                    &mut moves,
                    idx,
                    BOARD_W - idx % BOARD_W,
                    |idx, i| idx + i,
                    move_type,
                );
                self.check_continous_tiles(
                    // left
                    &mut moves,
                    idx,
                    idx % BOARD_W + 1,
                    |idx, i| idx - i,
                    move_type,
                );
                self.check_continous_tiles(
                    // down
                    &mut moves,
                    idx,
                    BOARD_H - idx / BOARD_W,
                    |idx, i| idx + i * BOARD_W,
                    move_type,
                );
                self.check_continous_tiles(
                    // up
                    &mut moves,
                    idx,
                    idx / BOARD_W + 1,
                    |idx, i| idx - i * BOARD_W,
                    move_type,
                );
            }
            WhiteBishop | BlackBishop => {
                self.check_continous_tiles(
                    &mut moves,
                    idx,
                    min(BOARD_W - idx % BOARD_W, BOARD_H - idx / BOARD_W),
                    |idx, i| idx + i * (BOARD_W + 1),
                    move_type,
                );
                self.check_continous_tiles(
                    &mut moves,
                    idx,
                    min(idx % BOARD_W + 1, BOARD_H - idx / BOARD_W),
                    |idx, i| idx + i * (BOARD_W - 1),
                    move_type,
                );
                self.check_continous_tiles(
                    &mut moves,
                    idx,
                    min(idx % BOARD_W + 1, idx / BOARD_W + 1),
                    |idx, i| idx - i * (BOARD_W + 1),
                    move_type,
                );
                self.check_continous_tiles(
                    &mut moves,
                    idx,
                    min(BOARD_W - idx % BOARD_W, idx / BOARD_W + 1),
                    |idx, i| idx - i * (BOARD_W - 1),
                    move_type,
                );
            }
            WhitePawn => {
                if idx / BOARD_W == 0 {
                    return moves;
                }
                if idx / BOARD_W == BOARD_H - 2 {
                    // double move
                    for i in 0..2 {
                        if let Empty = self.tiles[idx - BOARD_W * (i + 1)] {
                            moves[idx - BOARD_W * (i + 1)] = true;
                        } else {
                            break;
                        }
                    }
                } else if idx >= BOARD_W {
                    // single move
                    if let Empty = self.tiles[idx - BOARD_W] {
                        moves[idx - BOARD_W] = true;
                    }
                }

                // capture left
                if idx % BOARD_W > 0 {
                    let tile = self.tiles[idx - BOARD_W - 1];
                    if tile != Empty && tile.get_color() != move_type.get_color() {
                        moves[idx - BOARD_W - 1] = true;
                    }
                }

                // capture right
                if idx % BOARD_W < BOARD_W - 1 {
                    let tile = self.tiles[idx - BOARD_W + 1];
                    if tile != Empty && tile.get_color() != move_type.get_color() {
                        moves[idx - BOARD_W + 1] = true;
                    }
                }
            }
            BlackPawn => {
                if idx / BOARD_W == BOARD_H - 1 {
                    return moves;
                }
                if idx / BOARD_W == 1 {
                    // double move
                    for i in 0..2 {
                        if let Empty = self.tiles[idx + BOARD_W * (i + 1)] {
                            moves[idx + BOARD_W * (i + 1)] = true;
                        } else {
                            break;
                        }
                    }
                } else if idx < BOARD_W * (BOARD_H - 1) {
                    // single move
                    if let Empty = self.tiles[idx + BOARD_W] {
                        moves[idx + BOARD_W] = true;
                    }
                }

                // capture left
                if idx % BOARD_W > 0 {
                    let tile = self.tiles[idx + BOARD_W - 1];
                    if tile != Empty && tile.get_color() != move_type.get_color() {
                        moves[idx + BOARD_W - 1] = true;
                    }
                }

                // capture right
                if idx % BOARD_W < BOARD_W - 1 {
                    let tile = self.tiles[idx + BOARD_W + 1];
                    if tile != Empty && tile.get_color() != move_type.get_color() {
                        moves[idx + BOARD_W + 1] = true;
                    }
                }
            }
            WhiteKing | BlackKing => {
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

    pub fn get_moves(&self, m: SAN, color: PieceColor) -> Result<Moves, ChessErr> {
        // TODO special moves: en-passant, castling, promotion
        // supported only up to board sizes 9x9
        let target = self.san_pos_to_idx(m)?;
        if let Empty = self.tiles[target] {
            return Err(InvalidEmpty);
        }
        println!("{}, {color:?}", self.tiles[target] as u8);
        if self.tiles[target].get_color() != color {
            return Err(InvalidColor);
        }

        Ok(self.get_squares(target, self.tiles[target]))
    }

    pub fn make_move_coord(&mut self, start: SAN, end: SAN) -> Result<(), ChessErr> {
        // TODO checks
        let moves = self.get_moves(start, self.active_color)?;
        let start = self.san_pos_to_idx(start)?;
        let end = self.san_pos_to_idx(end)?;

        if !moves[end] {
            return Err(InvalidMove);
        }

        self.tiles[end] = self.tiles[start];
        self.tiles[start] = Empty;
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
            let piece = self.tiles[i];

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
