use std::io::{stdout, Read, Write};
use std::thread;
use std::time::Duration;
use termion::async_stdin;
use termion::color;
use termion::event::{parse_event, Event, Key};
use termion::raw::IntoRawMode;

use tetris::controls::{Controller, Button};
pub use tetris::grid::*;
pub use tetris::piece::*;
pub use tetris::utils::*;
pub use tetris::gamestate::*;

fn handle_keyboard_input(key: Key, gs: &mut GameState) {
    match key {
        Key::Up => gs.active_piece.rotate_clockwise(),
        Key::Down => {
            if gs.distance_to_drop() == 0 {
                gs.freeze_piece();
            } else {
                gs.active_piece.move_piece(Direction::Down);
            }
        }
        Key::Left => gs.active_piece.move_piece(Direction::Left),
        Key::Right => gs.active_piece.move_piece(Direction::Right),
        Key::Char('n') => gs.active_piece = Piece::new(rand::random()),
        Key::Char(' ') => gs.drop_piece(),
        _ => (),
    };
}

struct TerminalGame(GameState);

impl Controller for TerminalGame {
    type Key = Key;

    fn key_to_button(&mut self, key: Self::Key) -> Option<Button> {
        match key {
            Key::Up => Some(Button::RotateClockwise),
            Key::Left => Some(Button::MoveLeft),
            Key::Right => Some(Button::MoveRight),
            Key::Down => Some(Button::MoveDown),
            Key::Char(' ') => Some(Button::Drop),
            Key::Char('q') => Some(Button::Quit),
            _ => None,
        }
    }
}


fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    let mut stdin = async_stdin().bytes();
    let mut game = TerminalGame(GameState::default());
    write!(
        stdout,
        "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    )
    .unwrap();

    let ms_per_frame = 17;
    let ms_per_gravity_tick = 1000;
    let mut counter = 0;
    loop {
        if !game.0.is_running {
            break;
        }
        write!(stdout, "{}", termion::cursor::Goto(1, 1)).unwrap();
        // Clear screen and hide cursor
        write!(stdout, "{}{}", termion::clear::All, termion::cursor::Hide).unwrap();

        // Print the game (grid and active piece)
        write!(stdout, "{}{}", color::Fg(color::LightWhite), game.0).unwrap();

        // Poll events and handle keyboard input
        if let Some(Ok(b)) = stdin.next() {
            if let Ok(Event::Key(key)) = parse_event(b, &mut stdin) {
                    if let Some(button) = game.key_to_button(key) {
                        game.0.handle_button_input(button);
                    }
            }
        }
        

        // Wait ms_per_frame milliseconds before applying gravity
        thread::sleep(Duration::from_millis(ms_per_frame as u64));
        counter += ms_per_frame;

        if counter > ms_per_gravity_tick {
            counter %= ms_per_gravity_tick;
            game.0.apply_gravity();
        }
        game.0.on_update();

        stdout.flush().unwrap();
    }
}
