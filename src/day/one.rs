use anyhow::{anyhow, Result};
use colored::*;
use structopt::{self, StructOpt};

#[derive(Debug, StructOpt)]
pub struct Command {
    #[structopt(required(true))]
    depth_measurements: Vec<usize>,

    #[structopt(long, default_value("1"))]
    window_size: usize,
}

impl Command {
    pub fn run(&self) -> Result<()> {
        match self.window_size {
            0 => Err(anyhow!("window-size must be 1 or greater")),
            1 => {
                self.report_measures();
                Ok(())
            }
            _ => {
                self.report_sums();
                Ok(())
            }
        }
    }

    fn report_measures(&self) {
        let mut increased_measures = 0;
        let mut prior = None;
        for measure in &self.depth_measurements {
            match prior {
                None => println!("{} (N/A - no previous measurement)", measure),
                Some(prior) if prior < measure => {
                    println!("{} ({})", measure, "increased".bold());
                    increased_measures += 1;
                }
                Some(prior) if prior > measure => {
                    println!("{} (decreased)", measure)
                }
                _ => println!("{} (no change)", measure),
            }

            prior = Some(measure);
        }

        println!("{} increasing measures", increased_measures);
    }

    fn report_sums(&self) {
        let mut increased_sums = 0;
        let mut prior = None;
        let measure_count = self.depth_measurements.len();
        for index in 0..=(measure_count - self.window_size) {
            let sum: usize = self.depth_measurements
                [index..(self.window_size + index)]
                .iter()
                .sum();
            match prior {
                None => println!("{} (N/A - no previous sum)", sum),
                Some(prior) if prior < sum => {
                    println!("{} ({})", sum, "increased".bold());
                    increased_sums += 1;
                }
                Some(prior) if prior > sum => println!("{} (decreased)", sum),
                _ => println!("{} (no change)", sum),
            }

            prior = Some(sum);
        }

        println!("{} increasing sums", increased_sums);
    }
}
