use esp_hal::rng::Rng;
use log::warn;

const NUM_ROWS: usize = 21;
const NUM_COLS: usize = 40;
const GRID_LEN: usize = NUM_ROWS * NUM_COLS;

// Row / column to linear index, wrapping
fn rc_to_i(row: i16, col: i16) -> usize {
    let num_rows = NUM_ROWS as i16;
    let num_cols = NUM_COLS as i16;
    ((row.rem_euclid(num_rows) * num_cols) + col.rem_euclid(num_cols)) as usize
}

#[derive(Debug)] 
pub struct GolCoords {
    pub row: usize,
    pub col: usize,
}

impl GolCoords {
    pub fn new() -> Self {
        Self {
            row: 0,
            col: 0,
        }
    }

    pub fn from_index(index: usize) -> Self {
        Self {
            row: index / NUM_COLS,
            col: index % NUM_COLS,
        }
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
        let row = (index / NUM_COLS) as i16;
        let col = (index % NUM_COLS) as i16;
        self.num_neighbours_cache[ rc_to_i(row-1, col-1) ] += val;
        self.num_neighbours_cache[ rc_to_i(row-1, col  ) ] += val;
        self.num_neighbours_cache[ rc_to_i(row-1, col+1) ] += val;
        self.num_neighbours_cache[ rc_to_i(row  , col-1) ] += val;
        self.num_neighbours_cache[ rc_to_i(row  , col+1) ] += val;
        self.num_neighbours_cache[ rc_to_i(row+1, col-1) ] += val;
        self.num_neighbours_cache[ rc_to_i(row+1, col  ) ] += val;
        self.num_neighbours_cache[ rc_to_i(row+1, col+1) ] += val;
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
            updated: core::array::from_fn(|_| GolCoords::new()),
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

    pub fn alive(&self, row: usize, col: usize) -> bool {
        self.gol_grid.state[rc_to_i(row as i16, col as i16)]
    }

    fn push_updated_cell(&mut self, index: usize) {
        // I don't check for overflow because I like to live dangerously.
        // Also because the array can contain the total number of cells, so it shouldn't happen.
        // This makes things slightly faster.
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

