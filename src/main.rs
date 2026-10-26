mod chess_board;

use chess_board::*;

fn main() {
    // let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let fen = "8/8/8/8/8/8/8/8 w KQkq - 0 1";

    let mut board = ChessBoard::from_fen(fen).unwrap();
    println!("{board}");
    let _ = board.make_move("f5").unwrap();
    println!("{board}");
}
