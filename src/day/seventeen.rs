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
