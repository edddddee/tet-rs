use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::io::{stdout, Read, Write};
use std::mem;
use std::thread;
use std::time::Duration;
use termion::async_stdin;
use termion::event::{parse_event, Event, Key};
use termion::raw::IntoRawMode;

type PieceMap = [(i32, i32); 4];
// Bit masks for each piece kind in its initial (unrotated) state.
const PIECE_I: PieceMap = [(0, 1), (1, 1), (2, 1), (3, 1)];
const PIECE_J: PieceMap = [(0, 1), (1, 1), (2, 1), (2, 0)];
const PIECE_L: PieceMap = [(0, 0), (0, 1), (1, 1), (2, 1)];
const PIECE_O: PieceMap = [(0, 0), (1, 0), (0, 1), (1, 1)];
const PIECE_S: PieceMap = [(0, 0), (1, 0), (1, 1), (2, 1)];
const PIECE_T: PieceMap = [(0, 0), (1, 0), (2, 0), (1, 1)];
const PIECE_Z: PieceMap = [(1, 0), (2, 0), (0, 1), (1, 1)];

#[derive(Debug, Clone, Copy, PartialEq)]
enum PieceKind {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
    None,
}

impl Distribution<PieceKind> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PieceKind {
        match rng.gen_range(0..=6) {
            0 => PieceKind::I,
            1 => PieceKind::J,
            2 => PieceKind::L,
            3 => PieceKind::O,
            4 => PieceKind::S,
            5 => PieceKind::T,
            _ => PieceKind::Z,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Rotation {
    Rot0,
    Rot90,
    Rot180,
    Rot270,
}

enum Direction {
    Down,
    Left,
    Right,
}

impl TryFrom<i32> for Rotation {
    type Error = ();
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Rotation::Rot0),
            1 => Ok(Rotation::Rot90),
            2 => Ok(Rotation::Rot180),
            3 => Ok(Rotation::Rot270),
            _ => Err(()),
        }
    }
}

impl std::ops::Add for Rotation {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::try_from((self as i32 + rhs as i32).rem_euclid(4)).unwrap()
    }
}

impl std::ops::AddAssign for Rotation {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self::try_from((*self as i32 + rhs as i32).rem_euclid(4)).unwrap()
    }
}

impl std::ops::Sub for Rotation {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::try_from((self as i32 - rhs as i32).rem_euclid(4)).unwrap()
    }
}

impl std::ops::SubAssign for Rotation {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Self::try_from((*self as i32 - rhs as i32).rem_euclid(4)).unwrap()
    }
}

struct PieceDimensions {
    piece_map: PieceMap,
    width: i32,
    height: i32,
    skirt: Vec<i32>,
}

impl PieceDimensions {
    fn new(piece_map: PieceMap) -> Self {
        Self {
            piece_map,
            width: Self::get_width(piece_map),
            height: Self::get_height(piece_map),
            skirt: Self::get_skirt(piece_map),
        }
    }

    fn x_min(piece_map: PieceMap) -> i32 {
        piece_map
            .iter()
            .min_by(|(x1, _), (x2, _)| x1.cmp(x2))
            .unwrap()
            .0
    }

    fn x_max(piece_map: PieceMap) -> i32 {
        piece_map
            .iter()
            .max_by(|(x1, _), (x2, _)| x1.cmp(x2))
            .unwrap()
            .0
    }

    fn y_min(piece_map: PieceMap) -> i32 {
        piece_map
            .iter()
            .min_by(|(_, y1), (_, y2)| y1.cmp(y2))
            .unwrap()
            .0
    }

    fn y_max(piece_map: PieceMap) -> i32 {
        piece_map
            .iter()
            .max_by(|(y1, _), (_, y2)| y1.cmp(y2))
            .unwrap()
            .0
    }
    fn get_width(piece_map: PieceMap) -> i32 {
        Self::x_max(piece_map) - Self::x_min(piece_map) + 1
    }

    fn get_height(piece_map: PieceMap) -> i32 {
        Self::y_max(piece_map) - Self::y_min(piece_map) + 1
    }

    fn get_skirt(piece_map: PieceMap) -> Vec<i32> {
        (Self::x_min(piece_map)..=Self::x_max(piece_map))
            .map(|w| {
                piece_map
                    .iter()
                    .filter(|(x, _)| *x == w)
                    .min_by(|(_, y1), (_, y2)| y1.cmp(y2))
                    .unwrap()
                    .1
            })
            .collect()
    }

    fn get_rotated_piece_maps(&self, origin: (f32, f32)) -> [PieceMap; 4] {
        let width = self.width;
        let height = self.height;
        let mut rotated_pieces = [
            self.piece_map,
            self.piece_map,
            self.piece_map,
            self.piece_map,
        ];
        let mut new_width = height;
        let mut new_height = width;
        for i in 1..4 {
            rotated_pieces[i] = rotated_pieces[i - 1]
                .iter()
                .map(|(x, y)| (*x as f32 - origin.0, *y as f32 - origin.1))
                .map(|(x, y)| (y, -x))
                .map(|(x, y)| ((x + origin.0) as i32, (y + origin.0) as i32))
                .collect::<Vec<_>>()
                .as_slice()
                .try_into()
                .unwrap();
            mem::swap(&mut new_width, &mut new_height);
        }
        rotated_pieces
    }
}

#[derive(Clone, Copy, Debug)]
struct GridPosition {
    x: i32,
    y: i32,
}

struct Piece {
    kind: PieceKind,
    piece_dimensions: PieceDimensions,
    rotation: Rotation,
    rotated_pieces: [PieceMap; 4],
    position: GridPosition,
    origin: (f32, f32),
}

impl fmt::Debug for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::from("");
        for y in (0..self.piece_dimensions.height).rev() {
            for x in 0..self.piece_dimensions.width {
                output += if self.piece_dimensions.piece_map.contains(&(x, y)) {
                    "#"
                } else {
                    " "
                };
            }
            if y > 0 {
                output += "\n"
            };
        }
        writeln!(f, "{output}")
    }
}

impl Piece {
    fn new(kind: PieceKind) -> Self {
        let piece_dimensions: PieceDimensions;
        let origin: (f32, f32);
        match kind {
            PieceKind::I => {
                piece_dimensions = PieceDimensions::new(PIECE_I);
                origin = (1.5, 1.5);
            }
            PieceKind::L => {
                piece_dimensions = PieceDimensions::new(PIECE_L);
                origin = (1.0, 1.0);
            }
            PieceKind::J => {
                piece_dimensions = PieceDimensions::new(PIECE_J);
                origin = (1.0, 1.0);
            }
            PieceKind::O => {
                piece_dimensions = PieceDimensions::new(PIECE_O);
                origin = (0.5, 0.5);
            }
            PieceKind::S => {
                piece_dimensions = PieceDimensions::new(PIECE_S);
                origin = (1.0, 1.0);
            }
            PieceKind::Z => {
                piece_dimensions = PieceDimensions::new(PIECE_Z);
                origin = (1.0, 1.0);
            }
            PieceKind::T => {
                piece_dimensions = PieceDimensions::new(PIECE_T);
                origin = (1.0, 1.0);
            }
            _ => panic!("Invalid piece type: {:?}", kind),
        };
        let xpos = GRID_COLUMNS as i32 / 2 - piece_dimensions.width / 2;
        let ypos = GRID_ROWS as i32 - piece_dimensions.height;
        Piece {
            kind,
            rotated_pieces: piece_dimensions.get_rotated_piece_maps(origin),
            piece_dimensions,
            rotation: Rotation::Rot0,
            position: GridPosition { x: xpos, y: ypos },
            origin,
        }
    }

    fn rotate(&mut self, rot: Rotation) {
        self.rotation += rot;
        self.piece_dimensions = PieceDimensions::new(self.rotated_pieces[self.rotation as usize]);
    }

    fn rotate_clockwise(&mut self) {
        self.rotate(Rotation::Rot90);
    }

    fn rotate_counter_clockwise(&mut self) {
        self.rotate(Rotation::Rot270);
    }

    fn rotate_180(&mut self) {
        self.rotate(Rotation::Rot180);
    }

    fn move_piece(&mut self, direction: Direction) {
        match direction {
            Direction::Down => self.position.y -= 1,
            Direction::Left => self.position.x -= 1,
            Direction::Right => self.position.x += 1,
        }
    }
}

const GRID_COLUMNS: usize = 10;
const GRID_ROWS: usize = 20;
type GridMap = [[PieceKind; GRID_COLUMNS]; GRID_ROWS];

struct Grid {
    // Map of the entire grid
    grid_map: GridMap,
}

impl Grid {
    fn new() -> Self {
        let grid_map: GridMap = [[PieceKind::None; GRID_COLUMNS]; GRID_ROWS];

        // let mut heights = [0i32; GRID_COLUMNS];
        // for column in 0..GRID_COLUMNS {
        //     for row in (0..GRID_ROWS).rev() {
        //         if grid_map[row][column] != PieceKind::None {
        //             heights[column] = row as i32;
        //             break;
        //         };
        //     }
        // }
        Self { grid_map }
    }

    fn widths(&self) -> [i32; GRID_ROWS] {
        let mut result = [0i32; GRID_ROWS];
        for row in 0..GRID_ROWS {
            result[row] = self.grid_map[row]
                .iter()
                .map(|kind| match kind {
                    PieceKind::None => 0,
                    _ => 1,
                })
                .sum();
        }
        result
    }

    fn heights(&self, below_row: i32) -> [i32; GRID_COLUMNS] {
        let mut result = [0i32; GRID_COLUMNS];
        (0..GRID_COLUMNS).for_each(|col| {
            result[col] = (0..below_row)
                .rev()
                .skip_while(|row| self.grid_map[*row as usize][col] == PieceKind::None)
                .map(|row| row + 1)
                .next()
                .unwrap_or(0) as i32
        });
        return result;
    }

    fn is_within_bounds(x: i32, y: i32) -> bool {
        0 <= x && x < GRID_COLUMNS as i32 && 0 <= y && y < GRID_ROWS as i32
    }

    fn set_cell(&mut self, x: i32, y: i32, kind: PieceKind) {
        if Self::is_within_bounds(x, y) {
            self.grid_map[y as usize][x as usize] = kind;
        }
    }
}

struct GameState {
    grid: Grid,
    active_piece: Piece,
}

impl GameState {
    fn new() -> Self {
        Self {
            grid: Grid::new(),
            active_piece: Piece::new(rand::random()),
        }
    }

    fn apply_gravity(&mut self) {
        self.active_piece.position.y -= 1;
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

    fn drop_piece(&mut self) {
        let (x, y) = (self.active_piece.position.x, self.active_piece.position.y);
        let xmin = PieceDimensions::x_min(self.active_piece.piece_dimensions.piece_map);
        let y_drop: i32 = (0..self.active_piece.piece_dimensions.width)
            .filter(|w| 0 <= (x + w + xmin) && (x + w + xmin) < GRID_COLUMNS as i32)
            .map(|w| {
                self.active_piece.piece_dimensions.skirt[w as usize] + y
                    - self.grid.heights(y + PieceDimensions::y_min(self.active_piece.piece_dimensions.piece_map))[(x + w + xmin) as usize]
            })
            .min()
            .unwrap();
        self.active_piece.position.y -= y_drop;
        self.freeze_piece();
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
        Key::Down => gs.active_piece.move_piece(Direction::Down),
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
    let ms_per_gravity_tick = 1000;
    let mut counter = 0;
    loop {
        write!(stdout, "{}", termion::clear::All).unwrap();
        write!(stdout, "{gs}").unwrap();
        /* let b = stdin.next();
        if let Some(Ok(b'q')) = b {
            break;
        } */
        if let Some(Ok(b)) = stdin.next() {
            if let Ok(Event::Key(key)) = parse_event(b, &mut stdin) {
                if let Key::Char('q') = key {
                    break;
                } else {
                    handle_keyboard_input(key, &mut gs);
                }
            }
        }
        thread::sleep(Duration::from_millis(ms_per_frame as u64));
        counter += ms_per_frame;
        if counter > ms_per_gravity_tick {
            counter %= ms_per_gravity_tick;
            gs.apply_gravity();
        }
        write!(stdout, "\r\n").unwrap();
        for col in 0..GRID_COLUMNS {
            write!(stdout, "{}", gs.grid.heights(GRID_ROWS as i32)[col]).unwrap();
        }
        write!(stdout, "{}", termion::cursor::Goto(1, 1)).unwrap();
    }

    /* App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, setup)
    .run(); */
}
