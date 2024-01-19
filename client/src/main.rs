use leptos::*;
use reqwasm::http::Request;
use ultimate_tic_tac_toe::{Board, IndividualBoard, MiniMaxResult, Player};

fn main() {
    let (board, set_board) = create_signal(Board::default());

    let play = move |global: usize, local: usize| {
        let current_board = board.get();

        if current_board.to_play == Player::O
            || current_board.has_won().is_some()
            || current_board.is_tie()
        {
            return;
        }

        if let Some(new_board) = current_board.play(global, local) {
            set_board.set(new_board.clone());

            if new_board.has_won().is_some() || new_board.is_tie() {
                return;
            }

            spawn_local(async move {
                let response: MiniMaxResult = Request::post("/calc")
                    .body(serde_json::to_string(&new_board).unwrap())
                    .header("Content-Type", "application/json")
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();

                set_board.set(
                    board
                        .get_untracked()
                        .play(response.global, response.local)
                        .expect("Invalid move!"),
                );
            })
        }
    };

    let render_board = move || {
        let board = board.get();

        board
            .locals
            .into_iter()
            .enumerate()
            .map(|(global, local_board)| {
                view! {
                    <SingleBoard
                        board=local_board
                        active=board.global_idx.is_none() || board.global_idx.unwrap() == global
                        on_click=move |local| { play(global, local) }
                    />
                }
            })
            .collect_view()
    };

    mount_to_body(move || {
        view! {
            <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[27rem] h-[27rem]">
                <div class="grid grid-cols-3">{render_board}</div>
                <p>

                    {move || {
                        let board = board.get();
                        if let Some(winner) = board.has_won() {
                            if winner == Player::X { "You won!" } else { "You lost!" }
                        } else if board.is_tie() {
                            "Tie game!"
                        } else {
                            ""
                        }
                    }}

                </p>
            </div>
        }
    })
}

#[component]
fn SingleBoard<F: Fn(usize) + 'static + Clone>(
    board: IndividualBoard,
    active: bool,
    on_click: F,
) -> impl IntoView {
    let squares = board
        .0
        .into_iter()
        .map(|square| match square {
            None => ' ',
            Some(Player::X) => 'X',
            Some(Player::O) => 'O',
        }).enumerate()
        .map(|(idx, square)| {
            let on_click = on_click.clone();
            view! {
                <div
                    class=format!(
                        "w-8 h-8 border-b border-r [&:nth-child(3n)]:border-r-0 [&:nth-child(n+7)]:border-b-0 text-center {}",
                        if active { "border-blue-400" } else { "border-gray-400" },
                    )

                    on:click=move |_| on_click(idx)
                >
                    {square}
                </div>
            }
        })
        .collect_view();

    if let Some(state) = board.get_state() {
        let char = match state {
            ultimate_tic_tac_toe::LocalBoardState::Win(Player::X) => 'X',
            ultimate_tic_tac_toe::LocalBoardState::Win(Player::O) => 'O',
            ultimate_tic_tac_toe::LocalBoardState::Tie => 'T',
        };

        view! {
            <div class="relative p-6 border-black border-b border-r [&:nth-child(3n)]:border-r-0 [&:nth-child(n+7)]:border-b-0 w-[9rem] h-[9rem]">
                <p class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2">{char}</p>
            </div>
        }
    } else {
        view! {
            <div class="grid grid-cols-3 p-6 border-black border-b border-r [&:nth-child(3n)]:border-r-0 [&:nth-child(n+7)]:border-b-0 w-[9rem] h-[9rem]">
                {squares}
            </div>
        }
    }
}
