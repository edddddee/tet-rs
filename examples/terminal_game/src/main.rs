use tetris::controls::{Button, Controller};
use tetris::game::GameImplementation;
use tetris::gamestate::GameState;
use tetris::timer::*;

use std::collections::HashMap;
use std::io::{stdout, Read, StdoutLock, Write};
use std::thread;
use std::time::Duration;

use termion::color;
use termion::event::{self, parse_event, Event};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{async_stdin, AsyncReader};

struct TerminalGame<'a> {
    game_state: GameState,
    controls: HashMap<event::Key, Button>,
    stdout: RawTerminal<StdoutLock<'a>>,
    async_input_reader: std::io::Bytes<AsyncReader>,
    gravity_timer: Timer,
    update_timer: Timer,
}

impl<'a> Controller for TerminalGame<'a> {
    type Key = event::Key;

    fn key_to_button(&self, key: Self::Key) -> Option<Button> {
        self.controls.get(&key).copied()
    }
}

impl<'a> GameImplementation for TerminalGame<'a> {
    fn new() -> Self {
        Self {
            game_state: GameState::default(),
            controls: HashMap::from([
                (event::Key::Up, Button::RotateClockwise),
                (event::Key::Left, Button::MoveLeft),
                (event::Key::Right, Button::MoveRight),
                (event::Key::Down, Button::MoveDown),
                (event::Key::Char(' '), Button::Drop),
                (event::Key::Char('q'), Button::Quit),
            ]),
            stdout: stdout().lock().into_raw_mode().unwrap(),
            async_input_reader: async_stdin().bytes(),
            gravity_timer: Timer::new(Duration::from_millis(1000), Mode::Repeating),
            update_timer: Timer::new(Duration::from_millis(17), Mode::Repeating),
        }
    }

    fn handle_input(&mut self, key: <Self as Controller>::Key) {
        if let Some(button) = self.key_to_button(key) {
            self.game_state.on_button_pressed(button)
        }
    }

    fn on_setup(&mut self) {
        write!(
            self.stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1)
        )
        .unwrap();

        self.gravity_timer.start();
        self.update_timer.start();
    }

    fn on_update(&mut self) {
        // Goto top-left of terminal
        write!(self.stdout, "{}", termion::cursor::Goto(1, 1)).unwrap();
        // Clear screen and hide cursor
        write!(
            self.stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Hide
        )
        .unwrap();
        // Print the game (grid and active piece)
        write!(
            self.stdout,
            "{}{}",
            color::Fg(color::LightWhite),
            self.game_state
        )
        .unwrap();
        // Handle keyboard input
        if let Some(Ok(b)) = self.async_input_reader.next() {
            if let Ok(Event::Key(key)) = parse_event(b, &mut self.async_input_reader) {
                self.handle_input(key);
            }
        }
        

        if self.gravity_timer.finished() {
            self.game_state.apply_gravity();
        }
                
        self.game_state.on_update();

        self.stdout.flush().unwrap();
    }

    fn is_running(&self) -> bool {
        !self.game_state.gameover
    }

    fn run(&mut self) {
        self.on_setup();
        while self.is_running() {
            self.on_update();

            if !self.update_timer.finished() {
                thread::sleep(self.update_timer.time_left());
                self.update_timer.update();
            }
        }
    }

    fn quit(&mut self) {
        self.game_state.gameover = true;
    }
}

fn main() {
    TerminalGame::new().run();
}
