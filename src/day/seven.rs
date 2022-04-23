use std::path::PathBuf;

use anyhow::{Context, Result};
use structopt::{self, StructOpt};

use super::read_lines;

#[derive(Debug, StructOpt)]
pub struct Command {
    #[structopt(required(true), parse(from_os_str))]
    input: PathBuf,
}

impl Command {
    pub fn run(&self) -> Result<()> {
        let submarines = CrabSubmarineManager::parse(
            read_lines(&self.input)?.iter().map(String::as_ref),
        )?;

        let (sum, count) = submarines
            .positions()
            .iter()
            .fold((0, 0), |(sum, count), position| (sum + position, count + 1));
        let average = sum as f64 / count as f64;
        let mut sorted_positions: Vec<u32> =
            submarines.positions().iter().copied().collect();
        sorted_positions.sort_unstable();
        let median = sorted_positions[sorted_positions.len() / 2];

        println!(
            "The average position is {}; the cost to move to rounded average ({}) is {}",
            average,
            average.round() as u32,
            submarines.cost_to_move(average.round() as u32)
        );
        println!(
            "The median position is {}; the cost to move to median is {}",
            median,
            submarines.cost_to_move(median)
        );
        let minimum = sorted_positions[0];
        println!(
            "The minimum position is {}; the cost to move to minimum is {}",
            minimum,
            submarines.cost_to_move(minimum)
        );
        let maximum = sorted_positions[sorted_positions.len() - 1];
        println!(
            "The maximum position is {}; the cost to move to maximum is {}",
            maximum,
            submarines.cost_to_move(maximum)
        );
        let mut move_costs = vec![0; (maximum - minimum + 1) as usize];
        for (index, position) in (minimum..=maximum).enumerate() {
            move_costs[index] = submarines.cost_to_move(position);
        }
        let (lowest_cost_index, lowest_cost) =
            move_costs.iter().copied().enumerate().fold(
                (0, u32::MAX),
                |(lowest_cost_index, lowest_cost), (index, cost)| {
                    if cost < lowest_cost {
                        (index, cost)
                    } else {
                        (lowest_cost_index, lowest_cost)
                    }
                },
            );
        println!(
            "Moving to position {} has the lowest cost of {}",
            (minimum as usize) + lowest_cost_index,
            lowest_cost
        );
        Ok(())
    }
}

struct CrabSubmarineManager {
    positions: Vec<u32>,
}

impl CrabSubmarineManager {
    fn parse<'a, Iter>(input: Iter) -> Result<CrabSubmarineManager>
    where
        Iter: Iterator<Item = &'a str>,
    {
        let positions = input
            .flat_map(|line| line.split(','))
            .map(str::trim)
            .map(|entry| {
                entry.parse::<u32>().with_context(|| {
                    format!("failed to parse position '{}'", entry)
                })
            })
            .collect::<Result<Vec<u32>>>()?;
        Ok(CrabSubmarineManager { positions })
    }

    fn positions(&self) -> &[u32] {
        &self.positions
    }

    fn cost_to_move(&self, position: u32) -> u32 {
        self.positions
            .iter()
            .copied()
            .fold(0, |cost, current_position| {
                let step_count = if current_position > position {
                    current_position - position
                } else {
                    position - current_position
                };
                cost + (1..=step_count).into_iter().sum::<u32>()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::CrabSubmarineManager;

    #[test]
    fn parse_test() {
        let expected: Vec<u32> = vec![16, 1, 2, 0, 4, 2, 7, 1, 2, 14];

        let input = expected
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let submarines = CrabSubmarineManager::parse([&*input].into_iter())
            .expect("valid input");

        assert_eq!(submarines.positions(), expected);
    }

    #[test]
    fn avg_vs_median() {
        let mut positions: Vec<u32> = vec![16, 1, 2, 0, 4, 2, 7, 1, 2, 14];

        let (sum, count) = positions
            .iter()
            .fold((0, 0), |(sum, count), n| (sum + n, count + 1));
        let avg = sum as f64 / count as f64;
        println!("{}", avg);

        positions.sort();
        let median = positions[positions.len() / 2];
        println!("{}", median);
    }

    #[test]
    fn sum_of_steps() {
        let sum: u32 = (1..=11).into_iter().sum();
        assert_eq!(sum, 66);
    }
}
