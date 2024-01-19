use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dashmap::DashMap;
use ratatui::{
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::Widget,
    Terminal,
};
use ultimate_tic_tac_toe::{Board, IndividualBoard, LocalBoardState, Player};

fn to_xy(pos: (usize, usize)) -> (u16, u16) {
    let start_x = pos.0 % 3 * 8;
    let start_y = pos.0 / 3 * 8;

    let x = start_x + pos.1 % 3 * 2;
    let y = start_y + pos.1 / 3 * 2;

    (x as u16, y as u16)
}

fn render_group(board: IndividualBoard, x_start: u16, y_start: u16, active: bool) -> Buffer {
    let mut buf = Buffer::empty(Rect::new(x_start, y_start, 5, 5));
    if let Some(state) = board.get_state() {
        let char = match state {
            LocalBoardState::Win(Player::X) => 'X',
            LocalBoardState::Win(Player::O) => 'O',
            LocalBoardState::Tie => 'T',
        };
        buf.get_mut(x_start + 2, y_start + 2).set_char(char);
        return buf;
    }

    let color = if active { Color::Gray } else { Color::DarkGray };

    let mut blank_line = Line::raw(" | | ");
    blank_line.patch_style(Style::new().fg(color));
    buf.set_line(x_start, y_start, &blank_line, 5);
    buf.set_line(x_start, y_start + 2, &blank_line, 5);
    buf.set_line(x_start, y_start + 4, &blank_line, 5);

    board.0.iter().enumerate().for_each(|(idx, square)| {
        if let Some(square) = square {
            let (x, y) = to_xy((0, idx));

            buf.get_mut(x + x_start, y + y_start)
                .set_char(match square {
                    Player::X => 'X',
                    Player::O => 'O',
                })
                .set_fg(Color::White);
        }
    });

    let mut inter_line = Line::raw("-+-+-");
    inter_line.patch_style(Style::new().fg(color));
    buf.set_line(x_start, y_start + 1, &inter_line, 5);
    buf.set_line(x_start, y_start + 3, &inter_line, 5);

    buf
}

struct BoardWidget {
    selected: (usize, usize),
    last_played: Option<(usize, usize)>,
    board: Board,
}

impl Widget for BoardWidget {
    fn render(self, _area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let line = Line::raw("      |       |      ");
        for y in (0..21).filter(|y| *y != 6 && *y != 14) {
            buf.set_line(0, y, &line, 21);
        }

        let inter_line = Line::raw("------+-------+------");
        buf.set_line(0, 6, &inter_line, 21);
        buf.set_line(0, 14, &inter_line, 21);

        for (global, board) in self.board.locals.iter().enumerate() {
            let (x, y) = to_xy((global, 0));

            buf.merge(&render_group(
                board.to_owned(),
                x,
                y,
                self.board.global_idx.is_none() || self.board.global_idx.unwrap() == global,
            ));
        }

        let (selected_x, selected_y) = to_xy(self.selected);

        buf.get_mut(selected_x as u16, selected_y as u16)
            .set_bg(ratatui::style::Color::Red);

        if let Some(last_played) = self.last_played {
            let (last_x, last_y) = to_xy(last_played);

            buf.get_mut(last_x as u16, last_y as u16)
                .set_bg(ratatui::style::Color::Green);
        }
    }
}

fn main() -> anyhow::Result<()> {
    rayon::ThreadPoolBuilder::new()
        .stack_size(8000000)
        .build_global()
        .unwrap();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut board = Board::default();
    let mut current_pos = (1, 0);
    let mut last_played = None;

    let eval_cache = DashMap::new();
    let eval_cache2 = DashMap::new();
    let cache = DashMap::new();

    fn move_left((global, local): &mut (usize, usize)) {
        if *local % 3 == 0 && *global % 3 == 0 {
            return;
        } else if *local % 3 == 0 {
            *global -= 1;
            *local += 2;
        } else {
            *local -= 1;
        }
    }

    fn move_right((global, local): &mut (usize, usize)) {
        if *local % 3 == 2 && *global % 3 == 2 {
            return;
        } else if *local % 3 == 2 {
            *global += 1;
            *local -= 2;
        } else {
            *local += 1;
        }
    }

    fn move_up((global, local): &mut (usize, usize)) {
        if *local / 3 == 0 && *global / 3 == 0 {
            return;
        } else if *local / 3 == 0 {
            *global -= 3;
            *local += 6;
        } else {
            *local -= 3;
        }
    }

    fn move_down((global, local): &mut (usize, usize)) {
        if *local / 3 == 2 && *global / 3 == 2 {
            return;
        } else if *local / 3 == 2 {
            *global += 3;
            *local -= 6;
        } else {
            *local += 3;
        }
    }

    loop {
        terminal.draw(|frame| {
            frame.render_widget(
                BoardWidget {
                    board: board.clone(),
                    selected: current_pos,
                    last_played,
                },
                Rect::new(0, 0, 21, 21),
            )
        })?;

        match event::read().unwrap() {
            Event::Key(key) if key.code == KeyCode::Char('q') => break,
            Event::Key(key) if key.code == KeyCode::Left => move_left(&mut current_pos),
            Event::Key(key) if key.code == KeyCode::Right => move_right(&mut current_pos),
            Event::Key(key) if key.code == KeyCode::Up => move_up(&mut current_pos),
            Event::Key(key) if key.code == KeyCode::Down => move_down(&mut current_pos),
            Event::Key(key) if key.code == KeyCode::Enter => {
                if let Some(new_board) = board.play(current_pos.0, current_pos.1) {
                    board = new_board;

                    if board.has_won().is_some() || board.is_tie() {
                        break;
                    }

                    terminal.draw(|frame| {
                        frame.render_widget(
                            BoardWidget {
                                board: board.clone(),
                                selected: current_pos,
                                last_played,
                            },
                            Rect::new(0, 0, 21, 21),
                        )
                    })?;

                    let ((global, local), _eval, _depth) =
                        board.minimax(6, &cache, &eval_cache, &eval_cache2);

                    last_played = Some((global, local));

                    current_pos.0 = local;

                    let Some(new_board) = board.play(global as usize, local as usize) else {
                        panic!("Invalid input");
                    };

                    board = new_board;

                    if board.has_won().is_some() || board.is_tie() {
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    println!("{}", board);

    if let Some(winner) = board.has_won() {
        println!(
            "{} has won!",
            match winner {
                Player::X => 'X',
                Player::O => 'O',
            }
        );
    } else if board.is_tie() {
        println!("Tie!");
    }

    Ok(())
}
