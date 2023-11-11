#[macro_use]
extern crate static_assertions;

use bitvec::prelude::*;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::mem;
use std::sync::atomic::Ordering;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

type PieceMap = [(u8, u8); 4];
// Bit masks for each piece kind in its initial (unrotated) state.
const PIECE_I: PieceMap = [(0, 0), (0, 1), (0, 2), (0, 3)];
const PIECE_J: PieceMap = [(0, 0), (1, 0), (2, 0), (0, 1)];
const PIECE_L: PieceMap = [(0, 0), (1, 0), (2, 0), (2, 1)];
const PIECE_O: PieceMap = [(0, 0), (1, 0), (0, 1), (1, 1)];
const PIECE_S: PieceMap = [(0, 0), (1, 0), (1, 1), (2, 1)];
const PIECE_T: PieceMap = [(0, 0), (1, 0), (2, 0), (1, 1)];
const PIECE_Z: PieceMap = [(1, 0), (2, 0), (0, 1), (1, 1)];

#[derive(Component, Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Copy)]
enum Rotation {
    Rot0,
    Rot90,
    Rot180,
    Rot270,
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
    width: u8,
    height: u8,
    skirt: Vec<u8>,
}

impl PieceDimensions {
    fn new(piece_map: PieceMap) -> Self {
        Self {
            piece_map: piece_map,
            width: Self::get_width(piece_map),
            height: Self::get_height(piece_map),
            skirt: Self::get_skirt(piece_map),
        }
    }

    fn get_width(piece_map: PieceMap) -> u8 {
        piece_map
            .iter()
            .max_by(|(x1, _), (x2, _)| x1.cmp(x2))
            .unwrap()
            .0
            + 1
    }

    fn get_height(piece_map: PieceMap) -> u8 {
        piece_map
            .iter()
            .max_by(|(_, y1), (_, y2)| y1.cmp(y2))
            .unwrap()
            .1
            + 1
    }

    fn get_skirt(piece_map: PieceMap) -> Vec<u8> {
        (0..Self::get_width(piece_map))
            .into_iter()
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

    fn get_rotated_piece_maps(&self) -> [PieceMap; 4] {
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
                .map(|(x, y)| (*y, new_height - *x - 1))
                .collect::<Vec<_>>()
                .as_slice()
                .try_into()
                .unwrap();
            mem::swap(&mut new_width, &mut new_height);
        }
        rotated_pieces
    }
}

#[derive(Component)]
struct Piece {
    kind: PieceKind,
    piece_dimensions: PieceDimensions,
    rotation: Rotation,
    rotated_pieces: [PieceMap; 4],
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
        let piece_dimensions = match kind {
            PieceKind::I => PieceDimensions::new(PIECE_I),
            PieceKind::J => PieceDimensions::new(PIECE_J),
            PieceKind::L => PieceDimensions::new(PIECE_L),
            PieceKind::O => PieceDimensions::new(PIECE_O),
            PieceKind::S => PieceDimensions::new(PIECE_S),
            PieceKind::T => PieceDimensions::new(PIECE_T),
            PieceKind::Z => PieceDimensions::new(PIECE_Z),
            _ => panic!("Invalid piece type: {:?}", kind),
        };
        Piece {
            kind: kind,
            rotated_pieces: piece_dimensions.get_rotated_piece_maps(),
            piece_dimensions: piece_dimensions,
            rotation: Rotation::Rot0,
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
}

#[derive(Bundle)]
struct PieceBundle {
    piece: Piece,
    // TODO: Add sprite/visuals
}

const GRID_COLUMNS: usize = 10;
const GRID_ROWS: usize = 20;
type GridMap = [[PieceKind; GRID_ROWS]; GRID_COLUMNS];

struct Grid {
    // Map of the entire grid
    grid_map: GridMap,
    // Keeps track on when a line gets filled
    widths: [u8; GRID_ROWS],
    // Keeps track of the highest piece in each column
    heights: [u8; GRID_COLUMNS],
}

impl Grid {
    fn new(grid_map: GridMap) -> Self {
        let mut widths = [0u8; GRID_ROWS];
        for row in 0..GRID_ROWS {
            widths[row] = grid_map[row]
                .iter()
                .map(|kind| match kind {
                    PieceKind::None => 0,
                    _ => 1,
                })
                .sum();
        }

        let mut heights = [0u8; GRID_COLUMNS];
        for column in 0..GRID_COLUMNS {
            for row in 0..GRID_ROWS {
                heights[column] += if grid_map[row][column] == PieceKind::None {
                    0
                } else {
                    1
                };
            }
        }
        Self {
            grid_map,
            widths,
            heights,
        }
    }
}

fn main() {
    let mut x = Piece::new(PieceKind::Z);

    for i in 0..5 {
        println!("rot: {:?}", x.rotation);
        println!("{:?}", x);
        x.rotate_clockwise();
    }

    /*App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
    */
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());
}
