use std::fmt;

use crate::utils::Direction;
use crate::grid::{GRID_ROWS, GRID_COLUMNS, Grid};
use crate::piece::{Piece, PieceDimensions};

#[derive(Debug, Clone)]
pub struct GameState {
    pub grid: Grid,
    pub active_piece: Piece,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            grid: Grid::default(),
            active_piece: Piece::new(rand::random()),
        }
    }

}

impl GameState {
    pub fn apply_gravity(&mut self) {
        if self.distance_to_drop() == 0 {
            self.freeze_piece();
        } else {
            self.active_piece.move_piece(Direction::Down);
        }
    }

    pub fn freeze_piece(&mut self) {
        let (x, y) = (self.active_piece.position.x, self.active_piece.position.y);
        self.active_piece
            .piece_dimensions
            .piece_map
            .iter()
            .for_each(|(px, py)| self.grid.set_cell(x + px, y + py, self.active_piece.kind));
        self.active_piece = Piece::new(rand::random());
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
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
                    write!(f, "{}", self.active_piece.kind)?;
                } else {
                    write!(f, "{}", self.grid.grid_map[y][x])?;
                }
            }
            write!(f, "\r\n")?;
        }
        Ok(())
    }
}
