use std::{cmp::Ordering, collections::HashMap, path::PathBuf};

use regex::Regex;
use structopt::{self, StructOpt};

use super::read_lines;

#[derive(Debug, StructOpt)]
pub struct Command {
    #[structopt(required(true), parse(from_os_str))]
    input: PathBuf,

    #[structopt(long)]
    steps: usize,
}

impl Command {
    pub fn run(&self) -> anyhow::Result<()> {
        let mut polymizer = Polymerizer::parse(
            read_lines(&self.input)?.iter().map(String::as_str),
        )?;
        for _ in 0..self.steps {
            polymizer.step();
        }
        let counts = polymizer.element_counts().collect::<Vec<_>>();
        println!(
            "Element counts:\n{}",
            counts
                .iter()
                .map(|(element, count)| format!("  [{}; {}]", element, count))
                .collect::<Vec<_>>()
                .join("\n")
        );
        println!("  Difference: {}", counts[0].1 - counts[counts.len() - 1].1);
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to parse polymerizer from '{0}'")]
pub struct ParsePolymerizerError(String);
impl ParsePolymerizerError {
    fn new(text: &str) -> ParsePolymerizerError {
        ParsePolymerizerError(text.to_owned())
    }
}

fn element_pair_counts(chars: &[char]) -> HashMap<ElementPair, usize> {
    let mut counts: HashMap<ElementPair, usize> = HashMap::new();
    for index in 0..(chars.len() - 1) {
        let pair = ElementPair::new(chars[index], chars[index + 1]);
        *counts.entry(pair).or_insert(0) += 1;
    }

    counts
}

type Element = char;

#[derive(Clone, Copy, Debug, Hash, PartialOrd, Ord, PartialEq, Eq)]
struct ElementPair {
    first: char,
    second: char,
}

impl ElementPair {
    fn new(first: char, second: char) -> ElementPair {
        ElementPair { first, second }
    }
}

#[derive(Debug)]
struct Polymerizer {
    insertions: HashMap<ElementPair, Element>,
    last_char: char,
    element_pair_counts: HashMap<ElementPair, usize>,
}

impl Polymerizer {
    fn parse<'iter, Iter>(
        lines: Iter,
    ) -> Result<Polymerizer, ParsePolymerizerError>
    where
        Iter: Iterator<Item = &'iter str>,
    {
        let insertion_regex = Regex::new("([A-Z]{2}) -> ([A-Z])")
            .map_err(|_| ParsePolymerizerError::new("regex"))?;
        let mut template = None;
        let mut insertions: HashMap<ElementPair, Element> = HashMap::new();

        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if template.is_none() {
                template = Some(line.to_owned());
                continue;
            }

            let capture = insertion_regex
                .captures(line)
                .ok_or_else(|| ParsePolymerizerError::new(line))?;
            let (pair, insertion) = if let (Some(pair), Some(insertion)) =
                (capture.get(1), capture.get(2))
            {
                let pair_chars = pair.as_str().chars().collect::<Vec<_>>();
                let pair = ElementPair::new(pair_chars[0], pair_chars[1]);
                let insertion = insertion
                    .as_str()
                    .chars()
                    .next()
                    .ok_or_else(|| ParsePolymerizerError::new(line))?;
                Ok((pair, insertion))
            } else {
                Err(ParsePolymerizerError::new(line))
            }?;
            insertions.insert(pair, insertion);
        }

        if template.is_none() || insertions.is_empty() {
            return Err(ParsePolymerizerError::new("empty"));
        }

        let template = template.expect("is some");
        let template_chars =
            template.chars().into_iter().collect::<Vec<char>>();
        let last_char = template_chars[template_chars.len() - 1];
        let element_pair_counts = element_pair_counts(&template_chars);
        Ok(Polymerizer {
            insertions,
            last_char,
            element_pair_counts,
        })
    }

    fn step(&mut self) {
        let mut pair_counts: HashMap<ElementPair, usize> = HashMap::new();
        for (pair, count) in self.element_pair_counts.iter() {
            let insertion = self.insertions[pair];
            let first_pair = ElementPair::new(pair.first, insertion);
            *pair_counts.entry(first_pair).or_insert(0) += count;
            let second_pair = ElementPair::new(insertion, pair.second);
            *pair_counts.entry(second_pair).or_insert(0) += count;
        }
        self.element_pair_counts = pair_counts;
    }

    fn element_counts(&self) -> impl Iterator<Item = (char, usize)> {
        let mut counts = self.element_pair_counts.iter().fold(
            HashMap::new(),
            |mut counts, (pair, count)| {
                *counts.entry(pair.first).or_insert(0) += count;
                counts
            },
        );
        *counts.entry(self.last_char).or_insert(0) += 1;
        let mut counts = counts
            .iter()
            .map(|(c, count)| (*c, *count))
            .collect::<Vec<_>>();
        counts.sort_unstable_by(|a, b| match a.1.cmp(&b.1) {
            Ordering::Equal => a.0.cmp(&b.0),
            Ordering::Greater => Ordering::Less,
            Ordering::Less => Ordering::Greater,
        });
        counts.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::day::fourteen::ElementPair;

    use super::Polymerizer;

    #[test]
    fn polymerizer_parse() {
        let polymerizer =
            Polymerizer::parse(INPUT.split('\n')).expect("valid input");

        assert_eq!(polymerizer.insertions[&ElementPair::new('C', 'H')], 'B');
        assert_eq!(polymerizer.insertions[&ElementPair::new('B', 'H')], 'H');
        assert_eq!(polymerizer.insertions[&ElementPair::new('C', 'N')], 'C');
        assert_eq!(
            polymerizer.element_counts().collect::<Vec<_>>(),
            vec![('N', 2), ('B', 1), ('C', 1)]
        );
    }

    #[test]
    fn polymerizer_step() {
        let mut polymerizer =
            Polymerizer::parse(INPUT.split('\n')).expect("valid input");

        polymerizer.step();
        assert_eq!(
            polymerizer.element_counts().collect::<Vec<_>>(),
            vec![('B', 2), ('C', 2), ('N', 2), ('H', 1)]
        );
    }

    #[test]
    fn polymerizer_four_steps() {
        let mut polymerizer =
            Polymerizer::parse(INPUT.split('\n')).expect("valid inputg");

        (0..4).for_each(|_| polymerizer.step());

        assert_eq!(
            polymerizer.element_counts().collect::<Vec<_>>(),
            vec![('B', 23), ('N', 11), ('C', 10), ('H', 5)]
        );
    }

    #[test]
    fn polymerizer_ten_steps() {
        let mut polymerizer =
            Polymerizer::parse(INPUT.split('\n')).expect("valid input");

        (0..10).for_each(|_| polymerizer.step());

        assert_eq!(
            polymerizer.element_counts().collect::<Vec<_>>(),
            vec![('B', 1749), ('N', 865), ('C', 298), ('H', 161)]
        );
    }

    const INPUT: &str = r"NNCB

    CH -> B
    HH -> N
    CB -> H
    NH -> C
    HB -> C
    HC -> B
    HN -> C
    NN -> C
    BH -> H
    NC -> B
    NB -> B
    BN -> B
    BB -> N
    BC -> B
    CC -> N
    CN -> C";
}
