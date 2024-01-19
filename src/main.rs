use dashmap::DashMap;
use ultimate_tic_tac_toe::{evalute, Board, IndividualBoard, Player};

fn main() {
    let board: Board = serde_json::from_str(r#"{"global":[{"Win":"X"},null,null,null,{"Win":"X"},{"Win":"O"},{"Win":"O"},null,{"Win":"O"}],"locals":[["O",null,"X",null,null,"X",null,null,"X"],[null,"O","X","X",null,null,null,"X",null],[null,"O","O",null,null,null,"X",null,"X"],["O","X",null,null,null,"X",null,"O",null],["X",null,null,"X",null,null,"X",null,null],["O",null,null,"O",null,null,"O","X",null],[null,null,null,"O","O","O","X","X",null],[null,"O",null,null,null,"X",null,"O","X"],[null,null,"O",null,"O",null,"O",null,null]],"to_play":"O","global_idx":7}"#).unwrap();
    let ((global, local), eval, depth) =
        board.minimax(5, &DashMap::new(), &DashMap::new(), &DashMap::new());

    dbg!(global, local, depth);
    let value = board.evalutate(&DashMap::new(), &DashMap::new());

    let x: i32 = 0;
    x.zeros();

    println!("{}", board);

    dbg!(value);

    let board = IndividualBoard([
        None,
        None,
        None,
        Some(Player::X),
        None,
        Some(Player::X),
        Some(Player::O),
        None,
        None,
    ]);

    let value = evalute(board, Player::X, &DashMap::new());
    dbg!(value);
}
