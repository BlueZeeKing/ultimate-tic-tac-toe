use std::{cmp::Ordering, sync::atomic};

use dashmap::DashMap;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use ultimate_tic_tac_toe::{Board, Player};

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

pub struct AtomicF64 {
    storage: atomic::AtomicU64,
}

impl AtomicF64 {
    pub fn new(value: f64) -> Self {
        let as_u64 = value.to_bits();
        Self {
            storage: atomic::AtomicU64::new(as_u64),
        }
    }
    pub fn store(&self, value: f64, ordering: atomic::Ordering) {
        let as_u64 = value.to_bits();
        self.storage.store(as_u64, ordering)
    }
    pub fn load(&self, ordering: atomic::Ordering) -> f64 {
        let as_u64 = self.storage.load(ordering);
        f64::from_bits(as_u64)
    }
}

pub fn evaluate(whole_board: &Board) -> f64 {
    let sum = (0..9)
        .map(|board_idx| {
            let key = whole_board.get_local(board_idx).key();

            BOARD_EVALS.get(&key).unwrap()
        })
        .sum::<f64>()
        / 9.0;

    let key = whole_board.key();

    let sum2 = WHOLE_BOARD_EVALS.get(&key).unwrap();

    sum + sum2
}

pub fn minimax_single(
    whole_board: &Board,
    depth: u64,
    mut alpha: f64,
    mut beta: f64,
) -> ((usize, usize), f64, u64) {
    if whole_board.is_tie() {
        return ((0, 0), 0.0, depth);
    } else if let Some(player) = whole_board.has_won() {
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

    let range = if let Some(idx) = whole_board.global_idx {
        idx * 9..idx * 9 + 9
    } else {
        0..81
    };

    let results = range
        .filter(|idx| ((whole_board.locals_x | whole_board.locals_o) >> idx) & 1 == 0)
        .map(|idx| (idx / 9, idx % 9))
        .filter_map(|(global, local)| Some(((global, local), whole_board.play(global, local)?)))
        .filter_map(|(pos, board)| {
            if beta <= alpha {
                return None;
            }

            Some(if depth == 0 {
                (pos, evaluate(whole_board), depth)
            } else {
                let (_, value, eval_depth) = minimax_single(&board, depth - 1, alpha, beta);

                if whole_board.to_play == Player::X {
                    alpha = alpha.max(value);
                } else {
                    beta = beta.min(value);
                }

                (pos, value, eval_depth)
            })
        });

    let res = if whole_board.to_play == Player::X {
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

    res
}

pub fn minimax(
    whole_board: &Board,
    depth: u64,
    threaded_depth: u64,
    alpha: f64,
    beta: f64,
) -> ((usize, usize), f64, u64) {
    if whole_board.is_tie() {
        return ((0, 0), 0.0, depth);
    } else if let Some(player) = whole_board.has_won() {
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

    let range = if let Some(idx) = whole_board.global_idx {
        idx * 9..idx * 9 + 9
    } else {
        0..81
    };

    let alpha = AtomicF64::new(alpha);
    let beta = AtomicF64::new(beta);

    let results = range
        .into_par_iter()
        .filter(|idx| ((whole_board.locals_x | whole_board.locals_o) >> idx) & 1 == 0)
        .map(|idx| (idx / 9, idx % 9))
        .filter_map(|(global, local)| Some(((global, local), whole_board.play(global, local)?)))
        .filter_map(|(pos, board)| {
            if beta.load(atomic::Ordering::Relaxed) <= alpha.load(atomic::Ordering::Relaxed) {
                return None;
            }

            Some(if depth == 0 {
                (pos, evaluate(whole_board), depth)
            } else {
                let alpha_val = alpha.load(atomic::Ordering::Relaxed);
                let beta_val = beta.load(atomic::Ordering::Relaxed);
                let (_, value, eval_depth) = if threaded_depth == 0 {
                    minimax_single(&board, depth - 1, alpha_val, beta_val)
                } else {
                    minimax(&board, depth - 1, threaded_depth - 1, alpha_val, beta_val)
                };

                if whole_board.to_play == Player::X {
                    if value > alpha_val {
                        alpha.store(value, atomic::Ordering::Relaxed);
                    }
                } else {
                    if value < beta_val {
                        beta.store(value, atomic::Ordering::Relaxed);
                    }
                }

                (pos, value, eval_depth)
            })
        });

    let res = if whole_board.to_play == Player::X {
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

    res
}
