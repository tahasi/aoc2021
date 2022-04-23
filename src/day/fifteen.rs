use std::{option::Iter, path::PathBuf, process::Child};

use structopt::{self, StructOpt};

use super::read_lines;

#[derive(Debug, StructOpt)]
pub struct Command {
    #[structopt(required(true), parse(from_os_str))]
    input: PathBuf,
}

impl Command {
    pub fn run(&self) -> anyhow::Result<()> {
        let floor = CaveFloor::parse(
            read_lines(&self.input)?.iter().map(String::as_str),
        )?;

        //println!("Least risky path value: {}", floor.least_risk_path_value());
        Ok(())
    }
}

#[derive(Clone)]
struct Node<'a> {
    floor: &'a CaveFloor,
    id: String,
    row: usize,
    col: usize,
    chiten_density: u8,
}

impl<'a> Node<'a> {
    fn new(
        floor: &'a CaveFloor,
        row: usize,
        col: usize,
        chiten_density: u8,
    ) -> Self {
        Node {
            floor,
            id: format!("{row}:{col}"),
            row,
            col,
            chiten_density,
        }
    }

    fn chiten_density(&self) -> u8 {
        self.chiten_density
    }
}

impl<'a> PartialEq for Node<'a> {
    fn eq(&self, other: &Node<'a>) -> bool {
        std::ptr::eq(self.floor, other.floor)
            && self.row == other.row
            && self.col == other.col
            && self.chiten_density == other.chiten_density
    }
}

impl<'a> Eq for Node<'a> {}

struct CaveFloor {
    nodes: Vec<Vec<u8>>,
}

impl CaveFloor {
    fn start(&self) -> Node {
        Node::new(&self, 0, 0, self.nodes[0][0])
    }

    fn end(&self) -> Node {
        let row = self.nodes.len() - 1;
        let col = self.nodes[row].len() - 1;
        Node::new(&self, row, col, self.nodes[row][col])
    }
}

impl CaveFloor {
    fn new(nodes: Vec<Vec<u8>>) -> Self {
        CaveFloor { nodes }
    }

    fn parse<'iter, Iter>(lines: Iter) -> Result<Self, ParseCaveFloorError>
    where
        Iter: Iterator<Item = &'iter str>,
    {
        let mut risk_levels = Vec::new();
        let mut line_len = None;
        for line in lines {
            let line_levels = line
                .chars()
                .into_iter()
                .map(|c| match c {
                    n @ '0'..='9' => Ok((n as u8) - b'0'),
                    _ => Err(ParseCaveFloorError::new(line)),
                })
                .collect::<Result<Vec<_>, _>>()?;
            if let Some(len) = line_len {
                if len != line_levels.len() {
                    return Err(ParseCaveFloorError::new(line));
                }
            } else {
                line_len = Some(line_levels.len());
            }
            risk_levels.push(line_levels);
        }
        Ok(CaveFloor::new(risk_levels))
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to parse cave floor from '{0}'")]
pub struct ParseCaveFloorError(String);
impl ParseCaveFloorError {
    fn new(text: &str) -> ParseCaveFloorError {
        ParseCaveFloorError(text.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::CaveFloor;

    #[test]
    fn nodes() {
        let floor = CaveFloor::parse(INPUT.split('\n')).expect("valid input");

        assert_eq!(floor.start().chiten_density, 1u8);
        assert_eq!(floor.end().chiten_density, 1u8);
    }

    const INPUT: &str = "1163751742
1381373672
2136511328
3694931569
7463417111
1319128137
1359912421
3125421639
1293138521
2311944581";
}
