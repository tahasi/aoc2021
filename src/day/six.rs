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
        let mut population = FishPopulation::parse(
            read_lines(&self.input)?.iter().map(String::as_ref),
        )?;
        for day in 1..=256 {
            population.next_day();
            println!("Day {:>2} population: {}", day, population.count());
        }
        Ok(())
    }
}

const FISH_STAGE_COUNT: usize = 9;

struct FishPopulation {
    count_in_reproductive_stage: Vec<u128>,
}

impl FishPopulation {
    fn parse<'a, Iter>(input: Iter) -> Result<FishPopulation>
    where
        Iter: Iterator<Item = &'a str>,
    {
        let count_in_reproductive_stage = input
            .flat_map(|line| line.split(','))
            .map(str::trim)
            .map(|entry| {
                entry.parse::<u8>().with_context(|| {
                    format!("failed to parse fish stage '{}'", entry)
                })
            })
            .fold(
                Ok(vec![0u128; FISH_STAGE_COUNT]),
                |population_result, parse_result| match population_result {
                    Ok(mut population) => match parse_result {
                        Ok(fish_stage) => {
                            population[fish_stage as usize] += 1;
                            Ok(population)
                        }
                        Err(err) => Err(err),
                    },
                    Err(err) => Err(err),
                },
            )?;
        Ok(FishPopulation {
            count_in_reproductive_stage,
        })
    }

    fn count(&self) -> u128 {
        self.count_in_reproductive_stage.iter().sum()
    }

    fn next_day(&mut self) {
        let ready_to_give_birth = self.count_in_reproductive_stage.remove(0);
        self.count_in_reproductive_stage.push(ready_to_give_birth);
        self.count_in_reproductive_stage[6] += ready_to_give_birth;
    }
}

#[cfg(test)]
mod tests {
    use super::FishPopulation;

    #[test]
    fn fish_population_parse() {
        let population = FishPopulation::parse(["3,4,3,1,2"].into_iter())
            .expect("valid input");

        assert_eq!(population.count(), 5);
    }

    #[test]
    fn fist_population_next_day() {
        let mut population = FishPopulation::parse(["3,4,3,1,2"].into_iter())
            .expect("valid input");

        population.next_day(); // population next day is 2,3,2,0,1
        assert_eq!(population.count(), 5);

        population.next_day(); // population next day is 1,2,1,6,0,8
        assert_eq!(population.count(), 6);

        population.next_day(); // population next day is 0,1,0,5,6,7,8
        assert_eq!(population.count(), 7);

        population.next_day(); // population next day is 6,0,6,4,5,6,7,8,8
        assert_eq!(population.count(), 9);

        population.next_day(); // population next day is 5,6,5,3,4,5,6,7,7,8
        assert_eq!(population.count(), 10);

        population.next_day(); // population next day is 4,5,4,2,3,4,5,6,6,7
        population.next_day(); // population next day is 3,4,3,1,2,3,4,5,5,6
        population.next_day(); // population next day is 2,3,2,0,1,2,3,4,4,5
        assert_eq!(population.count(), 10);

        population.next_day(); // population next day is 1,2,1,6,0,1,2,3,3,4,8
        population.next_day(); // population next day is 0,1,0,5,6,0,1,2,2,3,7,8
        population.next_day(); // population next day is 6,0,6,4,5,6,0,1,1,2,6,7,8,8,8
        assert_eq!(population.count(), 15);
    }
}
