use std::fmt;
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

#[derive(Debug, Clone)]
struct GameState {
    grid: Grid,
    active_piece: Piece,
}

impl GameState {
    fn new() -> Self {
        Self {
            grid: Grid::default(),
            active_piece: Piece::new(rand::random()),
        }
    }

    fn apply_gravity(&mut self) {
        if self.distance_to_drop() == 0 {
            self.freeze_piece();
        } else {
            self.active_piece.move_piece(Direction::Down);
        }
    }

    fn freeze_piece(&mut self) {
        let (x, y) = (self.active_piece.position.x, self.active_piece.position.y);
        self.active_piece
            .piece_dimensions
            .piece_map
            .iter()
            .for_each(|(px, py)| self.grid.set_cell(x + px, y + py, self.active_piece.kind));
        self.active_piece = Piece::new(rand::random());
    }

    fn clear_full_rows(&mut self) {
        let mut rows_to_clear: i32 = 0;
        let mut new_gs = self.clone();
        let drop_amounts: Vec<_> = self
            .grid
            .widths()
            .iter()
            .enumerate()
            .map(|(row, w)| {
                if *w == GRID_COLUMNS as i32 {
                    new_gs.grid.clear_row(row);
                    rows_to_clear += 1;
                    0
                } else {
                    rows_to_clear
                }
            })
            .collect();
        drop_amounts
            .into_iter()
            .enumerate()
            .filter(|(_, drop_amt)| *drop_amt > 0)
            .for_each(|(row, drop_amt)| {
                (0..GRID_COLUMNS).for_each(|col| {
                    new_gs.grid.set_cell(
                        col as i32,
                        row as i32 - drop_amt,
                        self.grid.get_cell(col as i32, row as i32),
                    )
                })
            });
        *self = new_gs;
    }

    fn distance_to_drop(&self) -> i32 {
        let (x, y) = (self.active_piece.position.x, self.active_piece.position.y);
        let xmin = PieceDimensions::x_min(self.active_piece.piece_dimensions.piece_map);
        (0..self.active_piece.piece_dimensions.width)
            .filter(|w| 0 <= (x + w + xmin) && (x + w + xmin) < GRID_COLUMNS as i32)
            .map(|w| {
                self.active_piece.piece_dimensions.skirt[w as usize] + y
                    - self
                        .grid
                        .heights(self.active_piece.piece_dimensions.skirt[w as usize] + y)
                        [(x + w + xmin) as usize]
            })
            .min()
            .unwrap()
    }

    fn drop_piece(&mut self) {
        self.active_piece.position.y -= self.distance_to_drop();
        self.freeze_piece();
    }

    fn on_update(&mut self) {
        self.clear_full_rows();
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        for y in (0..GRID_ROWS).rev() {
            for x in 0..GRID_COLUMNS {
                let xcord = x as i8 - self.active_piece.position.x as i8;
                let ycord = y as i8 - self.active_piece.position.y as i8;
                if xcord >= 0
                    && ycord >= 0
                    && self
                        .active_piece
                        .piece_dimensions
                        .piece_map
                        .contains(&(xcord as i32, ycord as i32))
                {
                    output += "#";
                } else {
                    match self.grid.grid_map[y][x] {
                        PieceKind::None => output += ".",
                        _ => output += "#",
                    }
                }
            }
            output += "\r\n";
        }
        writeln!(f, "{}", output)
    }
}

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
    let mut gs = GameState::new();
    write!(
        stdout,
        "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    )
    .unwrap();

    let ms_per_frame = 17;
    let ms_per_gravity_tick = 9999999;
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

        write!(stdout, "{}", gs.distance_to_drop()).unwrap();
        write!(stdout, "{}", termion::cursor::Goto(1, 1)).unwrap();
        stdout.flush().unwrap();
    }

    /* App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, setup)
    .run(); */
}
