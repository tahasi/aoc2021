use std::{cmp::Ordering, collections::BinaryHeap, path::PathBuf};

use structopt::{self, StructOpt};

use super::read_lines;

#[derive(Debug, StructOpt)]
pub struct Command {
    #[structopt(required(true), parse(from_os_str))]
    input: PathBuf,

    #[structopt(long)]
    full: bool
}

impl Command {
    pub fn run(&self) -> anyhow::Result<()> {
        let floor = CaveFloor::parse(
            read_lines(&self.input)?.iter().map(String::as_str),
            self.full
        )?;

        if let Some(least_path_risk) = floor.least_risk_path_value() {
            println!("Least risky path value: {}", least_path_risk);
        } else {
            println!("There's no path out of here");
        }
        

        Ok(())
    }
}

struct CaveFloor {
    nodes: Vec<Vec<u8>>,
    length: usize,
    width: usize,
}

impl CaveFloor {
    fn new(nodes: Vec<Vec<u8>>, width: usize) -> Self {
        let length = nodes.len();
        CaveFloor {
            nodes,
            length,
            width,
        }
    }

    fn parse<'iter, Iter>(lines: Iter, full: bool) -> Result<Self, ParseCaveFloorError>
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

        if full {
            let inc_or_wrap = |inc: u8, value: &u8| {
                let new_value = *value + inc;
                if new_value <= 9 { new_value } else { new_value - 9 }
            };
            let template = risk_levels.clone();
            for increment in 1u8..=4 {
                for (row, row_risk_levels) in template.iter().enumerate() {
                    risk_levels[row].append(&mut row_risk_levels.iter()
                        .map(|risk| inc_or_wrap(increment, risk)).collect());
                }
            }
            let template = risk_levels.clone();
            for increment in 1u8..=4 {
                for row_risk_levels in template.iter() {
                    risk_levels.push(row_risk_levels.iter()
                        .map(|risk| inc_or_wrap(increment, risk)).collect());
                }
            }
            line_len = line_len.map(|len| len * 5);
        }
        Ok(CaveFloor::new(
            risk_levels,
            line_len.expect("there's at least one line"),
        ))
    }

    fn edges(&self) -> Vec<Vec<Edge>> {
        (0..self.length)
            .flat_map(|row| (0..self.width).map(move |column| (row, column)))
            .map(|(row, column)| self.node_edges(row, column))
            .collect()
    }

    fn node_edges(&self, row: usize, column: usize) -> Vec<Edge> {
        let mut edges = vec![];
        // left edge
        if column != 0 {
            edges.push(self.edge(row, column - 1))
        }
        // top edge
        if row != 0 {
            edges.push(self.edge(row - 1, column))
        }
        // right edge
        if column != self.width - 1 {
            edges.push(self.edge(row, column + 1))
        }
        // bottom edge
        if row != self.length - 1 {
            edges.push(self.edge(row + 1, column))
        }
        edges
    }

    fn edge(&self, row: usize, column: usize) -> Edge {
        let node = self.width * row + column;
        let risk = self.nodes[row][column];
        Edge { node, risk }
    }

    fn least_risk_path_value(&self) -> Option<usize> {
        let start = 0;
        let goal = self.width * self.length - 1;
        let edges = self.edges();
        let mut dist: Vec<_> = (0..edges.len()).map(|_| usize::MAX).collect();
        let mut heap = BinaryHeap::new();

        dist[start] = 0;
        heap.push(State { cost: 0, position: start });

        while let Some(State { cost, position }) = heap.pop() {
            if position == goal { return Some(cost); }

            if cost > dist[position] { continue; }

            for edge in &edges[position] {
                let next = State { cost: cost + edge.risk as usize, position: edge.node };

                if next.cost < dist[next.position] {
                    heap.push(next);
                    dist[next.position] = next.cost;
                }
            }
        }

        None
    }
}

struct Edge {
    node: usize,
    risk: u8,
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct State {
    cost: usize,
    position: usize
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
            .then_with(|| self.position.cmp(&other.position))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
    fn least_risk_path_value() {
        let floor = CaveFloor::parse(INPUT.split('\n'), false).expect("valid input");

        assert_eq!(Some(40), floor.least_risk_path_value());
    }

    #[test]
    fn full_least_risk_path_value() {
        let floor = CaveFloor::parse(INPUT.split('\n'), true).expect("valid input");

        assert_eq!(Some(315), floor.least_risk_path_value());
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
