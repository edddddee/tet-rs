use crate::piece::{Piece, PieceKind, PieceDimensions};

pub const GRID_COLUMNS: usize = 10;
pub const GRID_ROWS: usize = 20;

type GridMap = [[PieceKind; GRID_COLUMNS]; GRID_ROWS];

#[derive(Debug, Clone)]
pub struct Grid {
    // Map of the entire grid
    pub grid_map: GridMap,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            grid_map: [[PieceKind::None; GRID_COLUMNS]; GRID_ROWS],
        }
    }
}

impl Grid {
    pub fn widths(&self) -> [i32; GRID_ROWS] {
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

    pub fn heights(&self, below_row: i32) -> [i32; GRID_COLUMNS] {
        let mut result = [0i32; GRID_COLUMNS];
        (0..GRID_COLUMNS).for_each(|col| {
            result[col] = (0..below_row)
                .rev()
                .skip_while(|row| *row >= GRID_ROWS as i32)
                .skip_while(|row| self.grid_map[*row as usize][col] == PieceKind::None)
                .map(|row| row + 1)
                .next()
                .unwrap_or(0) as i32
        });
        return result;
    }

    pub fn is_within_bounds(x: i32, y: i32) -> bool {
        0 <= x && x < GRID_COLUMNS as i32 && 0 <= y && y < GRID_ROWS as i32
    }

    pub fn is_rect_inside(x_min: i32, x_max: i32, y_min: i32, y_max: i32) -> bool {
        0 <= x_min && x_max < GRID_COLUMNS as i32 && 0 <= y_min && y_max < GRID_ROWS as i32
    }

    pub fn set_cell(&mut self, x: i32, y: i32, kind: PieceKind) {
        if Self::is_within_bounds(x, y) {
            self.grid_map[y as usize][x as usize] = kind;
        }
    }

    pub fn get_cell(&self, x: i32, y: i32) -> PieceKind {
        assert!(
            Self::is_within_bounds(x, y),
            "({}, {}) is not on the grid!",
            x,
            y
        );
        self.grid_map[y as usize][x as usize]
    }

    pub fn clear_row(&mut self, row: usize) {
        assert!(row < GRID_ROWS, "Row {} out of bounds", row);
        (0..GRID_COLUMNS).for_each(|col| self.grid_map[row][col] = PieceKind::None)
    }
}
