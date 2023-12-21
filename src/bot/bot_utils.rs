use crate::gamestate::GameState;
use crate::grid::{Grid, GRID_COLUMNS, GRID_ROWS};

fn count_holes(game_state: &GameState) -> i32 {
    unimplemented!()
}

fn count_semi_holes(game_state: &GameState) -> i32 {
    let holes = 0;
    let heights = game_state.grid.heights(GRID_ROWS as i32);
    holes
}

pub fn cost_function(game_state: &GameState) -> f32 {
    let mut cost: f32 = 0.0;
    game_state.grid.widths().into_iter().for_each(|w| match w {
        x if x == GRID_COLUMNS as i32 => cost += 1000.0,
        x if x == GRID_COLUMNS as i32 - 1 => cost += 500.0,
        _ => {}
    });
    cost += count_holes(game_state) as f32 * 500.0;
    cost
}
