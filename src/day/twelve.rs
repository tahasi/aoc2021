use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    path::PathBuf,
    result,
    str::FromStr,
};

use lazy_static::lazy_static;
use structopt::{self, StructOpt};

use super::read_lines;

#[derive(Debug, thiserror::Error)]
#[error("Failed to parse mode from '{0}'")]
pub struct ParseModeError(String);

#[derive(Debug, thiserror::Error)]
#[error("Failed to parse cave system from '{0}'")]
pub struct ParseCaveSystemError(String);

#[derive(Debug, thiserror::Error)]
#[error("Invalid cave connection {0}")]
pub struct InvalidCaveConnectionError(String);

#[derive(Debug, StructOpt)]
pub struct Command {
    #[structopt(required(true), parse(from_os_str))]
    input: PathBuf,

    #[structopt(default_value("paths"), long)]
    mode: Mode,
}

#[derive(Debug, StructOpt)]
pub enum Mode {
    Paths,
    SmallCaveVisitTwiceOnce,
}

impl FromStr for Mode {
    type Err = ParseModeError;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        match s {
            "paths" => Ok(Mode::Paths),
            "small-cave-visit-twice-once" => Ok(Mode::SmallCaveVisitTwiceOnce),
            _ => Err(ParseModeError(s.to_owned())),
        }
    }
}

impl Command {
    pub fn run(&self) -> anyhow::Result<()> {
        let mut system = CaveSystem::parse(
            read_lines(&self.input)?.iter().map(String::as_ref),
        )?;
        if let Mode::SmallCaveVisitTwiceOnce = self.mode {
            system.set_allow_visit_one_small_cave_twice(true);
        }

        println!("All cave paths from start to end");
        let paths = system.paths().expect("valid input");
        let mut paths: Vec<String> =
            paths.into_iter().map(|path| path.join(",")).collect();
        paths.sort_unstable_by_key(|path| path.to_lowercase());
        for path in paths.iter() {
            println!("  {}", path);
        }
        println!("  Total paths: {}", paths.len());
        println!(
            "  Unique paths: {}",
            paths.iter().collect::<HashSet<_>>().len()
        );
        Ok(())
    }
}

lazy_static! {
    static ref EMPTY_ADJOINING_CAVE_VEC: Vec<usize> = Vec::new();
}

struct CaveSystem {
    caves: Vec<Cave>,
    connections: HashMap<usize, Vec<usize>>,
    allow_visit_one_small_twice: bool,
}

impl CaveSystem {
    fn parse<'a, Iter: Iterator<Item = &'a str>>(
        lines: Iter,
    ) -> result::Result<Self, ParseCaveSystemError> {
        let mut caves: Vec<Cave> = vec![];
        let mut cave_indices: HashMap<&str, usize> = HashMap::new();
        let mut cave_connections: HashMap<usize, Vec<usize>> = HashMap::new();

        let mut store_cave =
            |cave_name: &'a str| -> Result<usize, ParseCaveSystemError> {
                if let Some(index) = cave_indices.get(cave_name) {
                    Ok(*index)
                } else {
                    let index = caves.len();
                    caves.push(Cave::from_str(cave_name)?);
                    cave_indices.insert(cave_name, index);
                    Ok(index)
                }
            };

        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let mut connection = line.split('-');
            let (start_index, end_index) =
                match (connection.next(), connection.next(), connection.next())
                {
                    (Some(start), Some(end), None) => {
                        (store_cave(start)?, store_cave(end)?)
                    }
                    _ => return Err(ParseCaveSystemError(line.to_owned())),
                };
            cave_connections
                .entry(start_index)
                .or_insert_with(Vec::new)
                .push(end_index);
            cave_connections
                .entry(end_index)
                .or_insert_with(Vec::new)
                .push(start_index);
        }

        Ok(CaveSystem {
            caves,
            connections: cave_connections,
            allow_visit_one_small_twice: false,
        })
    }

    fn paths(
        &self,
    ) -> result::Result<Vec<Vec<&'_ str>>, InvalidCaveConnectionError> {
        if let Some(start_index) = self
            .caves
            .iter()
            .position(|cave| matches!(cave, Cave::Start))
        {
            Ok(self
                .find_paths_to_end(start_index, &HashSet::new(), false)
                .into_iter()
                .map(|mut path| {
                    path.reverse();
                    path
                })
                .collect())
        } else {
            Err(InvalidCaveConnectionError("missing 'start'".to_owned()))
        }
    }

    fn find_paths_to_end<'a>(
        &'a self,
        cave_index: usize,
        visited_small_caves: &HashSet<usize>,
        mut visited_one_small_cave_twice: bool,
    ) -> Vec<Vec<&'a str>> {
        let cave = self.get_cave(cave_index);
        let adjoining_cave_indices = self
            .get_adjoining_cave_indices(cave_index)
            .iter()
            .copied()
            .filter(|adjoining_cave_index| {
                if visited_small_caves.contains(adjoining_cave_index) {
                    if self.allow_visit_one_small_twice
                        && !visited_one_small_cave_twice
                        && self.get_cave(*adjoining_cave_index).is_small()
                    {
                        visited_one_small_cave_twice = true;
                        true
                    } else {
                        false
                    }
                } else {
                    true
                }
            })
            .collect::<Vec<_>>();
        let adjoining_caves_paths =
            adjoining_cave_indices.iter().map(|adjoining_cave_index| {
                let adjoining_cave_index = *adjoining_cave_index;
                let adjoining_cave = self.get_cave(adjoining_cave_index);
                if adjoining_cave.is_end() {
                    vec![vec!["end"]]
                } else if cave.is_big() {
                    self.find_paths_to_end(
                        adjoining_cave_index,
                        visited_small_caves,
                        visited_one_small_cave_twice,
                    )
                } else {
                    let visited_small_caves = HashSet::from([cave_index])
                        .union(visited_small_caves)
                        .copied()
                        .collect();
                    self.find_paths_to_end(
                        adjoining_cave_index,
                        &visited_small_caves,
                        visited_one_small_cave_twice,
                    )
                }
            });
        adjoining_caves_paths
            .into_iter()
            .flat_map(|cave_paths| cave_paths.into_iter())
            .map(|mut path| {
                path.push(self.get_cave(cave_index).name());
                path
            })
            .collect()
    }

    fn get_cave(&self, cave_index: usize) -> &Cave {
        self.caves
            .get(cave_index)
            .expect("internal use of cave index works")
    }

    fn get_adjoining_cave_indices(&self, cave_index: usize) -> &[usize] {
        if let Some(indices) = self.connections.get(&cave_index) {
            indices
        } else {
            &*EMPTY_ADJOINING_CAVE_VEC
        }
    }

    fn set_allow_visit_one_small_cave_twice(&mut self, allow: bool) {
        self.allow_visit_one_small_twice = allow;
    }
}

#[derive(Debug)]
enum Cave {
    Start,
    End,
    Big(String),
    Small(String),
}

impl Cave {
    fn name(&self) -> &str {
        match self {
            Cave::Start => "start",
            Cave::End => "end",
            Cave::Big(name) => &*name,
            Cave::Small(name) => &*name,
        }
    }

    fn is_end(&self) -> bool {
        matches!(self, Cave::End)
    }

    fn is_big(&self) -> bool {
        matches!(self, Cave::Big(_))
    }

    fn is_small(&self) -> bool {
        matches!(self, Cave::Small(_))
    }
}

impl Display for Cave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl FromStr for Cave {
    type Err = ParseCaveSystemError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "start" => Ok(Cave::Start),
            "end" => Ok(Cave::End),
            name => {
                let name = name.to_owned();
                if name.chars().all(|character| character.is_uppercase()) {
                    Ok(Cave::Big(name))
                } else if name.chars().all(|character| character.is_lowercase())
                {
                    Ok(Cave::Small(name))
                } else {
                    Err(ParseCaveSystemError(name))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CaveSystem;
    use lazy_static::lazy_static;

    #[test]
    fn cave_system_parse() {
        let system =
            CaveSystem::parse(SIMPLE_TEST.cave_connections.iter().copied())
                .expect("valid input");

        let (caves, connections) = system.connections.iter().fold(
            (0, 0),
            |(caves, connections), (_, adjoining_caves)| {
                (caves + 1, connections + adjoining_caves.len())
            },
        );

        assert_eq!(caves, 6);
        assert_eq!(connections, 14);

        let max_cave_index = system.caves.len() - 1;
        let valid_cave_index = move |index: usize| index <= max_cave_index;
        assert!(system.connections.iter().all(|(index, adjoining_indices)| {
            valid_cave_index(*index)
                && adjoining_indices
                    .iter()
                    .all(|index| valid_cave_index(*index))
        }));
    }

    #[test]
    fn cave_system_paths() {
        let system =
            CaveSystem::parse(SIMPLE_TEST.cave_connections.iter().copied())
                .expect("valid input");

        let paths = system.paths().expect("valid");
        assert_equivalent_paths(&paths, SIMPLE_TEST.sorted_expected_paths);
    }

    #[test]
    fn cave_system_paths_larger() {
        let system =
            CaveSystem::parse(LARGER_TEST.cave_connections.iter().copied())
                .expect("valid input");

        let paths = system.paths().expect("valid");
        assert_equivalent_paths(&paths, LARGER_TEST.sorted_expected_paths);
    }

    #[test]
    fn cave_system_paths_largest() {
        let cave_connections = &[
            "fs-end", "he-DX", "fs-he", "start-DX", "pj-DX", "end-zg", "zg-sl",
            "zg-pj", "pj-he", "RW-he", "fs-DX", "pj-RW", "zg-RW", "start-pj",
            "he-WI", "zg-he", "pj-fs", "start-RW",
        ];
        let system = CaveSystem::parse(cave_connections.iter().copied())
            .expect("valid input");

        let paths = system.paths().expect("valid");
        assert_eq!(paths.len(), 226);
    }

    #[test]
    fn cave_system_paths_visit_one_small_twice() {
        let mut system = CaveSystem::parse(
            SIMPLE_TEST_VISIT_ONE_SMALL_TWICE
                .cave_connections
                .iter()
                .copied(),
        )
        .expect("valid input");
        system.set_allow_visit_one_small_cave_twice(true);

        let paths = system.paths().expect("valid");
        assert_equivalent_paths(
            &paths,
            SIMPLE_TEST_VISIT_ONE_SMALL_TWICE.sorted_expected_paths,
        );
    }

    fn assert_equivalent_paths(
        paths: &[Vec<&str>],
        sorted_expected_paths: &[&str],
    ) {
        let mut paths: Vec<String> =
            paths.into_iter().map(|path| path.join(",")).collect();
        paths.sort_unstable_by_key(|path| path.to_lowercase());
        if paths.len() == sorted_expected_paths.len() {
            for (path, expected_path) in paths.iter().zip(sorted_expected_paths)
            {
                assert_eq!(&*path, *expected_path);
            }
        } else {
            for path in paths.iter() {
                if !sorted_expected_paths.contains(&path.as_ref()) {
                    panic!("Unexpected path: {}", path);
                }
            }

            for expected_path in sorted_expected_paths.iter() {
                if !paths
                    .iter()
                    .map(String::as_ref)
                    .any(|path: &str| path == *expected_path)
                {
                    panic!("Missing expected path: {}", expected_path);
                }
            }
        }
    }

    lazy_static! {
        static ref SIMPLE_TEST: TestCase = TestCase::new(
            &["start-A", "start-b", "A-c", "A-b", "b-d", "A-end", "b-end",],
            &[
                "start,A,b,A,c,A,end",
                "start,A,b,A,end",
                "start,A,b,end",
                "start,A,c,A,b,A,end",
                "start,A,c,A,b,end",
                "start,A,c,A,end",
                "start,A,end",
                "start,b,A,c,A,end",
                "start,b,A,end",
                "start,b,end",
            ]
        );
        static ref LARGER_TEST: TestCase = TestCase::new(
            &[
                "dc-end", "HN-start", "start-kj", "dc-start", "dc-HN", "LN-dc",
                "HN-end", "kj-sa", "kj-HN", "kj-dc",
            ],
            &[
                "start,dc,end",
                "start,dc,HN,end",
                "start,dc,HN,kj,HN,end",
                "start,dc,kj,HN,end",
                "start,HN,dc,end",
                "start,HN,dc,HN,end",
                "start,HN,dc,HN,kj,HN,end",
                "start,HN,dc,kj,HN,end",
                "start,HN,end",
                "start,HN,kj,dc,end",
                "start,HN,kj,dc,HN,end",
                "start,HN,kj,HN,dc,end",
                "start,HN,kj,HN,dc,HN,end",
                "start,HN,kj,HN,end",
                "start,kj,dc,end",
                "start,kj,dc,HN,end",
                "start,kj,HN,dc,end",
                "start,kj,HN,dc,HN,end",
                "start,kj,HN,end",
            ]
        );
        static ref SIMPLE_TEST_VISIT_ONE_SMALL_TWICE: TestCase = TestCase::new(
            &["start-A", "start-b", "A-c", "A-b", "b-d", "A-end", "b-end",],
            &[
                "start,A,b,A,b,A,c,A,end",
                "start,A,b,A,b,A,end",
                "start,A,b,A,b,end",
                "start,A,b,A,c,A,b,A,end",
                "start,A,b,A,c,A,b,end",
                "start,A,b,A,c,A,c,A,end",
                "start,A,b,A,c,A,end",
                "start,A,b,A,end",
                "start,A,b,d,b,A,c,A,end",
                "start,A,b,d,b,A,end",
                "start,A,b,d,b,end",
                "start,A,b,end",
                "start,A,c,A,b,A,b,A,end",
                "start,A,c,A,b,A,b,end",
                "start,A,c,A,b,A,c,A,end",
                "start,A,c,A,b,A,end",
                "start,A,c,A,b,d,b,A,end",
                "start,A,c,A,b,d,b,end",
                "start,A,c,A,b,end",
                "start,A,c,A,c,A,b,A,end",
                "start,A,c,A,c,A,b,end",
                "start,A,c,A,c,A,end",
                "start,A,c,A,end",
                "start,A,end",
                "start,b,A,b,A,c,A,end",
                "start,b,A,b,A,end",
                "start,b,A,b,end",
                "start,b,A,c,A,b,A,end",
                "start,b,A,c,A,b,end",
                "start,b,A,c,A,c,A,end",
                "start,b,A,c,A,end",
                "start,b,A,end",
                "start,b,d,b,A,c,A,end",
                "start,b,d,b,A,end",
                "start,b,d,b,end",
                "start,b,end",
            ]
        );
    }

    struct TestCase {
        cave_connections: &'static [&'static str],
        sorted_expected_paths: &'static [&'static str],
    }

    impl TestCase {
        fn new(
            cave_connections: &'static [&'static str],
            sorted_expected_paths: &'static [&'static str],
        ) -> Self {
            TestCase {
                cave_connections,
                sorted_expected_paths: sorted_expected_paths,
            }
        }
    }
}
