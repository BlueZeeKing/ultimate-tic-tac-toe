use std::{env, fs::File, io::Write, path::Path};

use ultimate_tic_tac_toe::{evaluate_whole, evalute, Board, IndividualBoard, Player};

fn main() {
    let mut map = phf_codegen::Map::new();
    let mut whole_map = phf_codegen::Map::new();

    for board in all_boards(0) {
        map.entry(
            board.key(),
            &format!(
                "{:.20}",
                (evalute(board, Player::X) + evalute(board, Player::O)) / 2.0
            ),
        );
    }

    for board in all_global_boards(0) {
        whole_map.entry(
            board.key(),
            &format!(
                "{:.20}",
                (evaluate_whole(board.clone(), Player::X) + evaluate_whole(board, Player::O)) / 2.0
            ),
        );
    }

    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = File::create(&path).unwrap();

    write!(
        file,
        "static BOARD_EVALS: phf::Map<u32, f64> = {};\n",
        map.build()
    )
    .unwrap();

    write!(
        file,
        "static WHOLE_BOARD_EVALS: phf::Map<u64, f64> = {};\n",
        whole_map.build()
    )
    .unwrap();
}

fn all_boards(idx: usize) -> Vec<IndividualBoard> {
    if idx == 8 {
        vec![
            IndividualBoard(0, 0),
            IndividualBoard(1 << 8, 0),
            IndividualBoard(0, 1 << 8),
        ]
    } else {
        let all_combos = all_boards(idx + 1);
        let mut out = Vec::new();

        for other_board in all_combos {
            out.append(
                &mut [
                    IndividualBoard(0, 0),
                    IndividualBoard(1 << idx, 0),
                    IndividualBoard(0, 1 << idx),
                ]
                .into_iter()
                .map(|board| IndividualBoard(board.0 | other_board.0, board.1 | other_board.1))
                .collect::<Vec<_>>(),
            );
        }

        out
    }
}

fn all_global_boards(idx: usize) -> Vec<Board> {
    if idx == 8 {
        vec![
            Board {
                locals_x: 0,
                locals_o: 0,
                global_x: 0,
                global_o: 0,
                global_full: 0,
                to_play: Player::X,
                global_idx: None,
            },
            Board {
                locals_x: 0,
                locals_o: 0,
                global_x: 1 << 8,
                global_o: 0,
                global_full: 0,
                to_play: Player::X,
                global_idx: None,
            },
            Board {
                locals_x: 0,
                locals_o: 0,
                global_x: 0,
                global_o: 0,
                global_full: 1 << 8,
                to_play: Player::X,
                global_idx: None,
            },
            Board {
                locals_x: 0,
                locals_o: 0,
                global_x: 0,
                global_o: 1 << 8,
                global_full: 0,
                to_play: Player::X,
                global_idx: None,
            },
        ]
    } else {
        let all_combos = all_global_boards(idx + 1);
        let mut out = Vec::new();

        for other_board in all_combos {
            out.append(
                &mut [
                    Board {
                        locals_x: 0,
                        locals_o: 0,
                        global_x: 0,
                        global_o: 0,
                        global_full: 0,
                        to_play: Player::X,
                        global_idx: None,
                    },
                    Board {
                        locals_x: 0,
                        locals_o: 0,
                        global_x: 1 << idx,
                        global_o: 0,
                        global_full: 0,
                        to_play: Player::X,
                        global_idx: None,
                    },
                    Board {
                        locals_x: 0,
                        locals_o: 0,
                        global_x: 0,
                        global_o: 0,
                        global_full: 1 << idx,
                        to_play: Player::X,
                        global_idx: None,
                    },
                    Board {
                        locals_x: 0,
                        locals_o: 0,
                        global_x: 0,
                        global_o: 1 << idx,
                        global_full: 0,
                        to_play: Player::X,
                        global_idx: None,
                    },
                ]
                .into_iter()
                .map(|board| Board {
                    locals_x: 0,
                    locals_o: 0,
                    global_x: board.global_x | other_board.global_x,
                    global_o: board.global_o | other_board.global_o,
                    global_full: board.global_full | other_board.global_full,
                    to_play: Player::X,
                    global_idx: None,
                })
                .collect::<Vec<_>>(),
            );
        }

        out
    }
}
