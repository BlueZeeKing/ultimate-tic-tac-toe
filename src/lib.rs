use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::Display,
    sync::atomic::{self, AtomicU64},
};

use dashmap::DashMap;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MiniMaxResult {
    pub global: usize,
    pub local: usize,
    pub eval: f64,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Board {
    pub locals_x: u128,
    pub locals_o: u128,
    pub global_x: u16,
    pub global_o: u16,
    pub global_full: u16,
    pub to_play: Player,
    pub global_idx: Option<usize>,
}

impl Default for Board {
    fn default() -> Self {
        Board {
            to_play: Player::X,
            global_idx: None,
            locals_x: 0,
            locals_o: 0,
            global_x: 0,
            global_o: 0,
            global_full: 0,
        }
    }
}

#[derive(Clone, PartialEq, Copy, Debug, Eq, Hash, Serialize, Deserialize)]
pub enum LocalBoardState {
    Win(Player),
    Tie,
}

#[derive(Clone, Copy, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub enum Player {
    X,
    O,
}

impl Player {
    pub fn invert(self) -> Self {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }

    pub fn to_char(this: Option<Self>) -> char {
        match this {
            Some(Player::X) => 'X',
            Some(Player::O) => 'O',
            None => ' ',
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Copy)]
pub struct IndividualBoard(pub u16, pub u16);

const ACROSS_TOP: u16 = 0b111000000;
const ACROSS_MIDDLE: u16 = ACROSS_TOP >> 3;
const ACROSS_BOTTOM: u16 = ACROSS_MIDDLE >> 3;

const DOWN_LEFT: u16 = 0b100100100;
const DOWN_MIDDLE: u16 = DOWN_LEFT >> 1;
const DOWN_RIGHT: u16 = DOWN_MIDDLE >> 1;

const DIAG1: u16 = 0b100010001;
const DIAG2: u16 = 0b001010100;

fn has_won_raw(board: u16) -> bool {
    board & ACROSS_TOP == ACROSS_TOP
        || board & ACROSS_MIDDLE == ACROSS_MIDDLE
        || board & ACROSS_BOTTOM == ACROSS_BOTTOM
        || board & DOWN_LEFT == DOWN_LEFT
        || board & DOWN_MIDDLE == DOWN_MIDDLE
        || board & DOWN_RIGHT == DOWN_RIGHT
        || board & DIAG1 == DIAG1
        || board & DIAG2 == DIAG2
}

impl IndividualBoard {
    pub fn key(self) -> u32 {
        ((self.0 as u32) << 16) | self.1 as u32
    }

    pub fn squares(self) -> impl Iterator<Item = Option<Player>> {
        (0..9).map(move |idx| get_player_at_idx(self, idx))
    }

    pub fn is_tie(&self) -> bool {
        self.0 | self.1 == 0b111111111
    }

    pub fn has_won(&self) -> Option<Player> {
        if has_won_raw(self.0) {
            Some(Player::X)
        } else if has_won_raw(self.1) {
            Some(Player::O)
        } else {
            None
        }
    }

    pub fn get_state(&self) -> Option<LocalBoardState> {
        if let Some(winner) = self.has_won() {
            Some(LocalBoardState::Win(winner))
        } else if self.is_tie() {
            Some(LocalBoardState::Tie)
        } else {
            None
        }
    }
}

impl Board {
    pub fn key(&self) -> u64 {
        ((self.global_full as u64) << 32) | ((self.global_o as u64) << 16) | self.global_x as u64
    }

    pub fn to_play(&self) -> char {
        match self.to_play {
            Player::X => 'X',
            Player::O => 'O',
        }
    }

    pub fn is_tie(&self) -> bool {
        self.global_o | self.global_x | self.global_full == 0b111111111
    }

    pub fn has_won(&self) -> Option<Player> {
        if has_won_raw(self.global_x) {
            Some(Player::X)
        } else if has_won_raw(self.global_o) {
            Some(Player::O)
        } else {
            None
        }
    }

    pub fn play(&self, global: usize, local: usize) -> Option<Self> {
        if matches!(self.global_idx, Some(allowed_global) if allowed_global != global) {
            return None;
        }

        if ((self.global_full | self.global_o | self.global_x) >> global) & 1 == 1 {
            return None;
        }

        let mut new_self = self.to_owned();

        let idx = global * 9 + local;

        if ((new_self.locals_x | new_self.locals_o) >> idx) & 1 == 1 {
            return None;
        }

        if new_self.to_play == Player::X {
            new_self.locals_x |= 1 << idx;
        } else {
            new_self.locals_o |= 1 << idx;
        }

        match new_self.get_local(global).get_state() {
            Some(LocalBoardState::Tie) => new_self.global_full |= 1 << global,
            Some(LocalBoardState::Win(Player::O)) => new_self.global_o |= 1 << global,
            Some(LocalBoardState::Win(Player::X)) => new_self.global_x |= 1 << global,
            None => {}
        }

        new_self.to_play = new_self.to_play.invert();
        new_self.global_idx =
            if ((new_self.global_x | new_self.global_o | new_self.global_full) >> local) & 1 == 1 {
                None
            } else {
                Some(local)
            };

        Some(new_self)
    }

    pub fn get_local(&self, idx: usize) -> IndividualBoard {
        IndividualBoard(
            ((self.locals_x >> (idx * 9)) & 0b111111111) as u16,
            ((self.locals_o >> (idx * 9)) & 0b111111111) as u16,
        )
    }

    pub fn evalutate(&self) -> f64 {
        let sum = (0..9)
            .map(|board_idx| {
                let sum = evalute(self.get_local(board_idx), Player::X);
                let sum2 = evalute(self.get_local(board_idx), Player::O);

                sum + sum2
            })
            .sum::<f64>()
            / 18.0;

        let sum1 = evaluate_whole(self.to_owned(), Player::X);
        let sum2 = evaluate_whole(self.to_owned(), Player::O);

        sum + (sum1 + sum2) / 2.0
    }
}

fn get_player_at_idx(board: IndividualBoard, idx: u16) -> Option<Player> {
    if (board.0 >> idx) & 1 == 1 {
        Some(Player::X)
    } else if (board.1 >> idx) & 1 == 1 {
        Some(Player::O)
    } else {
        None
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn render_group(items: &[IndividualBoard]) -> String {
            [0, 3, 6]
                .into_iter()
                .map(|idx| {
                    items
                        .iter()
                        .copied()
                        .map(|board| {
                            format!(
                                "{}|{}|{}",
                                Player::to_char(get_player_at_idx(board, idx)),
                                Player::to_char(get_player_at_idx(board, idx + 1)),
                                Player::to_char(get_player_at_idx(board, idx + 2))
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(" | ")
                })
                .collect::<Vec<_>>()
                .join("\n-+-+- | -+-+- | -+-+-\n")
        }

        let output = [0, 3, 6]
            .into_iter()
            .map(|idx| {
                render_group(&[
                    self.get_local(idx),
                    self.get_local(idx + 1),
                    self.get_local(idx + 2),
                ])
            })
            .collect::<Vec<_>>()
            .join("\n      |       |      \n------+-------+------\n      |       |      \n");

        writeln!(f, "{}", output)
    }
}

pub fn evalute(board: IndividualBoard, player: Player) -> f64 {
    if board.is_tie() {
        return 0.0;
    } else if let Some(winner) = board.has_won() {
        match winner {
            Player::X => return 1.0,
            Player::O => return -1.0,
        }
    }

    let res = (0..9)
        .filter(|idx| ((board.0 | board.1) >> idx) & 1 == 0)
        .map(|idx| {
            let mut new_board = board.clone();
            if player == Player::X {
                new_board.0 |= 1 << idx;
            } else {
                new_board.1 |= 1 << idx;
            }
            evalute(new_board, player.invert())
        })
        .sum::<f64>()
        / 9.0;

    res
}

pub fn evaluate_whole(board: Board, player: Player) -> f64 {
    if board.is_tie() {
        return 0.0;
    } else if let Some(winner) = board.has_won() {
        match winner {
            Player::X => return 1.0,
            Player::O => return -1.0,
        }
    }

    let res = (0..9)
        .filter(|idx| ((board.global_full | board.global_o | board.global_x) >> idx) & 1 == 0)
        .map(|idx| {
            let mut new_board = board.clone();
            if player == Player::X {
                new_board.global_x |= 1 << idx;
            } else {
                new_board.global_o |= 1 << idx;
            }
            evaluate_whole(new_board, player.invert())
        })
        .sum::<f64>()
        / 9.0;

    res
}
