use std::{cmp::Ordering, fmt::Display};

use dashmap::DashMap;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Board {
    pub global: [Option<LocalBoardState>; 9],
    pub locals: [IndividualBoard; 9],
    pub to_play: Player,
    pub global_idx: Option<usize>,
}

impl Default for Board {
    fn default() -> Self {
        Board {
            global: Default::default(),
            locals: Default::default(),
            to_play: Player::X,
            global_idx: None,
        }
    }
}

#[derive(Clone, PartialEq, Copy, Debug, Eq, Hash)]
pub enum LocalBoardState {
    Win(Player),
    Tie,
}

#[derive(Clone, Copy, PartialEq, Debug, Eq, Hash)]
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct IndividualBoard(pub [Option<Player>; 9]);

impl Default for IndividualBoard {
    fn default() -> Self {
        Self([None; 9])
    }
}

impl IndividualBoard {
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (usize, Option<Player>)> + 'a {
        self.0.iter().copied().enumerate()
    }

    pub fn is_tie(&self) -> bool {
        !self.iter().any(|val| val.1.is_none()) && self.has_won().is_none()
    }

    pub fn has_won(&self) -> Option<Player> {
        if self.0[0].is_some() && self.0[0] == self.0[1] && self.0[0] == self.0[2] {
            self.0[0]
        } else if self.0[3].is_some() && self.0[3] == self.0[4] && self.0[3] == self.0[5] {
            self.0[3]
        } else if self.0[6].is_some() && self.0[6] == self.0[7] && self.0[6] == self.0[8] {
            self.0[6]
        } else if self.0[0].is_some() && self.0[0] == self.0[3] && self.0[0] == self.0[6] {
            self.0[0]
        } else if self.0[1].is_some() && self.0[1] == self.0[4] && self.0[1] == self.0[7] {
            self.0[1]
        } else if self.0[2].is_some() && self.0[2] == self.0[5] && self.0[2] == self.0[8] {
            self.0[2]
        } else if self.0[0].is_some() && self.0[0] == self.0[4] && self.0[0] == self.0[8] {
            self.0[0]
        } else if self.0[2].is_some() && self.0[2] == self.0[4] && self.0[2] == self.0[6] {
            self.0[2]
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

fn has_winner(square: Option<LocalBoardState>) -> bool {
    matches!(square, Some(LocalBoardState::Win(_)))
}

fn player(square: Option<LocalBoardState>) -> Option<Player> {
    match square {
        Some(LocalBoardState::Win(player)) => Some(player),
        None => None,
        Some(LocalBoardState::Tie) => unreachable!(),
    }
}

impl Board {
    pub fn to_play(&self) -> char {
        match self.to_play {
            Player::X => 'X',
            Player::O => 'O',
        }
    }

    pub fn is_tie(&self) -> bool {
        !self.global.iter().any(|val| val.is_none()) && self.has_won().is_none()
    }

    pub fn has_won(&self) -> Option<Player> {
        if has_winner(self.global[0])
            && self.global[0] == self.global[1]
            && self.global[0] == self.global[2]
        {
            player(self.global[0])
        } else if has_winner(self.global[3])
            && self.global[3] == self.global[4]
            && self.global[3] == self.global[5]
        {
            player(self.global[3])
        } else if has_winner(self.global[6])
            && self.global[6] == self.global[7]
            && self.global[6] == self.global[8]
        {
            player(self.global[6])
        } else if has_winner(self.global[0])
            && self.global[0] == self.global[3]
            && self.global[0] == self.global[6]
        {
            player(self.global[0])
        } else if has_winner(self.global[1])
            && self.global[1] == self.global[4]
            && self.global[1] == self.global[7]
        {
            player(self.global[1])
        } else if has_winner(self.global[2])
            && self.global[2] == self.global[5]
            && self.global[2] == self.global[8]
        {
            player(self.global[2])
        } else if has_winner(self.global[0])
            && self.global[0] == self.global[4]
            && self.global[0] == self.global[8]
        {
            player(self.global[0])
        } else if has_winner(self.global[2])
            && self.global[2] == self.global[4]
            && self.global[2] == self.global[6]
        {
            player(self.global[2])
        } else {
            None
        }
    }

    pub fn play(&self, global: usize, local: usize) -> Option<Self> {
        if matches!(self.global_idx, Some(allowed_global) if allowed_global != global) {
            return None;
        }

        if self.global[global].is_some() {
            return None;
        }

        let mut new_self = self.to_owned();

        if new_self.locals[global].0[local].is_some() {
            return None;
        }

        new_self.locals[global].0[local] = Some(new_self.to_play);
        new_self.global[global] = new_self.locals[global].get_state();
        new_self.to_play = new_self.to_play.invert();
        new_self.global_idx = if new_self.global[local].is_some() {
            None
        } else {
            Some(local)
        };

        Some(new_self)
    }

    pub fn evalutate(
        &self,
        cache: &DashMap<(IndividualBoard, Player), f64>,
        eval_cache2: &DashMap<([Option<LocalBoardState>; 9], Player), f64>,
    ) -> f64 {
        let sum = self
            .locals
            .iter()
            .map(|board| {
                let sum = evalute(board.to_owned(), Player::X, cache);
                let sum2 = evalute(board.to_owned(), Player::O, cache);

                sum + sum2
            })
            .sum::<f64>()
            / 18.0;

        let sum1 = evaluate_whole(self.to_owned(), Player::X, eval_cache2);
        let sum2 = evaluate_whole(self.to_owned(), Player::O, eval_cache2);

        sum + (sum1 + sum2) / 2.0
    }

    pub fn minimax(
        &self,
        depth: u64,
        cache: &DashMap<Board, (u64, ((usize, usize), f64, u64))>,
        eval_cache: &DashMap<(IndividualBoard, Player), f64>,
        eval_cache2: &DashMap<([Option<LocalBoardState>; 9], Player), f64>,
    ) -> ((usize, usize), f64, u64) {
        if let Some((new_depth, res)) = cache.get(self).as_deref() {
            if *new_depth >= depth {
                return *res;
            }
        }

        if self.is_tie() {
            return ((0, 0), 0.0, depth);
        } else if let Some(player) = self.has_won() {
            if depth == 4 {
                return (
                    (0, 0),
                    match player {
                        Player::X => 10.0,
                        Player::O => -10.0,
                    },
                    depth,
                );
            }
            return (
                (0, 0),
                match player {
                    Player::X => 10.0,
                    Player::O => -10.0,
                },
                depth,
            );
        }

        let results = self
            .locals
            .par_iter()
            .enumerate()
            .filter(|(idx, _)| self.global_idx.is_none() || self.global_idx.unwrap() == *idx)
            // .filter(|(idx, _)| self.global[*idx].is_none())
            .map(|(global, board)| {
                board
                    .0
                    .par_iter()
                    .enumerate()
                    .filter(|(_, position)| position.is_none())
                    .map(move |(local, _)| (global, local))
            })
            .flatten()
            .filter_map(|(global, local)| Some(((global, local), self.play(global, local)?)))
            .map(|(pos, board)| {
                if depth == 0 {
                    (pos, board.evalutate(eval_cache, eval_cache2), depth)
                } else {
                    let (_, value, eval_depth) =
                        board.minimax(depth - 1, cache, eval_cache, eval_cache2);
                    (pos, value, eval_depth)
                }
            });

        let res = if self.to_play == Player::X {
            results
                .max_by(|(_, eval_a, depth_a), (_, eval_b, depth_b)| {
                    match eval_a.partial_cmp(eval_b).unwrap() {
                        Ordering::Equal => depth_a.cmp(depth_b),
                        n => n,
                    }
                })
                .unwrap()
        } else {
            results
                .min_by(|(_, eval_a, depth_a), (_, eval_b, depth_b)| {
                    match eval_a.partial_cmp(eval_b).unwrap() {
                        Ordering::Equal => depth_b.cmp(depth_a),
                        n => n,
                    }
                })
                .unwrap()
        };

        cache.insert(self.to_owned(), (depth, res));

        res
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
                        .map(|board| {
                            format!(
                                "{}|{}|{}",
                                Player::to_char(board.0[idx]),
                                Player::to_char(board.0[idx + 1]),
                                Player::to_char(board.0[idx + 2])
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
            .map(|idx| render_group(&self.locals[idx..idx + 3]))
            .collect::<Vec<_>>()
            .join("\n      |       |      \n------+-------+------\n      |       |      \n");

        writeln!(f, "{}", output)
    }
}

pub fn evalute(
    board: IndividualBoard,
    player: Player,
    cache: &DashMap<(IndividualBoard, Player), f64>,
) -> f64 {
    if let Some(res) = cache.get(&(board.clone(), player)) {
        return *res;
    }
    if board.is_tie() {
        return 0.0;
    } else if let Some(winner) = board.has_won() {
        match winner {
            Player::X => return 1.0,
            Player::O => return -1.0,
        }
    }

    let res = board
        .0
        .iter()
        .enumerate()
        .filter(|(_, square)| square.is_none())
        .map(|(idx, _)| {
            let mut new_board = board.clone();
            new_board.0[idx] = Some(player);
            evalute(new_board, player.invert(), cache)
        })
        .sum::<f64>()
        / 9.0;

    cache.insert((board, player), res);

    res
}

fn evaluate_whole(
    board: Board,
    player: Player,
    eval_cache2: &DashMap<([Option<LocalBoardState>; 9], Player), f64>,
) -> f64 {
    if let Some(res) = eval_cache2.get(&(board.global, player)) {
        return *res;
    }

    if board.is_tie() {
        return 0.0;
    } else if let Some(winner) = board.has_won() {
        match winner {
            Player::X => return 1.0,
            Player::O => return -1.0,
        }
    }

    let res = board
        .global
        .iter()
        .enumerate()
        .filter(|(_, square)| square.is_none())
        .map(|(idx, _)| {
            let mut new_board = board.clone();
            new_board.global[idx] = Some(LocalBoardState::Win(player));
            evaluate_whole(new_board, player.invert(), eval_cache2)
        })
        .sum::<f64>()
        / 9.0;

    eval_cache2.insert((board.global, player), res);

    res
}
