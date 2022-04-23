use std::{
    cmp::Reverse, collections::HashMap, path::PathBuf, result, str::FromStr,
};

use structopt::{self, StructOpt};
use thiserror;

use super::read_lines;

#[derive(Debug, thiserror::Error)]
#[error("Failed to parse mode from '{0}'")]
struct ParseModeError(String);

#[derive(Debug, StructOpt)]
enum Mode {
    RiskLevel,
    Basins,
}

impl FromStr for Mode {
    type Err = ParseModeError;
    fn from_str(mode: &str) -> result::Result<Self, Self::Err> {
        match mode {
            "risk-level" => Ok(Mode::RiskLevel),
            "basins" => Ok(Mode::Basins),
            _ => Err(ParseModeError(mode.to_owned())),
        }
    }
}

#[derive(Debug, StructOpt)]
pub struct Command {
    #[structopt(required(true), parse(from_os_str))]
    input: PathBuf,

    #[structopt(default_value("risk-level"), long)]
    mode: Mode,
}

impl Command {
    pub fn run(&self) -> anyhow::Result<()> {
        let lines = read_lines(&self.input)?;
        let map = HeightMap::parse(lines.iter().map(String::as_ref))?;

        match self.mode {
            Mode::Basins => report_basins(&map),
            Mode::RiskLevel => report_risk_levels(&map),
        }

        Ok(())
    }
}

fn report_basins(map: &HeightMap) {
    let mut basins = map.basins();
    basins.sort_by_key(|basin| Reverse(basin.size()));
    let measure = basins
        .iter()
        .take(3)
        .fold(1, |measure, basin| measure * basin.size());
    println!("Measure of three largest basins is: {}", measure);
}

fn report_risk_levels(map: &HeightMap) {
    let risk_level: u32 = map
        .risk_levels()
        .iter()
        .fold(0, |sum, risk_level| sum + *risk_level as u32);
    println!("The rish level is: {}", risk_level);
}

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("failed to parse heightmap")]
    ParseHeightMap(),
}

type Result<T> = result::Result<T, Error>;

struct HeightMap(Vec<Vec<u8>>);

impl HeightMap {
    fn parse<'a, Iter>(lines: Iter) -> Result<HeightMap>
    where
        Iter: Iterator<Item = &'a str>,
    {
        let mut map = vec![];
        let mut prior_width = None;
        for line in lines {
            let heights = line.as_bytes();
            let width = heights.len();
            match prior_width {
                Some(prior_width) if width != prior_width => {
                    return Err(Error::ParseHeightMap())
                }
                None => prior_width = Some(width),
                _ => {}
            }

            if heights
                .iter()
                .any(|height| *height < b'0' || *height > b'9')
            {
                return Err(Error::ParseHeightMap());
            }

            map.push(
                heights
                    .iter()
                    .copied()
                    .map(|height| height - b'0')
                    .collect::<Vec<u8>>(),
            );
        }

        Ok(HeightMap(map))
    }

    fn width(&self) -> usize {
        self.0[0].len()
    }

    fn length(&self) -> usize {
        self.0.len()
    }

    fn low_points(&self) -> Vec<u8> {
        let mut low_points = vec![];
        let max_row = self.length() - 1;
        let max_col = self.width() - 1;
        let left = |row: usize, column: usize| self.0[row][column - 1];
        let above = |row: usize, column: usize| self.0[row - 1][column];
        let right = |row: usize, column: usize| self.0[row][column + 1];
        let below = |row: usize, column: usize| self.0[row + 1][column];

        for row in 0..=max_row {
            for col in 0..=max_col {
                let cell = self.0[row][col];
                let low_point = match (row, col) {
                    // top left
                    (0, 0) => cell < right(0, 0) && cell < below(0, 0),
                    // top right
                    (0, col) if col == max_col => {
                        cell < left(0, col) && cell < below(0, col)
                    }
                    // top
                    (0, col) => {
                        cell < left(0, col)
                            && cell < right(0, col)
                            && cell < below(0, col)
                    }
                    // bottom left
                    (row, 0) if row == max_row => {
                        cell < above(row, 0) && cell < right(row, 0)
                    }
                    // bottom right
                    (row, col) if row == max_row && col == max_col => {
                        cell < left(row, col) && cell < above(row, col)
                    }
                    // bottom
                    (row, col) if row == max_row => {
                        cell < left(row, col)
                            && cell < above(row, col)
                            && cell < right(row, col)
                    }
                    // left
                    (row, 0) => {
                        cell < above(row, 0)
                            && cell < right(row, 0)
                            && cell < below(row, 0)
                    }
                    // right
                    (row, col) if col == max_col => {
                        cell < left(row, col)
                            && cell < above(row, col)
                            && cell < below(row, col)
                    }
                    (row, col) => {
                        cell < left(row, col)
                            && cell < above(row, col)
                            && cell < right(row, col)
                            && cell < below(row, col)
                    }
                };
                if low_point {
                    low_points.push(cell);
                }
            }
        }

        low_points
    }

    fn risk_levels(&self) -> Vec<u8> {
        self.low_points()
            .into_iter()
            .map(|low_point| low_point + 1)
            .collect()
    }

    fn basins(&self) -> Vec<Basin> {
        let mut mappings = BasinMappings::new(self.width(), self.length());
        let max_row = self.length() - 1;
        let max_col = self.width() - 1;
        for row in 0..=max_row {
            for col in 0..=max_col {
                let cell = self.0[row][col];
                if cell == 9 {
                    mappings.set_basin_border(row, col);
                    continue;
                }
                match (row, col) {
                    // top left
                    (0, 0) => mappings.new_basin(0, 0, cell),
                    // top
                    (0, col) => {
                        if let Some(basin) = mappings.left(0, col) {
                            mappings.set_basin(0, col, cell, basin);
                        } else {
                            mappings.new_basin(0, col, cell);
                        }
                    }
                    // left
                    (row, 0) => {
                        if let Some(basin) = mappings.above(row, 0) {
                            mappings.set_basin(row, 0, cell, basin);
                        } else {
                            mappings.new_basin(row, 0, cell);
                        }
                    }
                    // others
                    (row, col) => {
                        if let Some(basin) = mappings.left(row, col) {
                            mappings.set_basin(row, col, cell, basin);
                            if let Some(other_basin) = mappings.above(row, col)
                            {
                                mappings.merge_basin(basin, other_basin);
                            }
                        } else if let Some(basin) = mappings.above(row, col) {
                            mappings.set_basin(row, col, cell, basin);
                        } else {
                            mappings.new_basin(row, col, cell);
                        }
                    }
                };
            }
        }

        mappings.basins()
    }
}

struct BasinMappings {
    names: Vec<String>,
    mappings: Vec<Vec<(Option<u8>, Option<usize>)>>,
}

impl BasinMappings {
    fn new(width: usize, length: usize) -> Self {
        BasinMappings {
            names: vec![],
            mappings: vec![vec![(None, None); width]; length],
        }
    }

    fn new_basin(&mut self, row: usize, col: usize, value: u8) {
        let basin = self.names.len();
        let name = (basin + 1).to_string();
        self.names.push(name);
        self.set_basin(row, col, value, basin);
    }

    fn set_basin(&mut self, row: usize, col: usize, value: u8, basin: usize) {
        self.mappings[row][col] = (Some(value), Some(basin));
    }

    fn merge_basin(&mut self, basin: usize, other_basin: usize) {
        let basin_name = self.names[basin].clone();
        let other_basin_name = self.names[other_basin].clone();
        let mut names = self
            .names
            .iter()
            .map(|name| {
                if *name == other_basin_name {
                    basin_name.clone()
                } else {
                    name.clone()
                }
            })
            .collect::<Vec<String>>();
        std::mem::swap(&mut self.names, &mut names)
    }

    fn set_basin_border(&mut self, row: usize, col: usize) {
        self.mappings[row][col] = (Some(9), None);
    }

    fn left(&self, row: usize, col: usize) -> Option<usize> {
        self.mappings[row][col - 1].1
    }

    fn above(&self, row: usize, col: usize) -> Option<usize> {
        self.mappings[row - 1][col].1
    }

    fn basins(&self) -> Vec<Basin> {
        let mut basins: HashMap<&str, Vec<BasinPoint>> = HashMap::new();
        for row in self.mappings.iter() {
            for cell in row.iter() {
                if let (Some(height), Some(basin)) = cell {
                    if *height == 9 {
                        continue;
                    }

                    let basin = &*self.names[*basin];
                    basins
                        .entry(basin)
                        .or_insert_with(Vec::new)
                        .push(BasinPoint {})
                }
            }
        }

        basins
            .into_iter()
            .map(|(_name, points)| Basin { points })
            .collect()
    }
}

struct Basin {
    points: Vec<BasinPoint>,
}

impl Basin {
    fn size(&self) -> usize {
        self.points.len()
    }
}

struct BasinPoint;

#[cfg(test)]
mod tests {
    use super::HeightMap;

    #[test]
    fn height_map_parse() {
        let map = HeightMap::parse(INPUT.split('\n')).expect("valid input");
        assert_eq!(map.width(), 10);
        assert_eq!(map.length(), 5);
    }

    #[test]
    fn height_map_low_points() {
        let map = HeightMap::parse(INPUT.split('\n')).expect("valid input");

        let low_points = map.low_points();

        assert_eq!(low_points, vec![1, 0, 5, 5]);
    }

    #[test]
    fn height_map_risk_levels() {
        let map = HeightMap::parse(INPUT.split('\n')).expect("valid input");

        let risk_levels = map.risk_levels();

        assert_eq!(risk_levels, vec![2, 1, 6, 6]);
    }

    #[test]
    fn height_map_basins() {
        let map = HeightMap::parse(INPUT.split('\n')).expect("valid input");

        let mut basins = map.basins();

        basins.sort_by(|a, b| b.points.len().cmp(&a.points.len()));
        let measure = basins
            .iter()
            .take(3usize)
            .fold(1, |measure, basin| measure * basin.points.len());

        assert_eq!(measure, 1134);
    }

    const INPUT: &str = r"2199943210
3987894921
9856789892
8767896789
9899965678";
}
