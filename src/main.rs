use array2d::{Array2D, Error};
use colored::Colorize;
use itertools::Itertools;
use rand;
use rand_derive2::RandGen;

#[derive(RandGen, Debug, Clone, PartialEq)]
enum Color {
    Blue,
    Red,
    Green,
    Yellow,
}

#[derive(Debug, Clone)]
struct Cell {
    color: Color,
}

#[rustfmt::skip]
fn fmt_cell(cell: &Option<Cell>) -> String {
    let symbol = match cell {
        None => " ".normal(),
        Some(Cell { color: Color::Blue }) => "B".blue(),
        Some(Cell { color: Color::Red }) => "R".red(),
        Some(Cell { color: Color::Green }) => "G".green(),
        Some(Cell { color: Color::Yellow }) => "Y".yellow(),
    };
    format!("{} ", symbol)
}

#[derive(Debug)]
struct CellGrid {
    grid: Array2D<Option<Cell>>,
    score: usize,
    debug_info: bool,
}

#[derive(Debug)]
struct Match {
    length: usize,
    color: Color,
    inner_index: usize,
    outer_index: usize,
}

impl Cell {
    fn new_random() -> Option<Self> {
        Some(Self {
            color: rand::random(),
        })
    }

    const fn new_empty() -> Option<Self> {
        None
    }
}

impl CellGrid {
    fn new(length: usize, width: usize, info: bool) -> Self {
        Self {
            grid: Array2D::filled_by_column_major(Cell::new_random, length, width),
            score: 0,
            debug_info: info
        }
    }

    fn resolve_state(&mut self) {
        self.print("init", 300);
        'adding: loop {
            'grav: loop {
                if !self.delete_matches() {
                    break 'grav;
                };
                self.print("find matches", 150);

                if !self.do_gravity() {
                    break 'grav;
                };
            }
            // at this point, there is no gravity to be done, or any matches to make
            if !self.add_element_row() {
                break 'adding;
            };
            self.print("add row", 100);
            self.do_gravity();
        }
        self.print("final", 300);
    }

    fn delete_matches(&mut self) -> bool {
        let mut score = 0;
        let col_matches = self
            .grid
            .columns_iter()
            .enumerate()
            .flat_map(check_line)
            .collect::<Vec<_>>();
        let row_matches = self
            .grid
            .rows_iter()
            .enumerate()
            .flat_map(check_line)
            .collect::<Vec<_>>();

        for col_match in col_matches {
            for x in col_match.inner_index - col_match.length..col_match.inner_index {
                if let Some(element) = self.grid.get_mut(x, col_match.outer_index) {
                    *element = None;
                }
            }
            score += col_match.length;
        }

        for row_match in row_matches {
            for y in row_match.inner_index - row_match.length..row_match.inner_index {
                if let Some(element) = self.grid.get_mut(row_match.outer_index, y) {
                    *element = None;
                }
            }
            score += row_match.length;
        }

        self.score += (score) * (score + 1) / 2;
        return score != 0;
    }

    /*fn fast_do_gravity(&mut self) -> bool {
        // TODO: check if gravity didn't do anything, and return a bool
        for col_index in 0..self.grid.row_len() {
            let elements = self
                .grid
                .column_iter(col_index)
                .unwrap()
                .filter(|element| element.is_some())
                .cloned()
                .collect::<Vec<_>>();
            let num_empty = self.grid.column_len() - elements.len();
            for (row_index, element) in elements.into_iter().rev().enumerate() {
                self.grid
                    .set(self.grid.column_len() - row_index - 1, col_index, element);
            }

            for row_index in 0..num_empty {
                self.grid.set(row_index, col_index, Cell::new_empty());
            }
        }
        true
    }*/

    fn do_gravity(&mut self) -> bool {
        let mut swap_performed = false;
        loop {
            let result = self.do_gravity_step();
            self.print("intermediate grav", 30);
            if !result {
                return swap_performed
            }
            swap_performed = true;
        }
    }

    fn do_gravity_step(&mut self) -> bool {
        let mut swap_performed = false;
        for col_index in 0..self.grid.row_len() {
            for row_index in (1..self.grid.column_len()).rev() {
                // if cell is empty, move element above it downwards
                if let Some(None) = self.grid.get(row_index, col_index) {
                    let mut temp = None;
                    let above = self.grid.get_mut(row_index-1, col_index).expect("row index cannot be zero");
                    std::mem::swap(&mut temp, above);
                    if temp.is_some() {
                        swap_performed = true;
                        self.grid.set(row_index, col_index, temp);
                    }

                }
            }
        }
        swap_performed
    }

    fn add_element_row(&mut self) -> bool {
        let mut exists_arr = vec![];
        let mut result = false;
        for maybe_element in self.grid.row_iter(0).unwrap() {
            let x = maybe_element.is_some();
            if !x {
                result = true
            };
            exists_arr.push(x);
        }

        for (index, exists) in exists_arr.into_iter().enumerate() {
            if !exists {
                self.grid.set(0, index, Cell::new_random());
                //self.grid.set(0, index, Some(Cell { color: Color::Green}));
            }
        }
        result
    }

    fn print(&self, step: &str, time: u64) {
        std::thread::sleep(std::time::Duration::from_millis(time));
        let mut str: String = String::new();
        str.push_str(&format!("score: {}\n", self.score));
        if self.debug_info {str.push_str(&format!("step: {}\n", step))};
        self.grid.rows_iter().for_each(|it| {
            it.for_each(|x| str.push_str(&fmt_cell(x)));
            str.push_str("\n");
        });
        clearscreen::clear();
        print!("{str}");
    }
}

fn check_line<'a, I>((outer_index, line): (usize, I)) -> Vec<Match>
where
    I: Iterator<Item = &'a Option<Cell>>,
{
    let mut prev: Option<&Cell> = None;
    let mut count = 1;
    let mut length = 0;
    let mut matches = vec![];

    for (inner_index, maybe_element) in line.enumerate() {
        length += 1;
        if let Some(element) = maybe_element.as_ref() {
            if let Some(prev_element) = prev {
                if prev_element.color == element.color {
                    count += 1;
                } else {
                    if count >= 3 {
                        matches.push(Match {
                            color: prev_element.color.clone(),
                            length: count,
                            inner_index,
                            outer_index,
                        });
                    }
                    count = 1;
                }
            }
        } else {
            count = 1;
        }
        prev = maybe_element.as_ref();
    }

    if count >= 3 {
        matches.push(Match {
            color: prev.unwrap().color.clone(),
            length: count,
            inner_index: length,
            outer_index,
        });
    }

    matches
}

fn main() {
    println!("Hello, world!");
    let mut grid = CellGrid::new(15, 10, true);
    grid.resolve_state();
}
