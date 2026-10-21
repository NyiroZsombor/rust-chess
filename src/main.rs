mod chess_board;

use chess_board::*;

fn main() -> Result<(), ChessErr> {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    let board = ChessBoard::from_fen(fen)?;
    println!("{board}");
    Ok(())
}
