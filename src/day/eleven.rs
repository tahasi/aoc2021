use std::{fmt::Debug, path::PathBuf, str::FromStr};

use structopt::{self, StructOpt};

use super::read_all_text;

#[derive(Debug, thiserror::Error)]
#[error("Failed to parse mode from '{0}'")]
pub struct ParseModeError(String);

#[derive(Debug, thiserror::Error)]
#[error("Failed to parse octopus energy level grid from '{0}'")]
pub struct ParseOctopusEnergyLevelGridError(String);

#[derive(Debug, StructOpt)]
pub struct Command {
    #[structopt(required(true), parse(from_os_str))]
    input: PathBuf,

    #[structopt(default_value("10"), long)]
    steps: usize,

    #[structopt(default_value("flashes"), long)]
    mode: Mode,
}

#[derive(Debug, StructOpt)]
pub enum Mode {
    Flashes,
    StepsUntilAllFlash,
}

impl FromStr for Mode {
    type Err = ParseModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "flashes" => Ok(Mode::Flashes),
            "steps-until-all-flash" => Ok(Mode::StepsUntilAllFlash),
            _ => Err(ParseModeError(s.to_owned())),
        }
    }
}

impl Command {
    pub fn run(&self) -> anyhow::Result<()> {
        let mut grid =
            OctopusEnergyLevelGrid::from_str(&read_all_text(&self.input)?)?;

        match self.mode {
            Mode::Flashes => {
                let flashes = (0..self.steps)
                    .fold(0, |flashes, _| flashes + grid.step().flashes());
                println!(
                    "{} flashes occurred after {} steps.",
                    flashes, self.steps
                );
            }
            Mode::StepsUntilAllFlash => {
                let count = grid.width() * grid.length();
                let mut steps = 0;
                loop {
                    steps += 1;
                    let flashes = grid.step().flashes();
                    if flashes == count {
                        break;
                    }
                }
                println!("All octopuses flashed at step {}", steps);
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct OctopusEnergyLevelGrid(Vec<Vec<u8>>);

impl OctopusEnergyLevelGrid {
    fn step(&mut self) -> StepStats {
        self.0
            .iter_mut()
            .flat_map(|row| row.iter_mut())
            .for_each(|cell| *cell += 1);

        let above = |row: usize, col: usize| (row - 1, col);
        let above_left = |row: usize, col: usize| (row - 1, col - 1);
        let above_right = |row: usize, col: usize| (row - 1, col + 1);
        let left = |row: usize, col: usize| (row, col - 1);
        let right = |row: usize, col: usize| (row, col + 1);
        let below = |row: usize, col: usize| (row + 1, col);
        let below_left = |row: usize, col: usize| (row + 1, col - 1);
        let below_right = |row: usize, col: usize| (row + 1, col + 1);

        let max_row = self.length() - 1;
        let max_col = self.width() - 1;
        let mut flashes = 0;
        loop {
            let mut flashed = false;
            for row in 0..=max_row {
                for col in 0..=max_col {
                    if self.0[row][col] >= 10 {
                        flashed = true;
                        flashes += 1;
                        self.0[row][col] = 0;
                        match (row, col) {
                            // top left
                            (0, 0) => self.increment_not_flashed(&[
                                right(0, 0),
                                below_right(0, 0),
                                below(0, 0),
                            ]),
                            // top right
                            (0, col) if col == max_col => self
                                .increment_not_flashed(&[
                                    left(0, col),
                                    below(0, col),
                                    below_left(0, col),
                                ]),
                            // bottom right
                            (row, col) if row == max_row && col == max_col => {
                                self.increment_not_flashed(&[
                                    above_left(row, col),
                                    above(row, col),
                                    left(row, col),
                                ])
                            }
                            // botton left
                            (row, 0) if row == max_row => self
                                .increment_not_flashed(&[
                                    above(row, 0),
                                    above_right(row, 0),
                                    right(row, 0),
                                ]),
                            // top
                            (0, col) => self.increment_not_flashed(&[
                                left(0, col),
                                right(0, col),
                                below_right(0, col),
                                below(0, col),
                                below_left(0, col),
                            ]),
                            // bottom
                            (row, col) if row == max_row => self
                                .increment_not_flashed(&[
                                    left(row, col),
                                    above_left(row, col),
                                    above(row, col),
                                    above_right(row, col),
                                    right(row, col),
                                ]),
                            // right
                            (row, col) if col == max_col => self
                                .increment_not_flashed(&[
                                    left(row, col),
                                    above_left(row, col),
                                    above(row, col),
                                    below(row, col),
                                    below_left(row, col),
                                ]),
                            // left
                            (row, 0) => self.increment_not_flashed(&[
                                above(row, 0),
                                above_right(row, 0),
                                right(row, 0),
                                below_right(row, 0),
                                below(row, 0),
                            ]),
                            // others
                            (row, col) => self.increment_not_flashed(&[
                                left(row, col),
                                above_left(row, col),
                                above(row, col),
                                above_right(row, col),
                                right(row, col),
                                below_right(row, col),
                                below(row, col),
                                below_left(row, col),
                            ]),
                        }
                    }
                }
            }
            if !flashed {
                break;
            }
        }
        StepStats { flashes }
    }

    fn width(&self) -> usize {
        self.0.len()
    }

    fn length(&self) -> usize {
        self.0[0].len()
    }

    fn increment_not_flashed(&mut self, cells: &[(usize, usize)]) {
        for (row, col) in cells.iter().copied() {
            if self.0[row][col] != 0 {
                self.0[row][col] += 1;
            }
        }
    }
}

impl FromStr for OctopusEnergyLevelGrid {
    type Err = ParseOctopusEnergyLevelGridError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut grid = vec![];
        let mut prior_len = None;
        for line in s.split('\n') {
            if line.is_empty() {
                if prior_len.is_none() {
                    continue;
                } else {
                    break;
                }
            }
            prior_len = match prior_len {
                Some(prior_len) if line.len() != prior_len => {
                    return Err(ParseOctopusEnergyLevelGridError(s.to_owned()))
                }
                None => Some(line.len()),
                _ => prior_len,
            };
            let row_levels = line
                .as_bytes()
                .iter()
                .copied()
                .map(|b| match b {
                    b @ b'0'..=b'9' => Ok(b - b'0'),
                    _ => Err(ParseOctopusEnergyLevelGridError(s.to_owned())),
                })
                .collect::<Result<Vec<u8>, Self::Err>>()?;
            grid.push(row_levels);
        }

        Ok(OctopusEnergyLevelGrid(grid))
    }
}

struct StepStats {
    flashes: usize,
}

impl StepStats {
    fn flashes(&self) -> usize {
        self.flashes
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::OctopusEnergyLevelGrid;

    #[test]
    fn octopus_energy_level_grid_from_str_test() {
        let grid =
            OctopusEnergyLevelGrid::from_str(INPUT).expect("valid input");

        assert_eq!(grid.width(), 10);
        assert_eq!(grid.length(), 10);
    }

    #[test]
    fn octopus_energy_level_grid_step_test() {
        let mut grid =
            OctopusEnergyLevelGrid::from_str(INPUT).expect("valid input");

        let stats = grid.step();
        assert_eq!(stats.flashes(), 0);

        let stats = grid.step();
        assert_eq!(stats.flashes(), 35);

        let stats = grid.step();
        assert_eq!(stats.flashes(), 45);

        let stats = grid.step();
        assert_eq!(stats.flashes(), 16);

        let stats = grid.step();
        assert_eq!(stats.flashes(), 8);

        let stats = grid.step();
        assert_eq!(stats.flashes(), 1);

        let stats = grid.step();
        assert_eq!(stats.flashes(), 7);

        let stats = grid.step();
        assert_eq!(stats.flashes(), 24);

        let stats = grid.step();
        assert_eq!(stats.flashes(), 39);

        let stats = grid.step();
        assert_eq!(stats.flashes(), 29);
    }

    #[test]
    fn octopus_energy_level_grid_step_100_test() {
        let mut grid =
            OctopusEnergyLevelGrid::from_str(INPUT).expect("valid input");

        let flashes =
            (0..100).fold(0, |flashes, _| flashes + grid.step().flashes());

        assert_eq!(flashes, 1656);
    }

    #[test]
    fn octopus_energy_level_grid_step_until_all_flash_test() {
        let mut grid =
            OctopusEnergyLevelGrid::from_str(INPUT).expect("valid input");
        let count = grid.width() * grid.length();

        let mut step = 0;
        loop {
            step += 1;
            let flashes = grid.step().flashes();
            if flashes == count {
                break;
            }
        }

        assert_eq!(step, 195);
    }

    const INPUT: &str = r"5483143223
2745854711
5264556173
6141336146
6357385478
4167524645
2176841721
6882881134
4846848554
5283751526";
}
