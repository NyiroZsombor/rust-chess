mod chess_board;

use chess_board::*;

fn promotion_callback() -> ChessPiece {
    loop {
        if let Ok(piece) = ChessPiece::from_input("promote to: ") {
            return piece;
        };
    }
}

fn make_move(board: &mut ChessBoard) -> Result<(), ChessErr> {
    loop {
        let mut moves = Moves::new();
        board.display(moves, board.active_color);
        let start = SAN::from_input(format!("{:?} to move: ", board.active_color).as_str())?;
        moves = board.get_moves(start, board.active_color)?;
        board.display(moves, board.active_color);
        let end = SAN::from_input("end: ")?;
        board.make_move_coord(start, end, promotion_callback)?;

        return Ok(());
    }
}

fn main() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"; // starting FEN
    // let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"; // black castling
    // let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1"; // white castling
    // let fen = "1r6/5ppP/RNR1r2p/1r1pP3/1ppkqPP1/7P/1P2B3/2K5 w - - 0 41";
    // let fen = "8/p2ppp2/4P1p1/8/8/3p1P2/P1PPP2P/8 w - - 0 1";

    let mut board = ChessBoard::from_fen(fen).unwrap();
    loop {
        match make_move(&mut board) {
            Ok(s) => s,
            Err(err) => {
                println!("{err:?}");
                continue;
            }
        };
    }
}
