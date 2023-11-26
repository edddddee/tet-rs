use tetris::controls::{Button, Controller};
use tetris::game::GameImplementation;
use tetris::gamestate::GameState;

use std::collections::HashMap;
use std::io::{stdout, Read, Write, StdoutLock};
use std::thread;
use std::time::Duration;

use termion::{async_stdin, AsyncReader};
use termion::color;
use termion::event::{self, parse_event, Event, Key};
use termion::raw::{IntoRawMode, RawTerminal};

struct TerminalGame<'a> {
    game_state: GameState,
    controls: HashMap<event::Key, Button>,
    stdout: RawTerminal<StdoutLock<'a>>,
    async_input_reader: std::io::Bytes<AsyncReader>,
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
        }
    }
    fn handle_input(&mut self, key: <Self as Controller>::Key) {
        if let Some(button) = self.key_to_button(key) {
            self.game_state.on_button_pressed(button)
        }
    }
    fn on_update(&mut self) {
        if let Some(Ok(b)) = self.async_input_reader.next() {
            if let Ok(Event::Key(key)) = parse_event(b, &mut self.async_input_reader) {
                self.handle_input(key);
            }
        }
    }
}

fn main() {}
