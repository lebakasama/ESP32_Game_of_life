use esp_hal::rng::Rng;
use log::warn;

const NUM_ROWS: usize = 21;
const NUM_COLS: usize = 40;
const GRID_LEN: usize = NUM_ROWS * NUM_COLS;

// Row / column to linear index, wrapping
fn rc_to_i(row: usize, col: usize) -> usize {
    ((row % NUM_ROWS) * NUM_COLS) + (col % NUM_COLS)
}

fn i_to_rc(index: usize) -> (usize, usize) {
    (index / NUM_COLS, index % NUM_COLS)
}

#[derive(Debug)] 
pub struct GolCoords {
    pub row: i32,
    pub col: i32,
}

impl GolCoords {
    pub fn new(row: i32, col: i32) -> Self {
        Self {
            row,
            col,
        }
    }

    pub fn from_index(index: usize) -> Self {
        Self {
            row: (index / NUM_COLS) as i32,
            col: (index % NUM_COLS) as i32,
        }
    }

    pub fn to_index(&self) -> usize {
        ((self.row * NUM_COLS as i32) + self.col) as usize
    }
}

struct GolGrid {
    state: [bool; GRID_LEN],
    num_neighbours: [i8; GRID_LEN],
    num_neighbours_cache: [i8; GRID_LEN],
}

impl GolGrid {
    pub fn new_random(rng: &Rng) -> Self {
        let state: [bool; GRID_LEN] =
            core::array::from_fn(|_| rng.random().is_multiple_of(3));
        
        let mut new_obj = Self {
            state: state.clone(),
            num_neighbours: [0_i8; GRID_LEN],
            num_neighbours_cache: [0_i8; GRID_LEN],
        };

        new_obj.init_neighbours();

        new_obj
    }

    fn init_neighbours(&mut self) {
        for i in 0..GRID_LEN {
            if self.state[i] {
                self.add_to_neighbours(i, 1);
            }
        }

        self.num_neighbours = self.num_neighbours_cache;
    }

    fn add_to_neighbours(&mut self, index: usize, val: i8) {
        let row = index / NUM_COLS;
        let col = index % NUM_COLS;
        let prev_row = if row == 0 {NUM_ROWS - 1} else {row - 1};
        let next_row = (row + 1) % NUM_ROWS;
        let prev_col = if col == 0 {NUM_COLS - 1} else {col - 1};
        let next_col = (col + 1) % NUM_COLS;

        self.num_neighbours_cache[ rc_to_i(prev_row, prev_col) ] += val;
        self.num_neighbours_cache[ rc_to_i(prev_row, col     ) ] += val;
        self.num_neighbours_cache[ rc_to_i(prev_row, next_col) ] += val;
        self.num_neighbours_cache[ rc_to_i(row  , prev_col) ] += val;
        self.num_neighbours_cache[ rc_to_i(row  , next_col) ] += val;
        self.num_neighbours_cache[ rc_to_i(next_row, prev_col) ] += val;
        self.num_neighbours_cache[ rc_to_i(next_row, col  ) ] += val;
        self.num_neighbours_cache[ rc_to_i(next_row, next_col) ] += val;
    }

    pub fn kill(&mut self, i: usize) {
        if self.state[i] == true {
            self.state[i] = false;
            self.add_to_neighbours(i, -1);
        }
    }

    pub fn spawn(&mut self, i: usize) {
        if self.state[i] == false {
            self.state[i] = true;
            self.add_to_neighbours(i, 1);
        }
    }

    pub fn swap_cache(&mut self) {
        self.num_neighbours = self.num_neighbours_cache;
    }
}

pub struct GameOfLife {
    gol_grid: GolGrid,
    //updated: Vec<GolCoords, GRID_LEN>,
    updated: [GolCoords; GRID_LEN],
    num_updated: usize,
}

impl GameOfLife {
    pub fn new() -> Self {
        let mut new_obj = Self {
            gol_grid: GolGrid::new_random(&Rng::new()),
            updated: core::array::from_fn(|_| GolCoords::new(0, 0)),
            num_updated: 0,
        };

        for i in 0..GRID_LEN {
            if new_obj.gol_grid.state[i] {
                new_obj.updated[new_obj.num_updated] = GolCoords::from_index(i);
            }
        }

        new_obj
    }

    pub fn updated(&self) -> &[GolCoords] {
        &self.updated[0..self.num_updated]
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (NUM_ROWS, NUM_COLS)
    }

    pub fn alive(&self, coords: &GolCoords) -> bool {
        self.gol_grid.state[coords.to_index()]
    }

    fn push_updated_cell(&mut self, index: usize) {
        // I don't check for overflow because I like to live dangerously.
        // Also because the array can contain the total number of cells, so it shouldn't happen.
        self.updated[self.num_updated] = GolCoords::from_index(index);
        self.num_updated += 1;
    }

    fn reset_updated_cells(&mut self) {
        self.num_updated = 0;
    }

    pub fn update(&mut self) {
        self.reset_updated_cells();

        for i in 0..GRID_LEN {
            if self.gol_grid.state[i] {
                // Cell is alive
                if self.gol_grid.num_neighbours[i] < 2 || self.gol_grid.num_neighbours[i] > 3 {
                    self.gol_grid.kill(i); 
                    self.push_updated_cell(i);
                }
            } else {
                // Cell is dead
                if self.gol_grid.num_neighbours[i] == 3 {
                    self.gol_grid.spawn(i);
                    self.push_updated_cell(i);
                }
            }

            if self.gol_grid.num_neighbours_cache[i] < 0
                    || self.gol_grid.num_neighbours_cache[i] > 8 {
                warn!("Bad number of num_neighbours: {}", self.gol_grid.num_neighbours_cache[i]);
                self.gol_grid.num_neighbours_cache[i] = 0;
            }
        }

        self.gol_grid.swap_cache();
    }
}

