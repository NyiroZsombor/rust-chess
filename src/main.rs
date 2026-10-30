mod chess_board;

use std::io::{Read, Write};

use chess_board::*;

fn get_input(prompt: &str) -> [u8; 2] {
    let mut buf = [0_u8; 2];
    std::io::stdout().write(prompt.as_bytes()).unwrap();
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_exact(&mut buf).unwrap();
    std::io::stdin().read_exact(&mut [0; 1]).unwrap();
    buf
}

fn make_move(board: &mut ChessBoard, color: PieceColor) {
    loop {
        let mut moves = [false; BOARD_W * BOARD_H];
        board.display(moves, color);
        let start = get_input(format!("{color:?} to move: ").as_str());
        moves = match board.get_moves(start, color) {
            Ok(m) => m,
            Err(err) => {
                println!("{err:?}");
                continue;
            }
        };
        board.display(moves, color);
        let end = get_input("end: ");
        match board.make_move_coord(start, end) {
            Ok(()) => (),
            Err(err) => {
                println!("{err:?}");
                continue;
            }
        }

        break;
    }
}

fn main() {
    // let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    // let fen = "1r6/5pp1/RNR1r2p/1r1pP3/1ppkqPP1/7P/1P2B3/2K5 w - - 0 41";
    let fen = "8/p2ppp2/4P1p1/8/8/3p1P2/P1PPP2P/8 w - - 0 1";

    let mut board = ChessBoard::from_fen(fen).unwrap();
    loop {
        make_move(&mut board, PieceColor::White);
        make_move(&mut board, PieceColor::Black);
    }
}
