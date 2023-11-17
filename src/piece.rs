use crate::grid::{GRID_COLUMNS, GRID_ROWS};
use crate::utils::{Direction, Rotation};

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::fmt;
use std::mem;

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
pub enum PieceKind {
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

#[derive(Debug, Clone)]
pub struct PieceDimensions {
    pub piece_map: PieceMap,
    pub width: i32,
    pub height: i32,
    pub skirt: Vec<i32>,
}

impl PieceDimensions {
    pub fn new(piece_map: PieceMap) -> Self {
        Self {
            piece_map,
            width: Self::get_width(piece_map),
            height: Self::get_height(piece_map),
            skirt: Self::get_skirt(piece_map),
        }
    }

    pub fn x_min(piece_map: PieceMap) -> i32 {
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

    pub fn y_min(piece_map: PieceMap) -> i32 {
        piece_map
            .iter()
            .min_by(|(_, y1), (_, y2)| y1.cmp(y2))
            .unwrap()
            .0
    }

    pub fn y_max(piece_map: PieceMap) -> i32 {
        piece_map
            .iter()
            .max_by(|(y1, _), (_, y2)| y1.cmp(y2))
            .unwrap()
            .0
    }

    pub fn get_width(piece_map: PieceMap) -> i32 {
        Self::x_max(piece_map) - Self::x_min(piece_map) + 1
    }

    pub fn get_height(piece_map: PieceMap) -> i32 {
        Self::y_max(piece_map) - Self::y_min(piece_map) + 1
    }

    pub fn get_skirt(piece_map: PieceMap) -> Vec<i32> {
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
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone)]
pub struct Piece {
    pub kind: PieceKind,
    pub piece_dimensions: PieceDimensions,
    pub rotation: Rotation,
    pub rotated_pieces: [PieceMap; 4],
    pub position: GridPosition,
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
    pub fn new(kind: PieceKind) -> Self {
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
        }
    }

    pub fn rotate(&mut self, rot: Rotation) {
        self.rotation += rot;
        self.piece_dimensions = PieceDimensions::new(self.rotated_pieces[self.rotation as usize]);
    }

    pub fn rotate_clockwise(&mut self) {
        self.rotate(Rotation::Rot90);
    }

    pub fn rotate_counter_clockwise(&mut self) {
        self.rotate(Rotation::Rot270);
    }

    pub fn rotate_180(&mut self) {
        self.rotate(Rotation::Rot180);
    }

    pub fn move_piece(&mut self, direction: Direction) {
        match direction {
            Direction::Down => self.position.y -= 1,
            Direction::Left => self.position.x -= 1,
            Direction::Right => self.position.x += 1,
        }
    }
}
