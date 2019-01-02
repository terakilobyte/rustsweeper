use ggez::graphics::Point2;
use rand::prelude::*;
use std::collections::VecDeque;

use crate::cell::Cell;
use std::fmt;
#[derive(Debug, Clone)]
pub struct Board {
    pub cells: Vec<Vec<Cell>>,
    pub difficulty: (usize, usize),
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display = self
            .cells
            .iter()
            .map(|o| {
                o.iter()
                    .map(|i| format!("{}", i.rust_count))
                    .collect::<Vec<String>>()
                    .join(" ")
            })
            .collect::<Vec<String>>()
            .join("\n");
        write!(f, "{}", display)
    }
}

impl Board {
    pub fn new(difficulty: (usize, usize)) -> Self {
        let mut rng = rand::thread_rng();
        let mut cells = vec![];
        let (_, cells_row) = difficulty;

        let mut starting_states: Vec<bool> = (0..=(cells_row.pow(2))).map(|n| n % 9 == 0).collect();

        starting_states.shuffle(&mut rng);
        for i in 0..cells_row {
            let mut inner = vec![];
            for j in 0..cells_row {
                let cell = Cell::new(
                    Point2::new(i as f32, j as f32),
                    starting_states[i * cells_row + j],
                );
                inner.push(cell);
            }
            cells.push(inner);
        }
        Board { cells, difficulty }
    }

    pub fn calculate_rust_count(&mut self) {
        let (_, cells) = self.difficulty;
        for i in 0..cells {
            for j in 0..cells {
                let mut rusts_found = 0;
                'k: for k in -1..=1 {
                    'l: for l in -1..=1 {
                        if (i == 0 && k == -1) || (i == cells - 1 && k == 1) {
                            continue 'k;
                        }
                        if (j == 0 && l == -1) || (j == cells - 1 && l == 1) {
                            continue 'l;
                        }
                        if k != 0 || l != 0 {
                            if self.cells[(i as i32 + k) as usize][(j as i32 + l) as usize].is_rust
                            {
                                rusts_found += 1;
                            }
                        }
                    }
                }
                self.cells[i][j].rust_count = rusts_found;
            }
        }
    }

    // if ignore_rules is true (in the event a bomb was clicked on), flood fill
    // actually performs a flood fill. Otherwise this implements standard
    // minesweeper rules
    pub fn flood_fill(&mut self, cell_x: usize, cell_y: usize) {
        let mut queue: VecDeque<Cell> = VecDeque::new();
        queue.push_front(self.cells[cell_x][cell_y].clone());

        while let Some(cell) = queue.pop_front() {
            if !cell.is_rust && !cell.is_flagged {
                let (x, y) = (cell.position.x as usize, cell.position.y as usize);
                self.cells[x][y].is_hidden = false;
                'outer: for i in -1..=1 {
                    for j in -1..=1 {
                        if cell.position.x == 0.0 && i == -1
                            || cell.position.x == self.difficulty.1 as f32 - 1.0 && i == 1
                        {
                            continue 'outer;
                        }
                        if cell.position.y == 0.0 && j == -1
                            || cell.position.y == self.difficulty.1 as f32 - 1.0 && j == 1
                        {
                            continue;
                        }
                        if i != 0 || j != 0 {
                            if !self.cells[(cell.position.x as i8 + i) as usize]
                                [(cell.position.y as i8 + j) as usize]
                                .is_flagged
                                && self.cells[(cell.position.x as i8 + i) as usize]
                                    [(cell.position.y as i8 + j) as usize]
                                    .rust_count
                                    == 0
                                && self.cells[(cell.position.x as i8 + i) as usize]
                                    [(cell.position.y as i8 + j) as usize]
                                    .is_hidden
                            {
                                let new_cell = self.cells[(cell.position.x as i8 + i) as usize]
                                    [(cell.position.y as i8 + j) as usize]
                                    .clone();
                                queue.push_back(new_cell);
                            } else if !self.cells[(cell.position.x as i8 + i) as usize]
                                [(cell.position.y as i8 + j) as usize]
                                .is_flagged
                                && !self.cells[(cell.position.x as i8 + i) as usize]
                                    [(cell.position.y as i8 + j) as usize]
                                    .is_rust
                            {
                                self.cells[(cell.position.x as i8 + i) as usize]
                                    [(cell.position.y as i8 + j) as usize]
                                    .is_hidden = false;
                            }
                        }
                    }
                }
            } else if cell.is_rust {
                self.cells[cell_x][cell_y].game_over = true;
                self.cells[cell_x][cell_y].is_hidden = false;
            }
        }
    }
}
