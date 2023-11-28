use crate::piece::{Piece, PieceKind};

pub const GRID_COLUMNS: usize = 10;
pub const GRID_ROWS: usize = 24;
pub const GRID_VISIBLE_ROWS: usize = 20;

type GridMap = [[PieceKind; GRID_COLUMNS]; GRID_ROWS];

#[derive(Debug, Clone)]
pub struct Grid {
    // Map of the entire grid
    pub grid_map: GridMap,
}

impl Default for Grid {
    fn default() -> Self {
        Self::new()
    }
}

impl From<GridMap> for Grid {
    fn from(map: GridMap) -> Self {
        Self { grid_map: map }
    }
}

impl Grid {
    pub fn new() -> Self {
        Self {
            grid_map: [[PieceKind::None; GRID_COLUMNS]; GRID_ROWS],
        }
    }

    pub fn widths(&self) -> [i32; GRID_ROWS] {
        let mut result = [0i32; GRID_ROWS];
        result.iter_mut().enumerate().for_each(|(row, width)| {
            *width = self.grid_map[row]
                .iter()
                .map(|kind| match kind {
                    PieceKind::None => 0,
                    _ => 1,
                })
                .sum();
        });
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
                .unwrap_or(0)
        });
        result
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

    pub fn overlaps(&mut self, piece: &Piece) -> bool {
        let (x0, y0) = (piece.position.x, piece.position.y);
        for (px, py) in piece.piece_dimensions.piece_map {
            let (x, y) = (x0 + px, y0 + py);
            match self.get_cell(x, y) {
                PieceKind::None => (),
                _ => return true,
            };
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_grid() {
        // Empty grid
        let grid = Grid::new();
        // All widths should be 0
        assert_eq!(grid.widths(), [0i32; GRID_ROWS]);
        // All heights should be 0
        assert_eq!(grid.heights(GRID_ROWS as i32), [0i32; GRID_COLUMNS]);
    }

    #[test]
    fn full_grid() {
        // Grid completely filled with I piece blocks
        let grid_map: GridMap = [[PieceKind::I; GRID_COLUMNS]; GRID_ROWS];
        let grid = Grid::from(grid_map);
        // All rows should be GRID_COLUMNS wide
        assert_eq!(grid.widths(), [GRID_COLUMNS as i32; GRID_ROWS]);
        // All columns should be GRID_ROWS high
        assert_eq!(
            grid.heights(GRID_ROWS as i32),
            [GRID_ROWS as i32; GRID_COLUMNS]
        );
    }

    #[test]
    fn bounds_checking() {
        // Check all positions that SHOULD be within bounds
        for x in 0..(GRID_COLUMNS as i32) {
            for y in 0..(GRID_ROWS as i32) {
                assert!(Grid::is_within_bounds(x, y))
            }
        }
        // Test off-by-one cases (should be out of bounds)
        assert!(!Grid::is_within_bounds(-1, 0));
        assert!(!Grid::is_within_bounds(GRID_COLUMNS as i32, 0));
        assert!(!Grid::is_within_bounds(0, -1));
        assert!(!Grid::is_within_bounds(0, GRID_ROWS as i32));
        // Try valid rectangles with different x values but same y values
        for x1 in 0..(GRID_COLUMNS as i32) {
            for x2 in x1..(GRID_COLUMNS as i32) {
                let (y1, y2) = (0i32, GRID_ROWS as i32 - 1);
                assert!(Grid::is_rect_inside(x1, x2, y1, y2));
            }
        }
        // Try valid rectangles with different y values but same x values
        for y1 in 0..(GRID_ROWS as i32) {
            for y2 in y1..(GRID_ROWS as i32) {
                let (x1, x2) = (0i32, GRID_COLUMNS as i32 - 1);
                assert!(Grid::is_rect_inside(x1, x2, y1, y2));
            }
        }
        // Test off-by-one rectangles
        assert!(!Grid::is_rect_inside(-1, 0, 0, 1));
        assert!(!Grid::is_rect_inside(
            GRID_COLUMNS as i32 - 1,
            GRID_COLUMNS as i32,
            0,
            1
        ));
        assert!(!Grid::is_rect_inside(0, 1, -1, 0));
        assert!(!Grid::is_rect_inside(
            0,
            1,
            GRID_ROWS as i32 - 1,
            GRID_ROWS as i32
        ));
    }

    #[test]
    fn row_clearing() {
        // Grid completely filled with I piece blocks
        let grid_map: GridMap = [[PieceKind::I; GRID_COLUMNS]; GRID_ROWS];
        let mut grid = Grid::from(grid_map);
        // Clear every row
        for row in 0..GRID_ROWS {
            grid.clear_row(row)
        }
        // Now grid should be completely empty
        assert_eq!(grid.widths(), [0i32; GRID_ROWS]);
        assert_eq!(grid.heights(GRID_ROWS as i32), [0i32; GRID_COLUMNS]);
    }
}
