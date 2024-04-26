use array2d::{Array2D, Error};
use colored::Colorize;
use itertools::Itertools;
use rand;
use rand_derive2::RandGen;
use std::io::{self, Write};

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
    deleting: bool,
}

#[rustfmt::skip]
fn fmt_cell(cell: &Option<Cell>) -> String {
    let symbol = match cell {
        None => " ".normal(),
        Some(Cell { deleting: true, color: Color::Blue}) => "\u{2588}".blue(),
        Some(Cell { deleting: true, color: Color::Red}) => "\u{2588}".red(),
        Some(Cell { deleting: true, color: Color::Green}) => "\u{2588}".green(),
        Some(Cell { deleting: true, color: Color::Yellow}) => "\u{2588}".yellow(),

        Some(Cell { color: Color::Blue , ..}) => "B".blue(),
        Some(Cell { color: Color::Red, .. }) => "R".red(),
        Some(Cell { color: Color::Green, .. }) => "G".green(),
        Some(Cell { color: Color::Yellow, .. }) => "Y".yellow(),
    };
    format!("{} ", symbol)
}

#[derive(Debug)]
struct CellGrid {
    grid: Array2D<Option<Cell>>,
    score: usize,

    debug_info: bool,
    game_speed: u64,
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
            deleting: false,
        })
    }

    const fn new_empty() -> Option<Self> {
        None
    }
}

impl CellGrid {

    fn filled_grid_random(length:usize, width: usize) -> Array2D<Option<Cell>> {
        Array2D::filled_by_column_major(Cell::new_random, length, width)
    }

    fn new(length: usize, width: usize, debug_info: bool, game_speed: u64) -> Self {
        Self {
            grid: CellGrid::filled_grid_random(length, width),
            score: 0,
            debug_info,
            game_speed,
        }
    }

    fn play_game(&mut self) {
        loop {
            self.resolve_state();
            self.make_move();
        }
    }

    fn screensaver(&mut self) {
        loop {
            self.resolve_state();
            self.grid = Self::filled_grid_random(self.grid.row_len(), self.grid.column_len());
        }
    }

    fn make_move(&mut self) {
        loop {
            println!("Please enter the two elements you would like to swap");
            println!("(In the form 'x1 y1 x2 y2', eg '0 1 0 0': ");
            let (row1, col1, row2, col2): (i64, i64, i64, i64);
            text_io::scan!("{} {} {} {}", row1, col1, row2, col2);
            println!("{}, {}, {}, {}, {}, {}", row1, col1, row2, col2, (row1 - row2).abs(), (col1 - col2).abs());
            if (row1 == row2 && (col1 - col2).abs() == 1)
                || (col1 == col2 && (row1 - row2).abs() == 1)
            {
                let mut temp = None;
                let first = self.grid.get_mut(row1.try_into().unwrap(), col1.try_into().unwrap()).unwrap();
                std::mem::swap(&mut temp, first);
                let second = self
                    .grid
                    .get_mut(row2.try_into().unwrap(), col2.try_into().unwrap()).unwrap();
                std::mem::swap(&mut temp, second);
                self.grid.set(row1.try_into().unwrap(), col1.try_into().unwrap(), temp);
                break;
            }
        }
    }

    fn resolve_state(&mut self) {
        self.print("init", 10);
        'main: loop {
            'clear_matches: loop {
                if !self.delete_matches(true) {
                    break 'clear_matches;
                };

                if !self.do_gravity(true) {
                    break 'clear_matches;
                };
            }
            // at this point, there is no gravity to be done, or any matches to make
            // so if nothing can be added, then end
            if !self.add_element_row(true) {
                break 'main;
            };
            self.do_gravity_step(true);
            'adding: loop {
                if !self.add_element_row(true) {
                    break 'adding;
                };
                self.do_gravity_step(false);
            }
        }
        self.print("final", 0);
    }

    fn delete_matches(&mut self, print: bool) -> bool {
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
                    element.as_mut().unwrap().deleting = true;
                }
            }
            score += col_match.length;
        }

        for row_match in row_matches {
            for y in row_match.inner_index - row_match.length..row_match.inner_index {
                if let Some(element) = self.grid.get_mut(row_match.outer_index, y) {
                    element.as_mut().unwrap().deleting = true;
                }
            }
            score += row_match.length;
        }

        if print { self.print("matches found", 10) };

        for row in 0..self.grid.column_len() {
            for col in 0..self.grid.row_len() {
                if let Some(maybe_element) = self.grid.get_mut(row, col) {
                    if let Some(element) = maybe_element {
                        if element.deleting == true {
                            *maybe_element = None;
                        }
                    }
                };
            }
        }

        if print { self.print("matches deleted", 10) };

        self.score += (score) * (score + 1) / 2;
        return score != 0;
    }

    fn do_gravity(&mut self, print: bool) -> bool {
        let mut swap_performed = false;
        loop {
            if !self.do_gravity_step(print) {
                return swap_performed;
            }
            swap_performed = true;
        }
    }

    fn do_gravity_step(&mut self, print: bool) -> bool {
        let mut swap_performed = false;
        for col_index in 0..self.grid.row_len() {
            for row_index in (1..self.grid.column_len()).rev() {
                // if cell is empty, move element above it downwards
                if let Some(None) = self.grid.get(row_index, col_index) {
                    let mut temp = None;
                    let above = self
                        .grid
                        .get_mut(row_index - 1, col_index)
                        .expect("row index cannot be zero");
                    std::mem::swap(&mut temp, above);
                    if temp.is_some() {
                        swap_performed = true;
                        self.grid.set(row_index, col_index, temp);
                    }
                }
            }
        }
        if print { self.print("gravity step", 3) };
        swap_performed
    }

    fn add_element_row(&mut self, print: bool) -> bool {
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
        if print { self.print("add row", 5) };
        result
    }

    fn print(&self, step: &str, time: u64) {
        let stdout = io::stdout();
        let mut handle = io::BufWriter::new(stdout.lock());
        std::thread::sleep(std::time::Duration::from_millis(time * self.game_speed));
        let mut str: String = String::new();
        str.push_str(&format!("score: {}\n", self.score));
        if self.debug_info {
            str.push_str(&format!("step: {}\n", step))
        };

        //TODO: fix column names vertically when >9
        //let height = (self.grid.row_len().ilog10() + 1) as usize;

        str.push_str("\n  ");
        for i in 0..self.grid.row_len() {
            str.push_str(&format!("{} ", i))
        }
        str.push_str("\n");

        let len = (self.grid.column_len().ilog10() + 1) as usize;
        self.grid
            .rows_iter()
            .enumerate()
            .for_each(|(row_index, row)| {
                str.push_str(&format!("{:0width$} ", row_index, width = len));
                row.for_each(|x| str.push_str(&fmt_cell(x)));
                str.push_str("\n");
            });
        //clearscreen::clear();
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        std::thread::sleep(std::time::Duration::from_millis(120));
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
    let mut grid = CellGrid::new(300, 300, true, 10);
    //grid.play_game();
    grid.screensaver();
}
