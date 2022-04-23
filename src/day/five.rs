use std::path::PathBuf;

use anyhow::{Context, Result};
use regex::Regex;
use structopt::{self, StructOpt};

use super::read_lines;

#[derive(Debug, StructOpt)]
pub struct Command {
    #[structopt(required(true), parse(from_os_str))]
    input: PathBuf,
}

impl Command {
    pub fn run(&self) -> Result<()> {
        let lines =
            parse_lines(read_lines(&self.input)?.iter().map(String::as_ref))?;
        let lines: Vec<Line> = lines
            .into_iter()
            .filter(|line| {
                line.is_horizontal() || line.is_vertical() || line.is_diagonal()
            })
            .collect();
        let extents = lines
            .iter()
            .skip(1)
            .fold(lines[0].extents(), |extents, line| {
                extents.union(&line.extents())
            });
        let mut grid = Grid::new(&extents);
        for line in lines {
            grid.apply(line);
        }
        let dangerous_sector_count =
            grid.sectors.iter().fold(0, |count, row| {
                row.iter().fold(count, |count, sector| {
                    if *sector >= 2 {
                        count + 1
                    } else {
                        count
                    }
                })
            });
        println!("Sectors with two or more vents: {}", dangerous_sector_count);

        Ok(())
    }
}

fn parse_lines<'a, Iter>(lines: Iter) -> Result<Vec<Line>>
where
    Iter: Iterator<Item = &'a str>,
{
    let mut parsed_lines = Vec::new();
    let regex = Regex::new(
        r"^(?P<x1>\d+)\s*,\s*(?P<y1>\d+)\s*->\s*(?P<x2>\d+)\s*,\s*(?P<y2>\d+)$",
    )
    .with_context(|| "create regex to parse lines")?;
    for line in lines {
        let capture = regex
            .captures(line)
            .with_context(|| format!("failed to parse '{}'", line))?;
        let mut coords = Vec::new();
        for key in ["x1", "y1", "x2", "y2"] {
            let point_text = capture
                .name(key)
                .with_context(|| format!("missing key '{}'", key))?
                .as_str();
            let point = point_text.parse().with_context(|| {
                format!("failed to parse point '{}'", point_text)
            })?;
            coords.push(point);
        }
        parsed_lines.push(Line(
            Point {
                x: coords[0],
                y: coords[1],
            },
            Point {
                x: coords[2],
                y: coords[3],
            },
        ));
    }

    Ok(parsed_lines)
}

#[derive(Debug)]
struct Grid {
    origin: Point,
    sectors: Vec<Vec<usize>>,
}

impl Grid {
    fn new(extents: &Extents) -> Self {
        Grid {
            origin: Point {
                x: extents.left,
                y: extents.top,
            },
            sectors: vec![vec!(0; extents.width()); extents.heigth()],
        }
    }

    fn apply(&mut self, line: Line) {
        for point in line.path() {
            self.sectors[point.y - self.origin.y][point.x - self.origin.y] += 1;
        }
    }
}

/// Extents specifies the minimum area two points are contained within.
/// The area is defined as top-left to bottom-right where top-left is closests
/// to the origin and bottom-right is the furthest from the origin.
#[derive(Copy, Clone, Debug)]
struct Extents {
    top: usize,
    left: usize,
    bottom: usize,
    right: usize,
}

impl Extents {
    fn new(p1: Point, p2: Point) -> Self {
        Extents {
            top: p1.y.min(p2.y),
            left: p1.x.min(p2.x),
            bottom: p1.y.max(p2.y),
            right: p1.x.max(p2.x),
        }
    }

    fn heigth(&self) -> usize {
        self.bottom - self.top + 1
    }

    fn width(&self) -> usize {
        self.right - self.left + 1
    }

    fn union(&self, other: &Extents) -> Extents {
        Extents {
            top: self.top.min(other.top),
            left: self.left.min(other.left),
            bottom: self.bottom.max(other.bottom),
            right: self.right.max(other.right),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Line(Point, Point);

/// Line may be 0, 45 or 90 degrees.
impl Line {
    fn is_horizontal(&self) -> bool {
        self.0.y == self.1.y
    }

    fn is_vertical(&self) -> bool {
        self.0.x == self.1.x
    }

    fn is_diagonal(&self) -> bool {
        self.0.x.max(self.1.x) - self.0.x.min(self.1.x)
            == self.0.y.max(self.1.y) - self.0.y.min(self.1.y)
    }

    fn extents(&self) -> Extents {
        Extents::new(self.0, self.1)
    }

    fn path(&self) -> Vec<Point> {
        let step_forward: &dyn Fn(usize) -> usize =
            &|n: usize| -> usize { n + 1 };
        let step_back: &dyn Fn(usize) -> usize = &|n: usize| -> usize { n - 1 };
        let step_none: &dyn Fn(usize) -> usize = &|n: usize| -> usize { n };
        let (x_step, y_step) = if self.is_horizontal() {
            (
                if self.0.x <= self.1.x {
                    step_forward
                } else {
                    step_back
                },
                step_none,
            )
        } else if self.is_vertical() {
            (
                step_none,
                if self.0.y <= self.1.y {
                    step_forward
                } else {
                    step_back
                },
            )
        } else {
            assert!(self.is_diagonal());
            (
                if self.0.x <= self.1.x {
                    step_forward
                } else {
                    step_back
                },
                if self.0.y <= self.1.y {
                    step_forward
                } else {
                    step_back
                },
            )
        };
        let mut path = Vec::new();
        let mut point = self.0;
        loop {
            path.push(point);
            if point == self.1 {
                break;
            }
            point.x = x_step(point.x);
            point.y = y_step(point.y);
        }
        path
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, PartialOrd)]
struct Point {
    x: usize,
    y: usize,
}

#[cfg(test)]
mod tests {
    use super::{parse_lines, Line, Point};

    #[test]
    fn parse_lines_test() {
        match parse_lines(["0,9 -> 5,9"].into_iter()) {
            Ok(lines) => {
                assert_eq!(lines.len(), 1);
                assert_eq!(
                    lines[0],
                    Line(Point { x: 0, y: 9 }, Point { x: 5, y: 9 })
                );
            }
            Err(err) => eprintln!("{:?}", err),
        }
    }

    #[test]
    fn horizontal_line_path_test() {
        let line = Line(Point { x: 0, y: 0 }, Point { x: 0, y: 5 });

        let path = line.path();

        assert_eq!(
            path,
            vec!(
                Point { x: 0, y: 0 },
                Point { x: 0, y: 1 },
                Point { x: 0, y: 2 },
                Point { x: 0, y: 3 },
                Point { x: 0, y: 4 },
                Point { x: 0, y: 5 },
            )
        )
    }

    #[test]
    fn vertical_line_path_test() {
        let line = Line(Point { x: 5, y: 10 }, Point { x: 10, y: 10 });

        let path = line.path();

        assert_eq!(
            path,
            vec!(
                Point { x: 5, y: 10 },
                Point { x: 6, y: 10 },
                Point { x: 7, y: 10 },
                Point { x: 8, y: 10 },
                Point { x: 9, y: 10 },
                Point { x: 10, y: 10 },
            )
        )
    }

    #[test]
    fn short_reverse_path_test() {
        let line = Line(Point { x: 2, y: 2 }, Point { x: 2, y: 1 });

        let path = line.path();

        assert_eq!(path, vec!(Point { x: 2, y: 2 }, Point { x: 2, y: 1 }));
    }

    #[test]
    fn diagonal_path_test() {
        let line = Line(Point { x: 1, y: 1 }, Point { x: 3, y: 3 });

        let path = line.path();

        assert_eq!(
            path,
            vec!(
                Point { x: 1, y: 1 },
                Point { x: 2, y: 2 },
                Point { x: 3, y: 3 }
            )
        );
    }
}
