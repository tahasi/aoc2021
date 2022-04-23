use std::{
    cmp::{self, Ordering},
    fmt::Display,
    path::PathBuf,
    str::FromStr,
};

use structopt::{self, StructOpt};

use super::read_lines;

#[derive(Debug, thiserror::Error)]
#[error("Failed to parse mode from '{0}'")]
pub struct ParseModeError(String);

#[derive(Debug, StructOpt)]
pub struct Command {
    #[structopt(required(true), parse(from_os_str))]
    input: PathBuf,

    #[structopt(default_value("fold-one-count"), long)]
    mode: Mode,
}

#[derive(Debug, StructOpt)]
pub enum Mode {
    FoldOneCount,
    FoldAllRender,
}

impl FromStr for Mode {
    type Err = ParseModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fold-one-count" => Ok(Mode::FoldOneCount),
            "fold-all-render" => Ok(Mode::FoldAllRender),
            _ => Err(ParseModeError(s.to_owned())),
        }
    }
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::FoldOneCount => write!(f, "fold-one-count"),
            Mode::FoldAllRender => write!(f, "fold-all-render"),
        }
    }
}

impl Command {
    pub fn run(&self) -> anyhow::Result<()> {
        let mut transparency = Transparency::parse(
            read_lines(&self.input)?.iter().map(String::as_ref),
        )?;
        match self.mode {
            Mode::FoldOneCount => {
                transparency.fold();
                println!(
                    "Dots after one fold: {}",
                    transparency.dots().count()
                );
            }
            Mode::FoldAllRender => {
                while transparency.fold().is_some() {}
                let mut grid = vec![
                    vec!['.'; transparency.height()];
                    transparency.width()
                ];
                for dot in transparency.dots() {
                    grid[dot.x][dot.y] = '#';
                }
                for row in grid {
                    println!("{}", row.iter().collect::<String>());
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to parse transparency from '{0}'")]
pub struct ParseTransparencyError(String);
fn parse_error(text: &str) -> ParseTransparencyError {
    ParseTransparencyError(text.to_owned())
}

#[derive(Clone, Debug)]
struct Transparency {
    dots: Vec<Dot>,
    height: usize,
    width: usize,
    pending_folds: Vec<Fold>,
    applied_folds: Vec<Fold>,
}

#[derive(Clone, Copy, Debug)]
struct Dot {
    x: usize,
    y: usize,
}

#[derive(Clone, Copy, Debug)]
enum Fold {
    Horizontal(usize),
    Vertical(usize),
}

const FOLD_ALONG: &str = "fold along ";

impl Transparency {
    fn parse<'iter, Iter>(
        lines: Iter,
    ) -> Result<Transparency, ParseTransparencyError>
    where
        Iter: Iterator<Item = &'iter str>,
    {
        let mut dots = Vec::new();
        let mut height = 0;
        let mut width = 0;
        let mut folds = Vec::new();

        for line in lines.into_iter().map(str::trim) {
            if line.is_empty() {
                continue;
            }

            if let Some(text) = line.strip_prefix(FOLD_ALONG) {
                folds.push(Fold::parse(text)?)
            } else {
                let dot = Dot::parse(line)?;
                height = std::cmp::max(dot.y, height);
                width = std::cmp::max(dot.x, width);
                dots.push(dot);
            }
        }
        dots.sort_unstable();

        Ok(Transparency {
            dots,
            height: height + 1,
            width: width + 1,
            pending_folds: folds,
            applied_folds: Vec::new(),
        })
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn dots(&self) -> impl Iterator<Item = &Dot> {
        self.dots.iter()
    }

    #[allow(dead_code)]
    fn pending_folds(&self) -> impl Iterator<Item = &Fold> {
        self.pending_folds.iter()
    }

    #[allow(dead_code)]
    fn applied_folds(&self) -> impl Iterator<Item = &Fold> {
        self.applied_folds.iter()
    }

    fn fold(&mut self) -> Option<Fold> {
        if let Some(fold) = self.pending_folds.first() {
            let fold = *fold;
            match fold {
                Fold::Horizontal(value) => self.fold_horizontal(value),
                Fold::Vertical(value) => self.fold_vertical(value),
            };
            self.applied_folds.push(self.pending_folds.remove(0));
            Some(fold)
        } else {
            None
        }
    }

    fn fold_horizontal(&mut self, value: usize) {
        self.height = 0;
        for dot in self.dots.iter_mut() {
            if dot.y > value {
                dot.y = value - (dot.y - value);
            }
            self.height = cmp::max(dot.y, self.height);
        }
        self.height += 1;
        self.dots.sort_unstable();
        self.dots.dedup();
    }

    fn fold_vertical(&mut self, value: usize) {
        self.width = 0;
        for dot in self.dots.iter_mut() {
            if dot.x > value {
                dot.x = value - (dot.x - value);
            }
            self.width = cmp::max(dot.x, self.width);
        }
        self.width += 1;
        self.dots.sort_unstable();
        self.dots.dedup();
    }
}

impl Dot {
    fn parse(text: &str) -> Result<Dot, ParseTransparencyError> {
        let text = text.trim();
        if let Some(index) = text.find(',') {
            if let (Ok(x), Ok(y)) = (
                text[..index].parse::<usize>(),
                text[(index + 1)..].parse::<usize>(),
            ) {
                return Ok(Dot { x, y });
            }
        }
        Err(parse_error(text))
    }
}

impl Display for Dot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl PartialEq for Dot {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for Dot {}

impl PartialOrd for Dot {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.x.partial_cmp(&other.x) {
            Some(Ordering::Equal) => self.y.partial_cmp(&other.y),
            ord => ord,
        }
    }
}

impl Ord for Dot {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.x.cmp(&other.x) {
            Ordering::Equal => self.y.cmp(&other.y),
            ord => ord,
        }
    }
}

impl Fold {
    fn parse(text: &str) -> Result<Fold, ParseTransparencyError> {
        let index = text.rfind('=').ok_or_else(|| parse_error(text))?;
        if index != 1 {
            return Err(parse_error(text));
        }
        let value = text[(index + 1)..]
            .parse::<usize>()
            .map_err(|_| parse_error(text))?;
        match &text[(index - 1)..index] {
            "y" => Ok(Fold::Horizontal(value)),
            "x" => Ok(Fold::Vertical(value)),
            _ => Err(parse_error(text)),
        }
    }
}

impl Display for Fold {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fold::Horizontal(value) => write!(f, "y={}", value),
            Fold::Vertical(value) => write!(f, "x={}", value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Dot, Transparency};

    #[test]
    fn transparency_parse() {
        let transparency =
            Transparency::parse(INPUT.split("\n")).expect("valid input");

        assert_eq!(transparency.dots().count(), 18);
        assert_eq!(transparency.width(), 10);
        assert_eq!(transparency.height(), 14);
        assert_eq!(transparency.pending_folds().count(), 2);
        assert_eq!(transparency.applied_folds().count(), 0);
    }

    #[test]
    fn transparency_fold() {
        let mut transparency =
            Transparency::parse(INPUT.split("\n")).expect("valid input");

        transparency.fold();

        assert_eq!(transparency.dots().count(), 17);
        assert!(EXPECTED_FIRST_FOLD_DOTS
            .split('\n')
            .map(|text| Dot::parse(text).expect("valid imput"))
            .zip(transparency.dots())
            .all(|(expected, actual)| expected == *actual));
    }

    #[test]
    fn transparency_fold_second() {
        let mut transparency =
            Transparency::parse(INPUT.split("\n")).expect("valid input");

        transparency.fold();
        transparency.fold();

        assert_eq!(transparency.dots().count(), 16);
        assert!(EXPECTED_FIRST_FOLD_SECOND_DOTS
            .split('\n')
            .map(|text| Dot::parse(text).expect("valid imput"))
            .zip(transparency.dots())
            .all(|(expected, actual)| {
                let result = expected == *actual;
                result
            }));
    }

    const INPUT: &str = r"6,10
0,14
9,10
0,3
10,4
4,11
6,0
6,12
4,1
0,13
10,12
3,4
3,0
8,4
1,10
2,14
8,10
9,0

fold along y=7
fold along x=5";

    const EXPECTED_FIRST_FOLD_DOTS: &str = r"0,0
0,1
0,3
1,4
2,0
3,0
3,4
4,1
4,3
6,0
6,2
6,4
8,4
9,0
9,4
10,2
10,4";
    const EXPECTED_FIRST_FOLD_SECOND_DOTS: &str = r"0,0
0,1
0,2
0,3
0,4
1,0
1,4
2,0
2,4
3,0
3,4
4,0
4,1
4,2
4,3
4,4";
}
