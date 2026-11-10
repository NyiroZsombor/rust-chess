mod chess_board;

use chess_board::*;

fn make_move(board: &mut ChessBoard, color: PieceColor) -> Result<(), ChessErr> {
    loop {
        let mut moves = Moves::new();
        board.display(moves, color);
        let start = SAN::from_input(format!("{color:?} to move: ").as_str())?;
        moves = board.get_moves(&start, color)?;
        board.display(moves, color);
        let end = SAN::from_input("end: ")?;
        board.make_move_coord(start, end)?;

        return Ok(());
    }
}

fn main() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"; // starting FEN
    // let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";
    // let fen = "1r6/5pp1/RNR1r2p/1r1pP3/1ppkqPP1/7P/1P2B3/2K5 w - - 0 41";
    // let fen = "8/p2ppp2/4P1p1/8/8/3p1P2/P1PPP2P/8 w - - 0 1";

    let mut board = ChessBoard::from_fen(fen).unwrap();
    let mut color = PieceColor::White;
    loop {
        match make_move(&mut board, color) {
            Ok(s) => s,
            Err(err) => {
                println!("{err:?}");
                continue;
            }
        };
        color.switch();
    }
}
