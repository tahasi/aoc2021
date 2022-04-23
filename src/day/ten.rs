use std::{path::PathBuf, str::FromStr};

use structopt::{self, StructOpt};

use super::read_lines;

#[derive(Debug, thiserror::Error)]
#[error("Failed to parse mode from '{0}'")]
pub struct ParseModeError(String);

#[derive(Debug, StructOpt)]
pub struct Command {
    #[structopt(required(true), parse(from_os_str))]
    input: PathBuf,

    #[structopt(default_value("detect-corrupted"), long)]
    mode: Mode,
}

#[derive(Debug, StructOpt)]
pub enum Mode {
    DetectCorrupted,
    Repair,
}

impl FromStr for Mode {
    type Err = ParseModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "detect-corrupted" => Ok(Mode::DetectCorrupted),
            "repair" => Ok(Mode::Repair),
            _ => Err(ParseModeError(s.to_owned())),
        }
    }
}

impl Command {
    pub fn run(&self) -> anyhow::Result<()> {
        let lines = read_lines(&self.input)?;

        match self.mode {
            Mode::DetectCorrupted => {
                let points = lines
                    .iter()
                    .map(|line| check_syntax(&*line))
                    .fold(0, |sum, result| match result {
                        CheckResult::Corrupted {
                            expected: _,
                            found: _,
                            points,
                        } => sum + points,
                        _ => sum,
                    });

                println!("The total syntax error score is: {}", points);
            }
            Mode::Repair => {
                let mut points = lines
                    .iter()
                    .map(|line| check_syntax(&*line))
                    .map(|result| match result {
                        CheckResult::Incomplete {
                            original: _,
                            missing: _,
                            points,
                        } => Some(points),
                        _ => None,
                    })
                    .filter(Option::is_some)
                    .map(|points| points.expect("has some"))
                    .collect::<Vec<usize>>();
                points.sort_unstable();
                let mid_points = points[points.len() / 2];

                println!(
                    "The middle missing characters score is: {}",
                    mid_points
                );
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
enum CheckResult {
    Valid,
    Corrupted {
        expected: Option<char>,
        found: char,
        points: usize,
    },
    InvalidChar(char),
    Incomplete {
        original: String,
        missing: String,
        points: usize,
    },
}

impl std::cmp::Eq for CheckResult {}

fn check_syntax(line: &str) -> CheckResult {
    let mut state = vec![];
    for character in line.chars() {
        match character {
            '(' | '[' | '{' | '<' => state.push(close_character(character)),
            ')' | ']' | '}' | '>' => match state.pop() {
                Some(expected) if expected == character => {}
                expected => {
                    return CheckResult::Corrupted {
                        expected,
                        found: character,
                        points: corrupted_character_score(character),
                    }
                }
            },
            _ => return CheckResult::InvalidChar(character),
        }
    }
    if state.is_empty() {
        CheckResult::Valid
    } else {
        state.reverse();
        let (missing, points) = state.iter().fold(
            (String::with_capacity(state.len()), 0),
            |(mut missing, points), character| {
                missing.push(*character);
                (missing, points * 5 + missing_character_score(*character))
            },
        );
        CheckResult::Incomplete {
            original: line.to_owned(),
            missing,
            points,
        }
    }
}

fn corrupted_character_score(character: char) -> usize {
    match character {
        ')' => 3,
        ']' => 57,
        '}' => 1197,
        '>' => 25137,
        _ => 0,
    }
}

fn missing_character_score(character: char) -> usize {
    match character {
        ')' => 1,
        ']' => 2,
        '}' => 3,
        '>' => 4,
        _ => 0,
    }
}

fn close_character(character: char) -> char {
    match character {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        '<' => '>',
        _ => '!',
    }
}

#[cfg(test)]
mod tests {
    use super::{check_syntax, corrupted_character_score, CheckResult};
    use lazy_static::lazy_static;

    #[test]
    fn illegal_character_score_test() {
        let test_cases = [(')', 3), (']', 57), ('}', 1197), ('>', 25137)];
        for test in test_cases {
            let score = corrupted_character_score(test.0);
            assert_eq!(score, test.1);
        }
    }

    #[test]
    fn check_syntax_test() {
        for test in TEST_CASES.iter() {
            assert_eq!(check_syntax(test.input), test.expected);
        }
    }

    #[test]
    fn check_syntax_error_points_test() {
        let points = TEST_CASES
            .iter()
            .map(|test| check_syntax(test.input))
            .fold(0, |sum, result| match result {
                CheckResult::Corrupted {
                    expected: _,
                    found: _,
                    points,
                } => sum + points,
                _ => sum,
            });

        assert_eq!(points, 26397);
    }

    #[test]
    fn check_syntax_missing_characters_points_test() {
        let mut points = TEST_CASES
            .iter()
            .map(|test| check_syntax(test.input))
            .map(|result| match result {
                CheckResult::Incomplete {
                    original: _,
                    missing: _,
                    points,
                } => Some(points),
                _ => None,
            })
            .filter(Option::is_some)
            .map(|points| points.expect("has some"))
            .collect::<Vec<usize>>();
        points.sort();
        assert_eq!(points[points.len() / 2], 288957);
    }
    struct TestCase {
        input: &'static str,
        expected: CheckResult,
    }
    impl TestCase {
        fn incomplete(
            input: &'static str,
            missing: &'static str,
            points: usize,
        ) -> Self {
            TestCase {
                input,
                expected: CheckResult::Incomplete {
                    original: input.to_owned(),
                    missing: missing.to_owned(),
                    points,
                },
            }
        }

        fn corrupted(
            input: &'static str,
            expected: Option<char>,
            found: char,
            points: usize,
        ) -> TestCase {
            TestCase {
                input,
                expected: CheckResult::Corrupted {
                    expected,
                    found,
                    points,
                },
            }
        }
    }

    lazy_static! {
        static ref TEST_CASES: [TestCase; 10] = [
            TestCase::incomplete(
                "[({(<(())[]>[[{[]{<()<>>",
                "}}]])})]",
                288957
            ),
            TestCase::incomplete("[(()[<>])]({[<{<<[]>>(", ")}>]})", 5566),
            TestCase::corrupted(
                "{([(<{}[<>[]}>{[]{[(<()>",
                Some(']'),
                '}',
                1197
            ),
            TestCase::incomplete(
                "(((({<>}<{<{<>}{[]{[]{}",
                "}}>}>))))",
                1480781
            ),
            TestCase::corrupted("[[<[([]))<([[{}[[()]]]", Some(']'), ')', 3),
            TestCase::corrupted("[{[{({}]{}}([{[{{{}}([]", Some(')'), ']', 57),
            TestCase::incomplete(
                "{<[[]]>}<{[{[{[]{()[[[]",
                "]]}}]}]}>",
                995444
            ),
            TestCase::corrupted("[<(<(<(<{}))><([]([]()", Some('>'), ')', 3),
            TestCase::corrupted(
                "<{([([[(<>()){}]>(<<{{",
                Some(']'),
                '>',
                25137
            ),
            TestCase::incomplete("<{([{{}}[<[[[<>{}]]]>[]]", "])}>", 294),
        ];
    }
}
