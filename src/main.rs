use structopt::{self, StructOpt};

mod day;

#[derive(Debug, StructOpt)]
struct AdventOfCode {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    One(day::one::Command),
    Two(day::two::Command),
    Three(day::three::Command),
    Four(day::four::Command),
    Five(day::five::Command),
    Six(day::six::Command),
    Seven(day::seven::Command),
    Eight(day::eight::Command),
    Nine(day::nine::Command),
    Ten(day::ten::Command),
    Eleven(day::eleven::Command),
    Twelve(day::twelve::Command),
    Thirteen(day::thirteen::Command),
    Fourteen(day::fourteen::Command),
    Fifteen(day::fifteen::Command),
}

fn main() {
    let opt = AdventOfCode::from_args();
    if let Err(err) = match opt.command {
        Command::One(command) => command.run(),
        Command::Two(command) => command.run(),
        Command::Three(command) => command.run(),
        Command::Four(command) => command.run(),
        Command::Five(command) => command.run(),
        Command::Six(command) => command.run(),
        Command::Seven(command) => command.run(),
        Command::Eight(command) => command.run(),
        Command::Nine(command) => command.run(),
        Command::Ten(command) => command.run(),
        Command::Eleven(command) => command.run(),
        Command::Twelve(command) => command.run(),
        Command::Thirteen(command) => command.run(),
        Command::Fourteen(command) => command.run(),
        Command::Fifteen(command) => command.run(),
    } {
        eprintln!("{}", err);
    }
}
