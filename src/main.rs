use std::fmt;

use crate::{ChessErr::*, ChessPiece::*, PieceColor::*};

const BOARD_W: usize = 8;
const BOARD_H: usize = 8;

#[derive(Copy, Clone, Debug)]
enum ChessPiece {
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
enum PieceColor {
    White,
    Black,
}

enum CastlingRights {
    WhiteKing = 0b0001,
    WhiteQueen = 0b0010,
    BlackKing = 0b0100,
    BlackQueen = 0b1000,
}

#[derive(Debug)]
enum ChessErr {
    InvalidBoard,
    SpaceExpected,
    EOFError,
    ActiveColorError,
    CastlingRightError,
    HalfMovesError,
    MoveCountError,
    EnPassantError,
}

struct ChessBoard {
    tiles: [ChessPiece; BOARD_W * BOARD_H],
    active_color: PieceColor,
    castling_rights: u8,
    half_moves: u16,
    move_count: u16,
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

            _ => return Err(EOFError),
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
            _ => return Err(EOFError),
        })
    }
}

impl ChessBoard {
    fn from_fen(fen: &str) -> Result<ChessBoard, ChessErr> {
        let mut it = fen.chars();
        let mut rank = 0;
        let mut file = 0;
        let mut tiles: [ChessPiece; BOARD_W * BOARD_H] = [Empty; BOARD_W * BOARD_H];
        let mut active_color: PieceColor = PieceColor::White;
        let mut castling_rights: u8 = 0;
        let mut en_passant: isize = -1;
        let mut half_moves: u16 = 0;
        let mut move_count: u16 = 0;

        for c in it.by_ref() {
            if rank == 7 && file == 8 {
                break;
            }
            match c {
                '/' => {
                    rank += 1;
                    file = 0;
                    if rank >= BOARD_H {
                        return Err(InvalidBoard);
                    }
                }
                ' ' => {
                    if rank == 7 && file == 8 {
                        break;
                    } else {
                        return Err(InvalidBoard);
                    }
                }
                p => {
                    let idx = rank * BOARD_W + file;
                    if p.is_ascii_digit() {
                        let num = (p as u8 - '0' as u8) as usize;
                        for j in 0..num {
                            if idx + j >= BOARD_W * BOARD_H {
                                return Err(InvalidBoard);
                            }
                            tiles[idx + j] = Empty;
                        }
                        file += num;
                    } else {
                        tiles[idx] = ChessPiece::from_char(p)?;
                        file += 1;
                    }
                }
            }
        }

        // active color
        active_color = match it.next().ok_or(EOFError)? {
            'w' => White,
            'b' => Black,
            _ => return Err(ActiveColorError),
        };

        let mut c = it.next().ok_or(EOFError)?;
        if c != ' ' {
            return Err(SpaceExpected);
        }
        c = it.next().ok_or(EOFError)?;

        // castling rigths
        if c != '-' {
            while c != ' ' {
                castling_rights |= match c {
                    'K' => CastlingRights::WhiteKing,
                    'Q' => CastlingRights::WhiteQueen,
                    'k' => CastlingRights::BlackKing,
                    'q' => CastlingRights::BlackQueen,
                    _ => return Err(CastlingRightError),
                } as u8;
                c = it.next().ok_or(EOFError)?;
            }
        } else {
            c = it.next().ok_or(EOFError)?;
            if c != ' ' {
                return Err(SpaceExpected);
            }
        }
        c = it.next().ok_or(EOFError)?;
        // en passant
        if c != '-' {
            // supported only up to board sizes 24x9
            if c < 'a' || c > 'z' {
                return Err(EnPassantError);
            }
            let rank = c as u8 - 'a' as u8;
            if rank as usize >= BOARD_H {
                return Err(EnPassantError);
            }
            c = it.next().ok_or(EOFError)?;
            if !c.is_ascii_digit() {
                return Err(EnPassantError);
            }
            let file = c as u8 - '1' as u8;
            if file as usize >= BOARD_W {
                return Err(EnPassantError);
            }
            en_passant = (file * BOARD_W as u8 + rank) as isize;
        }

        c = it.next().ok_or(EOFError)?;
        if c != ' ' {
            return Err(SpaceExpected);
        }

        //half moves
        c = it.next().ok_or(EOFError)?;
        if !c.is_ascii_digit() {
            return Err(HalfMovesError);
        }
        loop {
            half_moves *= 10;
            half_moves += (c as u8 - '0' as u8) as u16;
            c = it.next().ok_or(EOFError)?;
            if c == ' ' {
                break;
            }
            if !c.is_ascii_digit() {
                return Err(HalfMovesError);
            }
        }

        // move count
        c = it.next().ok_or(EOFError)?;
        if !c.is_ascii_digit() {
            return Err(MoveCountError);
        }
        loop {
            move_count *= 10;
            move_count += (c as u8 - '0' as u8) as u16;
            let Some(c) = it.next() else {
                if move_count == 0 {
                    return Err(MoveCountError);
                }
                return Ok(ChessBoard {
                    tiles,
                    castling_rights,
                    move_count,
                    half_moves,
                    active_color,
                    en_passant,
                });
            };

            if c == ' ' {
                break;
            }
            if !c.is_ascii_digit() {
                return Err(MoveCountError);
            }
        }

        if move_count == 0 {
            return Err(MoveCountError);
        }

        Ok(ChessBoard {
            tiles,
            castling_rights,
            move_count,
            half_moves,
            active_color,
            en_passant,
        })
    }

    fn get_by_idx(&self, idx: usize) -> ChessPiece {
        self.tiles[idx]
    }
}

fn main() -> Result<(), ChessErr> {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    let board = ChessBoard::from_fen(fen)?;
    println!("{board}");
    Ok(())
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

        // write!(f, "{}\n", self.active_color);

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
