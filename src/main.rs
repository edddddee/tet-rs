use std::io::{stdout, Read, Write};
use std::thread;
use std::time::Duration;
use termion::async_stdin;
use termion::color;
use termion::event::{parse_event, Event, Key};
use termion::raw::IntoRawMode;

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

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    let mut stdin = async_stdin().bytes();
    let mut gs = GameState::default();
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
        // Clear screen and hide cursor
        write!(stdout, "{}{}", termion::clear::All, termion::cursor::Hide).unwrap();

        // Print the game (grid and active piece)
        write!(stdout, "{}{gs}", color::Fg(color::LightWhite)).unwrap();

        // Poll events and handle keyboard input
        if let Some(Ok(b)) = stdin.next() {
            if let Ok(Event::Key(key)) = parse_event(b, &mut stdin) {
                if let Key::Char('q') = key {
                    break;
                } else {
                    handle_keyboard_input(key, &mut gs);
                }
            }
        }

        // Wait ms_per_frame milliseconds before applying gravity
        thread::sleep(Duration::from_millis(ms_per_frame as u64));
        counter += ms_per_frame;

        if counter > ms_per_gravity_tick {
            counter %= ms_per_gravity_tick;
            gs.apply_gravity();
        }
        gs.on_update();

        write!(stdout, "{}", termion::cursor::Goto(1, 1)).unwrap();
        stdout.flush().unwrap();
    }
}
