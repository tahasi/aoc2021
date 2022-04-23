use anyhow::{anyhow, Context, Result};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};
use structopt::{self, StructOpt};

#[derive(Debug, StructOpt)]
pub struct Command {
    #[structopt(required(true), parse(from_os_str))]
    input: PathBuf,

    #[structopt(default_value("movement"), long)]
    mode: String,
}

const FORWARD: &str = "forward";
const UP: &str = "up";
const DOWN: &str = "down";
const MODE_MOVEMENT: &str = "movement";
const MODE_AIM: &str = "aim";

impl Command {
    pub fn run(&self) -> Result<()> {
        let file = File::open(&self.input).with_context(|| {
            format!("failed to open file '{}'", self.input.display())
        })?;
        match self.mode.as_ref() {
            MODE_MOVEMENT => {
                self.report_position_by_movements(BufReader::new(file));
                Ok(())
            }
            MODE_AIM => {
                self.report_position_by_aim(BufReader::new(file));
                Ok(())
            }
            invalid_mode => Err(anyhow!("invalid mode '{}'", invalid_mode)),
        }
    }

    fn report_position_by_movements(&self, reader: BufReader<File>) {
        let mut horizontal = 0;
        let mut vertical = 0;
        for line in reader.lines() {
            match line {
                Ok(ref text) => {
                    let movement: Vec<&str> = text.split(' ').collect();
                    if movement.len() != 2 {
                        eprintln!("invalid movement entry '{}'", &text);
                        continue;
                    }
                    let distance: i32 = match movement[1].parse() {
                        Ok(value) => value,
                        Err(err) => {
                            eprintln!(
                                "failed to parse movement '{}' due to {:?}",
                                &text, &err
                            );
                            continue;
                        }
                    };
                    let direction = match movement[0] {
                        FORWARD => {
                            horizontal += distance;
                            FORWARD
                        }
                        UP => {
                            vertical -= distance;
                            UP
                        }
                        DOWN => {
                            vertical += distance;
                            DOWN
                        }
                        _ => {
                            eprintln!("failed to parse movement '{}'", &text);
                            continue;
                        }
                    };
                    println!(
                        "{} {} ({}:{})[{}]",
                        direction,
                        distance,
                        horizontal,
                        vertical,
                        horizontal * vertical
                    );
                }
                Err(err) => eprintln!("failed to read text: {:?}", err),
            }
        }
    }

    fn report_position_by_aim(&self, reader: BufReader<File>) {
        let mut horizontal = 0;
        let mut vertical = 0;
        let mut aim = 0;
        for line in reader.lines() {
            match line {
                Ok(ref text) => {
                    let movement: Vec<&str> = text.split(' ').collect();
                    if movement.len() != 2 {
                        eprintln!("invalid movement entry '{}'", &text);
                        continue;
                    }
                    let distance: i32 = match movement[1].parse() {
                        Ok(value) => value,
                        Err(err) => {
                            eprintln!(
                                "failed to parse movement '{}' due to {:?}",
                                &text, &err
                            );
                            continue;
                        }
                    };
                    let direction = match movement[0] {
                        FORWARD => {
                            horizontal += distance;
                            vertical += aim * distance;
                            FORWARD
                        }
                        UP => {
                            aim -= distance;
                            UP
                        }
                        DOWN => {
                            aim += distance;
                            DOWN
                        }
                        _ => {
                            eprintln!("failed to parse movement '{}'", &text);
                            continue;
                        }
                    };
                    println!(
                        "{} {} ({}:{})[{}]",
                        direction,
                        distance,
                        horizontal,
                        vertical,
                        horizontal * vertical
                    );
                }
                Err(err) => eprintln!("failed to read text: {:?}", err),
            }
        }
    }
}
