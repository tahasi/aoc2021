use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use colored::*;
use structopt::{self, StructOpt};

use super::read_lines;

#[derive(Debug, StructOpt)]
pub struct Command {
    #[structopt(required(true), parse(from_os_str))]
    input: PathBuf,

    #[structopt(long)]
    last: bool,
}

impl Command {
    pub fn run(&self) -> Result<()> {
        let owned_lines = read_lines(&self.input)?;
        let lines = owned_lines
            .iter()
            .map(String::as_str)
            .collect::<Vec<&str>>();

        let drawn_numbers = parse_numbers(lines[0])?;
        let mut boards = parse_boards(&lines[1..])?;

        for number in drawn_numbers {
            for board in boards.iter_mut() {
                board.mark_number(number);
            }

            let winning_boards = boards
                .iter()
                .enumerate()
                .filter(|(_, board)| board.is_winner())
                .map(|(index, _)| index)
                .collect::<Vec<usize>>();
            if !winning_boards.is_empty() {
                if self.last && boards.len() != 1 {
                    winning_boards.iter().rev().for_each(|index| {
                        boards.remove(*index);
                    });
                    continue;
                }
                println!("winning boards:");
                winning_boards.iter().for_each(|index| {
                    let board = &boards[*index];
                    println!("{}", board);
                    let sum_unmarked = board.sum_unmarked_numbers();
                    println!("sum of unmarked numbers is: {}", sum_unmarked);
                    println!("measure: {}", sum_unmarked * number as i32);
                });

                break;
            }
        }

        Ok(())
    }
}

fn parse_numbers(line: &str) -> Result<Vec<u8>> {
    let mut numbers: Vec<u8> = Vec::new();
    for item in line.split(',') {
        numbers.push(item.parse().with_context(|| {
            format!("failed to parse '{}' as a number", item)
        })?);
    }
    Ok(numbers)
}

fn parse_boards(lines: &[&str]) -> Result<Vec<Board>> {
    let mut boards = Vec::new();
    let mut numbers = Vec::new();
    for line in lines {
        let row_numbers: Vec<u8> = line
            .split(' ')
            .filter(|item| !item.is_empty())
            .map(|item| {
                item.parse().with_context(|| {
                    format!("failed to parse board number '{}'", item)
                })
            })
            .collect::<Result<Vec<u8>>>()?;
        match row_numbers.len() {
            0 => continue,
            5 => numbers.push(row_numbers),
            _ => return Err(anyhow!("invalid row number count")),
        }
    }

    if numbers.is_empty() || numbers.len() % 5 != 0 {
        return Err(anyhow!("invalid number or rows"));
    }

    for board_numbers in numbers.chunks(5) {
        let mut grid = [[Cell::default(); 5]; 5];
        for (row_index, row_numbers) in board_numbers.iter().enumerate() {
            for (column_index, column_number) in row_numbers.iter().enumerate()
            {
                grid[row_index][column_index].number = *column_number;
            }
        }
        boards.push(Board { grid });
    }
    Ok(boards)
}

#[derive(Debug)]
struct Board {
    grid: [[Cell; 5]; 5],
}

impl Board {
    pub fn mark_number(&mut self, number: u8) {
        for row in self.grid.iter_mut() {
            for cell in row.iter_mut() {
                if cell.number == number {
                    cell.marked = true;
                }
            }
        }
    }

    pub fn is_winner(&self) -> bool {
        self.grid
            .iter()
            .any(|row| row.iter().all(|cell| cell.marked))
            || self
                .grid
                .iter()
                .fold(vec![0; 5], |mut selected_counts, row| {
                    row.iter().enumerate().for_each(|(index, cell)| {
                        if cell.marked {
                            selected_counts[index] += 1;
                        }
                    });
                    selected_counts
                })
                .iter()
                .any(|count| *count == 5)
    }

    pub fn sum_unmarked_numbers(&self) -> i32 {
        self.grid.iter().fold(0, |sum, row| {
            row.iter().fold(sum, |sum, cell| {
                if cell.marked {
                    sum
                } else {
                    sum + cell.number as i32
                }
            })
        })
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.grid {
            writeln!(
                f,
                "{} {} {} {} {}",
                row[0], row[1], row[2], row[3], row[4],
            )?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Cell {
    number: u8,
    marked: bool,
}

impl Cell {
    #[allow(dead_code)]
    pub fn new(number: u8) -> Cell {
        Cell {
            number,
            marked: false,
        }
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.marked {
            write!(f, "{:>2}", self.number.to_string().bold())
        } else {
            write!(f, "{:>2}", self.number)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Board, Cell};

    #[test]
    fn is_winner_is_true_when_all_cells_of_any_column_are_selected() {
        let mut board = create_board();

        vec![2u8, 7u8, 12u8, 17u8, 22u8]
            .into_iter()
            .for_each(|number| board.mark_number(number));

        assert!(board.is_winner());
    }

    #[test]
    fn is_winner_is_true_when_all_cells_of_any_row_are_selected() {
        let mut board = create_board();

        vec![5u8, 6u8, 7u8, 8u8, 9u8]
            .into_iter()
            .for_each(|number| board.mark_number(number));

        assert!(board.is_winner());
    }

    #[test]
    fn is_winner_is_false_when_no_cell_is_selected() {
        let board = create_board();

        assert!(!board.is_winner());
    }

    fn create_board() -> Board {
        Board {
            grid: [
                [
                    Cell::new(0),
                    Cell::new(1),
                    Cell::new(2),
                    Cell::new(3),
                    Cell::new(4),
                ],
                [
                    Cell::new(5),
                    Cell::new(6),
                    Cell::new(7),
                    Cell::new(8),
                    Cell::new(9),
                ],
                [
                    Cell::new(10),
                    Cell::new(11),
                    Cell::new(12),
                    Cell::new(13),
                    Cell::new(14),
                ],
                [
                    Cell::new(15),
                    Cell::new(16),
                    Cell::new(17),
                    Cell::new(18),
                    Cell::new(19),
                ],
                [
                    Cell::new(20),
                    Cell::new(21),
                    Cell::new(22),
                    Cell::new(23),
                    Cell::new(24),
                ],
            ],
        }
    }
}
