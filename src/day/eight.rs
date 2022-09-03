use std::{path::PathBuf, result, str::FromStr};

use lazy_static::lazy_static;
use structopt::{self, StructOpt};

use super::read_lines;

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error("invalid display digits format '{0}'")]
    ParseDisplayDigitsError(String),

    #[error("invalid display sample format '{0}'")]
    ParseDisplaySampleError(String),
}

type ParseResult<T> = result::Result<T, ParseError>;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("invalid display decoder patterns")]
    InvalidDisplayDecoderPatterns,
}

type Result<T> = result::Result<T, Error>;

#[derive(Debug, StructOpt)]
pub struct Command {
    #[structopt(required(true), parse(from_os_str))]
    input: PathBuf,
}

impl Command {
    pub fn run(&self) -> anyhow::Result<()> {
        let mut count = 0;
        for line in read_lines(&self.input)? {
            let sample = DisplaySample::parse(&line)?;
            let decoder = DisplayDecoder::build(sample.patterns())?;
            let message = decoder.decode(sample.output())?;
            count += message.parse::<i32>()?
        }
        println!("summed message output: {}", count);
        Ok(())
    }
}

struct DisplayDecoder {
    digit_patterns: [DigitPattern; 10],
}

impl DisplayDecoder {
    fn build<'a>(
        patterns: impl Iterator<Item = &'a DigitPattern>,
    ) -> Result<DisplayDecoder> {
        let mut patterns = patterns
            .map(DigitPattern::clone)
            .collect::<Vec<DigitPattern>>();
        patterns.sort_by_key(DigitPattern::len);
        if !patterns
            .iter()
            .zip(SORTED_DIGIT_PATTERN_LENGTHS.iter())
            .all(|(a, b)| a.len() == *b)
        {
            return Err(Error::InvalidDisplayDecoderPatterns);
        }

        // known patterns by lengths (e.g. one has 2 elements)
        let eight = patterns.remove(9);
        let four = patterns.remove(2);
        let seven = patterns.remove(1);
        let one = patterns.remove(0);

        let nine = patterns.remove(
            patterns[3..]
                .iter()
                .position(|pattern| pattern.contains(&four))
                .unwrap()
                + 3,
        );
        let zero = patterns.remove(
            patterns[3..]
                .iter()
                .position(|pattern| pattern.contains(&one))
                .unwrap()
                + 3,
        );
        let six = patterns.remove(3);
        let three = patterns.remove(
            patterns
                .iter()
                .position(|pattern| pattern.contains(&one))
                .unwrap(),
        );
        let element_in_nine_but_not_in_six: Vec<u8> = nine
            .0
            .iter()
            .copied()
            .filter(|nine_element| {
                six.0.iter().all(|six_element| nine_element != six_element)
            })
            .collect();
        assert_eq!(element_in_nine_but_not_in_six.len(), 1);
        let (two, five) = if patterns[0]
            .contains_element(element_in_nine_but_not_in_six[0])
        {
            (patterns.remove(0), patterns.remove(0))
        } else {
            (patterns.remove(1), patterns.remove(0))
        };

        Ok(DisplayDecoder {
            digit_patterns: [
                zero, one, two, three, four, five, six, seven, eight, nine,
            ],
        })
    }

    fn decode<'a>(
        &self,
        patterns: impl Iterator<Item = &'a DigitPattern>,
    ) -> Result<String> {
        let mut message = String::new();
        for pattern in patterns {
            if let Some(index) = self
                .digit_patterns
                .iter()
                .position(|digit_pattern| digit_pattern == pattern)
            {
                message.push((index as u8 + b'0') as char)
            } else {
                return Err(Error::InvalidDisplayDecoderPatterns);
            }
        }
        Ok(message)
    }
}

struct DisplaySample {
    patterns: Vec<DigitPattern>,
    output: Vec<DigitPattern>,
}

lazy_static! {
    static ref DIGIT_PATTERN_LENGTHS: Vec<usize> =
        vec![6, 2, 5, 5, 4, 5, 6, 3, 7, 6];
    static ref SORTED_DIGIT_PATTERN_LENGTHS: Vec<usize> =
        vec![2, 3, 4, 5, 5, 5, 6, 6, 6, 7];
}

#[derive(Clone, Debug, PartialEq)]
struct DigitPattern(Vec<u8>);

impl DigitPattern {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn contains(&self, other: &Self) -> bool {
        other.0.iter().all(|other_element| {
            self.0.iter().any(|element| other_element == element)
        })
    }

    fn contains_element(&self, element: u8) -> bool {
        self.0.iter().any(|self_element| *self_element == element)
    }
}

impl FromStr for DigitPattern {
    type Err = ParseError;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        let mut pattern: Vec<u8> = s.as_bytes().to_vec();
        pattern.sort_unstable();
        let mut prior_element = 0u8;
        for element in pattern.iter().copied() {
            if element == prior_element || !(b'a'..=b'g').contains(&element) {
                return Err(ParseError::ParseDisplayDigitsError(s.to_owned()));
            }
            prior_element = element;
        }
        Ok(DigitPattern(pattern))
    }
}

impl DisplaySample {
    fn parse(text: &str) -> ParseResult<DisplaySample> {
        let patterns_and_output: Vec<&str> = text
            .split('|')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if patterns_and_output.len() != 2 {
            return Err(ParseError::ParseDisplaySampleError(text.to_owned()));
        }
        let patterns = patterns_and_output[0]
            .split(' ')
            .map(DigitPattern::from_str)
            .collect::<ParseResult<Vec<DigitPattern>>>()?;
        if patterns.len() != 10 {
            return Err(ParseError::ParseDisplaySampleError(text.to_owned()));
        }
        let mut pattern_lens =
            patterns.iter().map(|p| p.len()).collect::<Vec<usize>>();
        pattern_lens.sort_unstable();
        if pattern_lens != (*SORTED_DIGIT_PATTERN_LENGTHS) {
            return Err(ParseError::ParseDisplaySampleError(text.to_owned()));
        }

        let output = patterns_and_output[1]
            .split(' ')
            .map(DigitPattern::from_str)
            .collect::<ParseResult<Vec<DigitPattern>>>()?;
        if output.len() != 4 {
            return Err(ParseError::ParseDisplaySampleError(text.to_owned()));
        }

        Ok(DisplaySample { patterns, output })
    }

    fn patterns(&self) -> impl Iterator<Item = &DigitPattern> {
        self.patterns.iter()
    }

    fn output(&self) -> impl Iterator<Item = &DigitPattern> {
        self.output.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::{DigitPattern, DisplayDecoder, DisplaySample};
    use std::str::FromStr;

    #[test]
    fn display_sample_parse() {
        let expected_patterns = vec![
            "be", "cfbegad", "cbdgef", "fgaecd", "cgeb", "fdcge", "agebfd",
            "fecdb", "fabcd", "edb",
        ];
        let expected_output = vec!["fdgacbe", "cefdb", "cefbgd", "gcbe"];
        let text = format!(
            "{} | {}",
            expected_patterns.join(" "),
            expected_output.join(" ")
        );

        let sample = DisplaySample::parse(&text).expect("valid text input");

        assert!(sample
            .patterns()
            .zip(expected_patterns.iter())
            .all(|(actual, expected_str)| *actual
                == DigitPattern::from_str(expected_str).unwrap()));
        assert!(sample
            .output()
            .zip(expected_output.iter())
            .all(|(actual, expected_str)| *actual
                == DigitPattern::from_str(expected_str).unwrap()));
    }

    #[test]
    fn display_decoder_decode() {
        let sample =
            DisplaySample::parse(SINGLE_INPUT.0).expect("valid text input");
        let decoder =
            DisplayDecoder::build(sample.patterns()).expect("valid patterns");

        let message = decoder.decode(sample.output()).expect("valid output");

        assert_eq!(message, SINGLE_INPUT.1);
    }

    #[test]
    fn test_run() {
        let mut count = 0;
        for line in INPUT {
            let sample =
                DisplaySample::parse(line.0).expect("valid text input");
            let decoder =
                DisplayDecoder::build(sample.patterns()).expect("valid patter");
            let message =
                decoder.decode(sample.output()).expect("valid output");
            assert_eq!(message, line.1);
            count += message.parse::<i32>().expect("valid number");
        }

        assert_eq!(count, 61229);
    }

    const SINGLE_INPUT: (&str, &str) = (
        "be cfbegad cbdgef fgaecd cgeb fdcge agebfd fecdb fabcd edb | fdgacbe cefdb cefbgd gcbe",
        "8394",
    );
    const INPUT: [(&str, &str); 10] = [
        ("be cfbegad cbdgef fgaecd cgeb fdcge agebfd fecdb fabcd edb | fdgacbe cefdb cefbgd gcbe", "8394"),
        ("edbfga begcd cbg gc gcadebf fbgde acbgfd abcde gfcbed gfec | fcgedb cgb dgebacf gc", "9781"),
        ("fgaebd cg bdaec gdafb agbcfd gdcbef bgcad gfac gcb cdgabef | cg cg fdcagb cbg", "1197"),
        ("fbegcd cbd adcefb dageb afcb bc aefdc ecdab fgdeca fcdbega | efabcd cedba gadfec cb", "9361"),
        ("aecbfdg fbg gf bafeg dbefa fcge gcbea fcaegb dgceab fcbdga | gecf egdcabf bgf bfgea", "4873"),
        ("fgeab ca afcebg bdacfeg cfaedg gcfdb baec bfadeg bafgc acf | gebdcfa ecba ca fadegcb", "8418"),
        ("dbcfg fgd bdegcaf fgec aegbdf ecdfab fbedc dacgb gdcebf gf | cefg dcbef fcge gbcadfe", "4548"),
        ("bdfegc cbegaf gecbf dfcage bdacg ed bedf ced adcbefg gebcd | ed bcgafe cdgba cbgef", "1625"),
        ("egadfb cdbfeg cegd fecab cgb gbdefca cg fgcdab egfdb bfceg | gbdfcae bgc cg cgb", "8717"),
        ("gcafb gcf dcaebfg ecagb gf abcdeg gaef cafbge fdbac fegbdc | fgae cfgab fg bagce", "4315"),
    ];
}
