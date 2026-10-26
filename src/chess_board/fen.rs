use crate::chess_board::*;

use FENErr::*;

#[derive(Debug)]
pub enum FENErr {
    InvalidBoard,
    SpaceExpected,
    EOFError,
    ActiveColorError,
    CastlingRightError,
    HalfMovesError,
    MoveCountError,
    EnPassantError,
}

impl ChessPiece {
    fn from_char(c: char) -> Result<ChessPiece, FENErr> {
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
    fn from_char(c: char) -> Result<CastlingRights, FENErr> {
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
    fn expect_space(it: &mut std::str::Chars) -> Result<(), FENErr> {
        if it.next().ok_or(EOFError)? == ' ' {
            return Ok(());
        }
        Err(SpaceExpected)
    }

    fn gen_active_color(it: &mut std::str::Chars) -> Result<PieceColor, FENErr> {
        let active_color = match it.next().ok_or(EOFError)? {
            'w' => White,
            'b' => Black,
            _ => return Err(ActiveColorError),
        };

        ChessBoard::expect_space(it)?;
        Ok(active_color)
    }

    fn gen_castling_rights(it: &mut std::str::Chars) -> Result<u8, FENErr> {
        let mut c = it.next().ok_or(EOFError)?;
        let mut castling_rights: u8 = 0;
        if c != '-' {
            while c != ' ' {
                castling_rights |= CastlingRights::from_char(c)? as u8;
                c = it.next().ok_or(EOFError)?;
            }
        } else {
            ChessBoard::expect_space(it)?;
        }
        Ok(castling_rights)
    }

    fn gen_en_passant(it: &mut std::str::Chars) -> Result<isize, FENErr> {
        let mut c = it.next().ok_or(EOFError)?;
        if c == '-' {
            ChessBoard::expect_space(it)?;
            return Ok(-1);
        }

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
        let en_passant = (file * BOARD_W as u8 + rank) as isize;

        ChessBoard::expect_space(it)?;
        Ok(en_passant)
    }

    fn gen_half_moves(it: &mut std::str::Chars) -> Result<u16, FENErr> {
        let mut half_moves: u16 = 0;
        let mut c = it.next().ok_or(EOFError)?;
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

        Ok(half_moves)
    }

    fn gen_move_count(it: &mut std::str::Chars) -> Result<u16, FENErr> {
        let mut move_count: u16 = 0;
        let c = it.next().ok_or(EOFError)?;
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
                return Ok(move_count);
            };

            if !c.is_ascii_digit() {
                return Err(MoveCountError);
            }
        }
    }

    pub fn from_fen(fen: &str) -> Result<ChessBoard, FENErr> {
        let mut it = fen.chars();
        let mut rank = 0;
        let mut file = 0;
        let mut tiles: [ChessPiece; BOARD_W * BOARD_H] = [Empty; BOARD_W * BOARD_H];

        for c in it.by_ref() {
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
                    if file > 7 {
                        return Err(InvalidBoard);
                    }
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
                        if file > BOARD_W {
                            return Err(InvalidBoard);
                        }
                    } else {
                        tiles[idx] = ChessPiece::from_char(p)?;
                        file += 1;
                    }
                }
            }
        }

        let active_color = ChessBoard::gen_active_color(&mut it)?;
        let castling_rights = ChessBoard::gen_castling_rights(&mut it)?;
        let en_passant = ChessBoard::gen_en_passant(&mut it)?;
        let half_moves = ChessBoard::gen_half_moves(&mut it)?;
        let move_count = ChessBoard::gen_move_count(&mut it)?;

        Ok(ChessBoard {
            tiles,
            castling_rights,
            move_count,
            half_moves,
            active_color,
            en_passant,
        })
    }
}
