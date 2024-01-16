use std::io::{stdin, stdout, Write};

use nom::{
    bytes::complete::tag,
    character::complete::u8 as parse_u8,
    sequence::{terminated, tuple},
    IResult,
};
use ultimate_tic_tac_toe::Board;

fn main() {
    let mut board = Board::default();
    println!("{}", board);

    loop {
        println!("{} to play", board.to_play());
        print!("global idx then local idx ");
        stdout().flush().unwrap();

        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();

        let parsed: IResult<&str, (u8, u8)> =
            tuple((terminated(parse_u8, tag(" ")), parse_u8))(buf.as_str());
        let (global, local) = parsed.unwrap().1;

        let Some(new_board) = board.play(global as usize, local as usize) else {
            println!("Invalid input");
            continue;
        };

        board = new_board;
        println!("{}", board);

        if board.is_tie() {
            println!("Tie!");
            break;
        } else if let Some(winner) = board.has_won() {
            println!(
                "{} has won!",
                match winner {
                    ultimate_tic_tac_toe::Player::X => 'X',
                    ultimate_tic_tac_toe::Player::O => 'O',
                }
            );
            break;
        }
    }
}
