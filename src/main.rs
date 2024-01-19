use dashmap::DashMap;
use leptos::{logging::log, *};
use ultimate_tic_tac_toe::{Board, IndividualBoard, Player};
use web_sys::window;

fn main() {
    wasm_bindgen_rayon::init_thread_pool(
        window().unwrap().navigator().hardware_concurrency() as usize
    );

    let (board, set_board) = create_signal(Board::default());

    let play = move |global: usize, local: usize| {
        if let Some(new_board) = board.get().play(global, local) {
            set_board.set(new_board.clone());

            let cache1 = DashMap::new();
            let cache2 = DashMap::new();
            let cache3 = DashMap::new();

            let ((global, local), _eval) = new_board.minimax(5, &cache1, &cache2, &cache3);

            set_board.set(new_board.play(global, local).expect("Invalid move"));
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
            <div class="grid grid-cols-3 absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[27rem] h-[27rem]">
                {render_board}
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
            <div class="relative p-6 border-black border-b border-r [&:nth-child(3n)]:border-r-0 [&:nth-child(n+7)]:border-b-0 w-[9rem]">
                <p class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2">{char}</p>
            </div>
        }
    } else {
        view! {
            <div class="grid grid-cols-3 p-6 border-black border-b border-r [&:nth-child(3n)]:border-r-0 [&:nth-child(n+7)]:border-b-0 w-[9rem]">
                {squares}
            </div>
        }
    }
}
