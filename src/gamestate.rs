use std::fmt;

use termion::color;

use crate::controls::Button;
use crate::grid::{Grid, GRID_COLUMNS, GRID_VISIBLE_ROWS};
use crate::piece::{self, Piece, PieceDimensions, PieceKind};
use crate::utils::{Direction, Rotation};

#[derive(Debug, Clone)]
pub struct GameState {
    pub grid: Grid,
    pub active_piece: Piece,
    pub gameover: bool,
    pub current_piece_bag: Vec<PieceKind>,
    pub next_piece_bag: Vec<PieceKind>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            grid: Grid::default(),
            active_piece: Piece::new(rand::random()),
            gameover: false,
            current_piece_bag: piece::gen_piece_bag().to_vec(),
            next_piece_bag: piece::gen_piece_bag().to_vec(),
        }
    }
}

impl GameState {
    pub fn apply_gravity(&mut self) {
        match self.distance_to_drop() {
            0 => self.freeze_piece(),
            _ => self.active_piece.move_piece(Direction::Down),
        }
    }

    pub fn freeze_piece(&mut self) {
        let (x, y) = (self.active_piece.position.x, self.active_piece.position.y);
        if self.active_piece.y_min() >= GRID_VISIBLE_ROWS as i32 {
            self.gameover = true;
        } else {
            self.active_piece
                .piece_dimensions
                .piece_map
                .iter()
                .for_each(|(px, py)| {
                    self.grid.set_cell(x + px, y + py, self.active_piece.kind);
                });
            let new_piece_kind = self.current_piece_bag.pop().unwrap_or_else(|| {
                self.current_piece_bag =
                    std::mem::replace(&mut self.next_piece_bag, piece::gen_piece_bag().to_vec());
                self.current_piece_bag.pop().unwrap()
            });
            let new_piece = Piece::new(new_piece_kind);
            if self.grid.overlaps(&new_piece) {
                self.gameover = true;
            } else {
                self.active_piece = new_piece;
            }
        }
    }

    pub fn clear_full_rows(&mut self) {
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

    pub fn distance_to_drop(&self) -> i32 {
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

    pub fn drop_piece(&mut self) {
        self.active_piece.position.y -= self.distance_to_drop();
        self.freeze_piece();
    }

    pub fn on_update(&mut self) {
        self.clear_full_rows();
    }

    fn is_valid_move(&self, dir: Direction) -> bool {
        let (dx, dy): (i32, i32) = match dir {
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
            Direction::Down => (0, -1),
        };
        for (rx, ry) in self.active_piece.piece_dimensions.piece_map {
            let (x, y) = (
                self.active_piece.position.x + rx + dx,
                self.active_piece.position.y + ry + dy,
            );
            if !(Grid::is_within_bounds(x, y) && self.grid.get_cell(x, y) == PieceKind::None) {
                return false;
            }
        }
        true
    }

    fn try_move(&mut self, dir: Direction) {
        if self.is_valid_move(dir) {
            self.active_piece.move_piece(dir)
        }
    }

    fn is_valid_rotation(&self, rot: Rotation, offset: (i32, i32)) -> bool {
        let rotated_piecemap =
            self.active_piece.rotated_pieces[(self.active_piece.rotation + rot) as usize];

        for (rx, ry) in rotated_piecemap {
            let (x, y) = (
                self.active_piece.position.x + rx + offset.0,
                self.active_piece.position.y + ry + offset.1,
            );
            if !(Grid::is_within_bounds(x, y) && self.grid.get_cell(x, y) == PieceKind::None) {
                return false;
            }
        }
        true
    }

    fn try_rotate(&mut self, rot: Rotation) {
        let transition = (
            self.active_piece.rotation,
            (self.active_piece.rotation + rot),
        );

        let offset_list = match self.active_piece.kind {
            PieceKind::I => match transition {
                (Rotation::Rot0, Rotation::Rot90) => [(-2, 0), (1, 0), (-2, -1), (1, 2)],
                (Rotation::Rot90, Rotation::Rot0) => [(2, 0), (-1, 0), (2, 1), (-1, -2)],
                (Rotation::Rot90, Rotation::Rot180) => [(-1, 0), (2, 0), (-1, 2), (2, -1)],
                (Rotation::Rot180, Rotation::Rot90) => [(1, 0), (-2, 0), (1, -2), (-2, 1)],
                (Rotation::Rot180, Rotation::Rot270) => [(2, 0), (-1, 0), (2, 1), (-1, -2)],
                (Rotation::Rot270, Rotation::Rot180) => [(-2, 0), (1, 0), (-2, -1), (1, 2)],
                (Rotation::Rot270, Rotation::Rot0) => [(1, 0), (-2, 0), (1, -2), (-2, 1)],
                (Rotation::Rot0, Rotation::Rot270) => [(-1, 0), (2, 0), (-1, 2), (2, -1)],
                _ => unreachable!(),
            },
            _ => match transition {
                (Rotation::Rot0, Rotation::Rot90) => [(-1, 0), (-1, 1), (0, -2), (-1, -2)],
                (Rotation::Rot90, Rotation::Rot0) => [(1, 0), (1, -1), (0, 2), (1, 2)],
                (Rotation::Rot90, Rotation::Rot180) => [(1, 0), (1, -1), (0, 2), (1, 2)],
                (Rotation::Rot180, Rotation::Rot90) => [(-1, 0), (-1, 1), (0, -2), (-1, -2)],
                (Rotation::Rot180, Rotation::Rot270) => [(1, 0), (1, 1), (0, -2), (1, -2)],
                (Rotation::Rot270, Rotation::Rot180) => [(-1, 0), (-1, -1), (0, 2), (-1, 2)],
                (Rotation::Rot270, Rotation::Rot0) => [(-1, 0), (-1, -1), (0, 2), (-1, 2)],
                (Rotation::Rot0, Rotation::Rot270) => [(1, 0), (1, 1), (0, -2), (1, -2)],
                _ => unreachable!(),
            },
        };
        if self.is_valid_rotation(rot, (0, 0)) {
            self.active_piece.rotate(rot)
        } else {
            for offset in offset_list {
                if self.is_valid_rotation(rot, offset) {
                    self.active_piece.position.x += offset.0;
                    self.active_piece.position.y += offset.1;
                    self.active_piece.rotate(rot);
                    break;
                }
            }
        };
    }

    pub fn on_button_pressed(&mut self, button: Button) {
        match button {
            Button::Quit => self.gameover = true,
            Button::MoveDown => self.try_move(Direction::Down),
            Button::MoveLeft => self.try_move(Direction::Left),
            Button::MoveRight => self.try_move(Direction::Right),
            Button::Drop => self.drop_piece(),
            Button::RotateClockwise => self.try_rotate(Rotation::Rot90),
        };
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ydrop = self.distance_to_drop();
        for y in (0..GRID_VISIBLE_ROWS).rev() {
            for x in 0..GRID_COLUMNS {
                let rel_x = x as i32 - self.active_piece.position.x;
                let rel_y = y as i32 - self.active_piece.position.y;

                if self
                    .active_piece
                    .piece_dimensions
                    .piece_map
                    .contains(&(rel_x, rel_y))
                {
                    write!(f, "{}", self.active_piece.kind)?;
                } else if self
                    .active_piece
                    .piece_dimensions
                    .piece_map
                    .contains(&(rel_x, rel_y + ydrop))
                {
                    // Draw ghost piece
                    write!(f, "{}{}", color::Fg(color::Rgb(150,150,150)), piece::BLOCK_STR)?;
                } else {
                    write!(f, "{}", self.grid.grid_map[y][x])?;
                }
            }
            write!(f, "\r\n")?;
        }
        Ok(())
    }
}
