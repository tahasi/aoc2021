use anyhow::{anyhow, Result};
use std::path::PathBuf;
use structopt::{self, StructOpt};

use super::read_lines;

const POWER_CONSUMPTION: &str = "power-consumption";
const LIFE_SUPPORT: &str = "life-support";

#[derive(Debug, StructOpt)]
pub struct Command {
    #[structopt(required(true), parse(from_os_str))]
    input: PathBuf,

    #[structopt(long)]
    system: String,
}

struct MeasureStats {
    set_bit_counts: Vec<usize>,
    count: usize,
}

fn get_measure_stats(lines: &[&str]) -> Result<MeasureStats> {
    let mut set_bit_counts: Vec<usize> = Vec::new();
    let mut count = 0;
    for line in lines {
        if count == 0 {
            set_bit_counts.extend(vec![0; line.len()]);
        } else if set_bit_counts.len() != line.len() {
            return Err(anyhow!(
                "input must have consistent bit count representation"
            ));
        }

        for (index, char) in line.chars().enumerate() {
            match char {
                '0' => {}
                '1' => set_bit_counts[index] += 1,
                c if c.is_ascii_whitespace() => {}
                _ => {
                    return Err(anyhow!(format!(
                        "invalid character '{}' in '{}'",
                        char, line
                    )))
                }
            }
        }

        count += 1;
    }

    Ok(MeasureStats {
        set_bit_counts,
        count,
    })
}

impl Command {
    pub fn run(&self) -> Result<()> {
        let owned_lines = read_lines(&self.input)?;
        let lines: Vec<&str> = owned_lines.iter().map(String::as_str).collect();
        match self.system.as_ref() {
            POWER_CONSUMPTION => self.calc_power_consumption(&lines),
            LIFE_SUPPORT => self.calc_life_support(&lines),
            _ => Err(anyhow!(format!("unknown system '{}'", &self.system))),
        }
    }

    fn calc_power_consumption(&self, lines: &[&str]) -> Result<()> {
        let stats = get_measure_stats(lines)?;
        let majority =
            stats.count / 2 + if stats.count % 2 == 1 { 1 } else { 0 };
        let mut gamma_rate: usize = 0;
        let mut epsilon_rate: usize = 0;
        for set_bit_count in stats.set_bit_counts {
            gamma_rate = (gamma_rate << 1)
                + if set_bit_count >= majority { 1 } else { 0 };
            epsilon_rate = (epsilon_rate << 1)
                + if set_bit_count < majority { 1 } else { 0 };
        }

        println!(
            "Gamma rate: {}, Epsilon rate: {}, Measure: {}",
            gamma_rate,
            epsilon_rate,
            gamma_rate * epsilon_rate
        );

        Ok(())
    }

    fn calc_life_support(&self, lines: &[&str]) -> Result<()> {
        let mut oxygen_rating: Vec<&str> = Vec::new();
        oxygen_rating.extend(lines);
        let mut scrubber_rating = oxygen_rating.clone();

        let mut index = 0;
        loop {
            let stats = get_measure_stats(&oxygen_rating)?;
            let majority =
                stats.count / 2 + if stats.count % 2 == 1 { 1 } else { 0 };
            let majority_value = if stats.set_bit_counts[index] >= majority {
                '1'
            } else {
                '0'
            };
            oxygen_rating = oxygen_rating
                .into_iter()
                .filter(|measure| {
                    measure
                        .chars()
                        .nth(index)
                        .expect("already validated length")
                        == majority_value
                })
                .collect();
            if oxygen_rating.len() == 1 {
                break;
            }
            index += 1;
        }

        index = 0;
        loop {
            let stats = get_measure_stats(&scrubber_rating)?;
            let majority =
                stats.count / 2 + if stats.count % 2 == 1 { 1 } else { 0 };
            let minority_value = if stats.set_bit_counts[index] < majority {
                '1'
            } else {
                '0'
            };
            scrubber_rating = scrubber_rating
                .into_iter()
                .filter(|measure| {
                    measure
                        .chars()
                        .nth(index)
                        .expect("already validated length")
                        == minority_value
                })
                .collect();
            if scrubber_rating.len() == 1 {
                break;
            }
            index += 1;
        }

        if oxygen_rating.len() == 1 && scrubber_rating.len() == 1 {
            let oxygen_rating_str = oxygen_rating[0];
            let oxygen_rating =
                i32::from_str_radix(oxygen_rating_str, 2).unwrap();
            let scrubber_rating_str = scrubber_rating[0];
            let scrubber_rating =
                i32::from_str_radix(scrubber_rating_str, 2).unwrap();
            let measure = oxygen_rating * scrubber_rating;
            println!(
                "{}({}) : {}({}) [{}]",
                oxygen_rating_str,
                oxygen_rating,
                scrubber_rating_str,
                scrubber_rating,
                measure
            );
        } else {
            println!("wtf");
        }

        Ok(())
    }
}
