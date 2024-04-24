use array2d::{Array2D, Error};
use rand;
use rand_derive2::RandGen;
use colored::Colorize;
use itertools::Itertools;

#[derive(RandGen, Debug, Clone, PartialEq)]
enum Color {
    Blue,
    Red,
    Green,
}

#[derive(Debug, Clone)]
struct Cell {
    color: Color, 
}

fn fmt_cell(cell: &Option<Cell>) -> String {
    let symbol = match cell {
        None => " ".normal(),
        Some(Cell {color:Color::Blue}) => "B".blue(),
        Some(Cell {color:Color::Red}) => "R".red(),
        Some(Cell {color:Color::Green}) => "G".green(),
    };
    format!("{} ", symbol)
}

#[derive(Debug)]
struct CellGrid {
    grid: Array2D<Option<Cell>>,
    score: usize,
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

    fn new_empty() -> Option<Self> {
        None
    }
}

impl CellGrid {
    fn new(length: usize, width: usize) -> Self {
        Self {
            grid: Array2D::filled_by_column_major(Cell::new_random, length, width),
            score: 0,
        }
    }

    fn resolve_state(&mut self) {
        self.print();
        self.score += self.delete_matches();
        self.print();
        self.do_gravity();
        self.print();
    }

    fn delete_matches(self: &mut Self) -> usize {
        let mut score = 0;
        let col_matches = self.grid.columns_iter().enumerate().flat_map(check_line).collect::<Vec<_>>();
        let row_matches = self.grid.rows_iter().enumerate().flat_map(check_line).collect::<Vec<_>>();

        for col_match in col_matches {
            println!("col {col_match:?}");
            for x in col_match.inner_index-col_match.length..col_match.inner_index {
                if let Some(element) = self.grid.get_mut(x, col_match.outer_index) {
                    *element = None;
                }
            }
            score += col_match.length;
        }

        for row_match in row_matches {
            println!("row {row_match:?}");
            for y in row_match.inner_index-row_match.length..row_match.inner_index {
                if let Some(element) = self.grid.get_mut(row_match.outer_index, y) {
                    *element = None;
                }
            }
            score += row_match.length;
        }

        (score)*(score+1)/2
    }

    fn do_gravity(&mut self) {
        for col_index in 0..self.grid.row_len() {
            let elements = self.grid.column_iter(col_index).unwrap().filter(|element| element.is_some()).map(|element| element.clone()).collect::<Vec<_>>();
            let num_empty = self.grid.column_len() - elements.len();
            for (row_index, element) in elements.into_iter().rev().enumerate() {
                self.grid.set(self.grid.column_len() - row_index - 1, col_index, element);
            }

            for row_index in 0..num_empty {
                self.grid.set(row_index, col_index, Cell::new_empty());
            }

        };
    } 

    fn print(&self) {
        println!();
        println!("score: {}", self.score);
        self.grid.rows_iter().for_each(|it| {
            let mut str: String = String::new();
            it.for_each(|x| str.push_str(&fmt_cell(x)));
            println!("{str}");
        })
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
        let element = maybe_element.as_ref().unwrap();
        if let Some(prev_element) = prev {
            if prev_element.color == element.color { // TODO no unwrap here
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
        prev = Some(element);
    }

    if count >= 3 {
        matches.push(Match {
            color: prev.unwrap().color.clone(),
            length: count,
            inner_index: length,
            outer_index
        });
    }

    matches
}

fn main() {
    println!("Hello, world!");
    let mut grid = CellGrid::new(5, 5);
    grid.resolve_state();
}
