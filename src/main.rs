use array2d::{Array2D, Error};
use colored::Colorize;
use rand;
use rand_derive2::RandGen;
use std::io::{self, Write};

#[derive(RandGen, Debug, Clone, PartialEq, Copy)]
enum Color {
    Blue,
    Red,
    Green,
    Yellow,
}

#[derive(Debug, Clone)]
enum Cell {
    Normal(Color),
    Deleting(Color),
    Empty,
}

#[derive(Debug)]
struct CellGrid {
    grid: Array2D<Cell>,
    score: usize,

    debug_info: bool,
    reset_cursor: bool,
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
    fn new_random() -> Self {
        Self::Normal(rand::random())
    }

    fn to_deleting(&mut self) {
        *self = match self {
            Self::Normal(color) | Self::Deleting(color) => Self::Deleting(*color),
            Self::Empty => unreachable!(),
        }
    }

    const fn is_some(&self) -> bool {
        match self {
            Self::Normal(..) => true,
            Self::Empty | Self::Deleting(..) => false,
        }
    }

    //#[rustfmt::skip]
    fn fmt_cell(&self) -> String {
        let symbol = match self {
            Self::Empty => " ".normal(),
            Self::Deleting(Color::Blue) => "\u{2588}".blue(),
            Self::Deleting(Color::Red) => "\u{2588}".red(),
            Self::Deleting(Color::Green) => "\u{2588}".green(),
            Self::Deleting(Color::Yellow) => "\u{2588}".yellow(),

            Self::Normal(Color::Blue) => "B".blue(),
            Self::Normal(Color::Red) => "R".red(),
            Self::Normal(Color::Green) => "G".green(),
            Self::Normal(Color::Yellow) => "Y".yellow(),
        };
        format!("{symbol} ")
    }
}

impl CellGrid {
    fn filled_grid_random(height: usize, width: usize) -> Array2D<Cell> {
        Array2D::filled_by_column_major(Cell::new_random, height, width)
    }

    fn new(
        height: usize,
        width: usize,
        debug_info: bool,
        reset_cursor: bool,
        game_speed: u64,
    ) -> Self {
        Self {
            grid: Self::filled_grid_random(height, width),
            score: 0,
            debug_info,
            reset_cursor,
            game_speed,
        }
    }

    fn user_play_game(&mut self) {
        loop {
            self.resolve_state(true);
            self.make_move();
        }
    }

    fn screensaver(&mut self) {
        loop {
            self.resolve_state(true);
            self.grid = Self::filled_grid_random(self.grid.column_len(), self.grid.row_len());
        }
    }

    fn faster_screensaver(&mut self) {
        loop {
            self.delete_matches(true);
            self.print("nyoom", 25);
            self.do_gravity(true);
            self.grid = Self::filled_grid_random(self.grid.column_len(), self.grid.row_len());
        }
    }

    fn auto_play_game(&mut self) {
        loop {
            let mut best_score: usize = 0;
            let mut best_move: (usize, usize, usize, usize) = (0, 0, 0, 0);
            for col_index in 0..self.grid.row_len() {
                for row_index in 0..self.grid.column_len() {
                    for x in [
                        (col_index, row_index, col_index.saturating_add(1), row_index),
                        (col_index, row_index, col_index.saturating_sub(1), row_index),
                        (col_index, row_index, col_index, row_index.saturating_add(1)),
                        (col_index, row_index, col_index, row_index.saturating_sub(1)),
                    ] {
                        self.swap(x.0, x.1, x.2, x.3);
                        let count = self.count_matches();
                        if count > best_score {
                            best_score = count;
                            best_move = x;
                        }
                        self.swap(x.0, x.1, x.2, x.3);
                    }
                }
            }
            let x = best_move;
            self.swap(x.0, x.1, x.2, x.3);
            self.resolve_state(true);
        }
    }

    fn make_move(&mut self) {
        loop {
            println!("Please enter the two elements you would like to swap");
            println!("(In the form 'x1 y1 x2 y2', eg '0 1 0 0': ");
            let (row1, col1, row2, col2): (i64, i64, i64, i64);
            text_io::scan!("{} {} {} {}", row1, col1, row2, col2);
            println!(
                "{}, {}, {}, {}, {}, {}",
                row1,
                col1,
                row2,
                col2,
                (row1 - row2).abs(),
                (col1 - col2).abs()
            );
            if (row1 == row2 && (col1 - col2).abs() == 1)
                || (col1 == col2 && (row1 - row2).abs() == 1)
            {
                self.swap(
                    row1.try_into().unwrap(),
                    col1.try_into().unwrap(),
                    row2.try_into().unwrap(),
                    col2.try_into().unwrap(),
                );
                break;
            }
        }
    }

    fn swap(&mut self, row1: usize, col1: usize, row2: usize, col2: usize) {
        let mut temp = Cell::Empty;
        if row1 == row2 && col1 == col2 {
            return;
        }
        // first contains A, second contains B, temp contains _
        // swap first <-> temp, so first: _ and temp: A
        // swap second <-> temp, so second: A and temp: B
        // (if this isn't possible, put temp back into first)
        // put temp into first, so first: B, second: A
        if let Some(first) = self.grid.get_mut(row1, col1) {
            std::mem::swap(&mut temp, first);
            if let Some(second) = self.grid.get_mut(row2, col2) {
                std::mem::swap(&mut temp, second);
            }
            let _ = self.grid.set(row1, col1, temp);
        }
    }

    fn resolve_state(&mut self, print: bool) {
        self.print("init", 10);
        'main: loop {
            'clear_matches: loop {
                if !self.delete_matches(print) {
                    break 'clear_matches;
                };

                if !self.do_gravity(print) {
                    break 'clear_matches;
                };
            }
            // at this point, there is no gravity to be done, or any matches to make
            // so if nothing can be added, then end
            if !self.add_element_row(print) {
                break 'main;
            };
            self.do_gravity_step(false);
            'adding: loop {
                if !self.add_element_row(print) {
                    break 'adding;
                };
                self.do_gravity_step(false);
            }
        }
        self.print("final", 0);
    }

    fn count_matches(&mut self) -> usize {
        self.grid
            .columns_iter()
            .enumerate()
            .flat_map(check_line)
            .count()
            + self
                .grid
                .rows_iter()
                .enumerate()
                .flat_map(check_line)
                .count()
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
                    element.to_deleting();
                }
            }
            score += col_match.length;
        }

        for row_match in row_matches {
            for y in row_match.inner_index - row_match.length..row_match.inner_index {
                if let Some(element) = self.grid.get_mut(row_match.outer_index, y) {
                    element.to_deleting();
                }
            }
            score += row_match.length;
        }

        if print {
            self.print("matches found", 10);
        };

        for row in 0..self.grid.column_len() {
            for col in 0..self.grid.row_len() {
                if let Some(maybe_element) = self.grid.get_mut(row, col) {
                    if let Cell::Deleting(_) = maybe_element {
                        *maybe_element = Cell::Empty;
                    }
                };
            }
        }

        if print {
            self.print("matches deleted", 10);
        };

        self.score += (score) * (score + 1) / 2;
        score != 0
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
                if matches!(self.grid.get(row_index, col_index), Some(Cell::Empty)) {
                    let mut temp = Cell::Empty;
                    let above = self
                        .grid
                        .get_mut(row_index - 1, col_index)
                        .expect("row index cannot be zero");
                    std::mem::swap(&mut temp, above);
                    if temp.is_some() {
                        swap_performed = true;
                        let _ = self.grid.set(row_index, col_index, temp);
                    }
                }
            }
        }
        if print {
            self.print("gravity step", 3);
        };
        swap_performed
    }

    fn add_element_row(&mut self, print: bool) -> bool {
        let mut exists_arr = vec![];
        let mut result = false;
        for element in self.grid.row_iter(0).unwrap() {
            let x = element.is_some();
            if !x {
                result = true;
            };
            exists_arr.push(x);
        }

        for (index, exists) in exists_arr.into_iter().enumerate() {
            if !exists {
                let _ = self.grid.set(0, index, Cell::new_random());
                //self.grid.set(0, index, Some(Cell { color: Color::Green}));
            }
        }
        if print {
            self.print("add row", 5);
        };
        result
    }

    fn print(&self, info: &str, time: u64) {
        let _ = self.print_err(info, time);
    }

    fn print_err(&self, info: &str, time: u64) -> std::io::Result<()> {
        let stdout = io::stdout();
        let buf_size = (self.grid.row_len() + 3) * (self.grid.column_len() + 3) * 2;
        let mut handle = termion::cursor::HideCursor::from(io::BufWriter::with_capacity(
            buf_size,
            stdout.lock(),
        ));
        std::thread::sleep(std::time::Duration::from_millis(time * self.game_speed));
        //write!(handle, "{esc}c", esc = 27 as char); // clear
        if self.reset_cursor {
            writeln!(handle, "{}", termion::cursor::Goto(1, 1))?;
        };
        writeln!(handle, "score: {}\n", self.score)?;
        if self.debug_info {
            writeln!(handle, "step: {info}         \n")?;
        };

        //TODO: fix column names vertically when >9
        //let height = (self.grid.row_len().ilog10() + 1) as usize;

        write!(handle, "\n  ")?;
        for i in 0..self.grid.row_len() {
            write!(handle, "{i} ")?;
        }
        writeln!(handle)?;

        let len = (self.grid.column_len().ilog10() + 1) as usize;
        for (row_index, row) in self.grid.rows_iter().enumerate() {
            write!(handle, "{row_index:0len$} ")?;
            for x in row {
                write!(handle, "{}", Cell::fmt_cell(x))?;
            }
            writeln!(handle)?;
            //std::thread::sleep(std::time::Duration::from_millis(1));
        }
        handle.flush()?;
        Ok(())
    }
}

fn check_line<'a, I>((outer_index, line): (usize, I)) -> Vec<Match>
where
    I: Iterator<Item = &'a Cell>,
{
    let mut maybe_prev: Option<&Cell> = None;
    let mut count = 1;
    let mut length = 0;
    let mut matches = vec![];

    for (inner_index, element) in line.enumerate() {
        length += 1;
        if let Cell::Normal(color) = element {
            if let Some(Cell::Normal(prev_color)) = maybe_prev {
                if prev_color == color {
                    count += 1;
                } else {
                    if count >= 3 {
                        matches.push(Match {
                            color: *prev_color,
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
        maybe_prev = Some(element);
    }

    if count >= 3 {
        if let Some(Cell::Normal(color)) = maybe_prev {
            matches.push(Match {
                color: *color,
                length: count,
                inner_index: length,
                outer_index,
            });
        }
    }

    matches
}

fn main() {
    println!("Hello, world!");
    let _ = clearscreen::clear();
    let mut game = CellGrid::new(28, 18, true, true, 2);
    let choice = text_io::read!();
    match choice {
        0 => game.auto_play_game(), //TODO: search more than just next step for better moves
        1 => game.user_play_game(),
        2 => game.screensaver(),
        3 => game.faster_screensaver(),
        _ => todo!(),
    }
}
