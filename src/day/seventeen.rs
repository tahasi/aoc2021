use std::path::PathBuf;

use structopt::{self, StructOpt};

use super::read_all_text;

#[derive(Debug, StructOpt)]
pub struct Command {
    #[structopt(required(true), parse(from_os_str))]
    input: PathBuf,
}

impl Command {
    pub fn run(&self) -> anyhow::Result<()> {
        let input = read_all_text(&self.input)?;
        println!("seventeen input: {input}");
        Ok(())
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Velocity {
    x: i32,
    y: i32,
}

#[derive(Copy, Clone, Debug)]
struct Probe {
    position: Position,
    velocity: Velocity,
}

impl Probe {
    fn launch(velocity: Velocity) -> Self {
        let position = Position::default();
        Probe { position, velocity }
    }
}

#[cfg(test)]
mod test {}
