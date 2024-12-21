use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    fmt::Display,
    path::Path,
};

use aoc::input_lines;
use clap::Parser;
use itertools::Itertools;

#[derive(Debug, Clone, Copy)]
enum MapEntry {
    Start,
    End,
    Road,
    Wall,
}

impl Display for MapEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Self::Start => 'S',
            Self::End => 'E',
            Self::Road => '.',
            Self::Wall => '#',
        };
        write!(f, "{c}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    x: usize,
    y: usize,
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.x.cmp(&other.x).then(self.y.cmp(&other.y))
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

#[derive(Debug, Clone)]
struct Map {
    entries: Vec<Vec<MapEntry>>,
    start: Position,
    end: Position,
}

fn parse_input<P: AsRef<Path>>(path: P) -> anyhow::Result<Map> {
    let mut entries: Vec<Vec<MapEntry>> = Default::default();
    let mut start = Position { x: 0, y: 0 };
    let mut end = Position { x: 0, y: 0 };
    for (y, line) in input_lines(path)?.into_iter().enumerate() {
        entries.push(Vec::new());
        for (x, c) in line.chars().enumerate() {
            let entry = match c {
                'S' => {
                    start.x = x;
                    start.y = y;
                    MapEntry::Start
                }
                'E' => {
                    end.x = x;
                    end.y = y;
                    MapEntry::End
                }
                '.' => MapEntry::Road,
                '#' => MapEntry::Wall,
                _ => panic!("Unexpected input char {c}"),
            };
            entries[y].push(entry);
        }
    }

    let map = Map {
        entries,
        start,
        end,
    };
    Ok(map)
}

fn print_map(map: &Map) {
    for row in &map.entries {
        for entry in row {
            print!("{entry}");
        }
        println!("");
    }
}

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    input: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Cheat {
    start: Position,
    end: Position,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Node {
    position: Position,
    cost: usize,
    cheat: Option<Cheat>,
}

const DELTAS: [(isize, isize); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];

fn next_position(
    x: usize,
    y: usize,
    dx: isize,
    dy: isize,
    rows: usize,
    cols: usize,
) -> Option<(usize, usize)> {
    let nx = x.checked_add_signed(dx)?;
    let ny = y.checked_add_signed(dy)?;
    if nx >= rows || ny >= cols {
        None
    } else {
        Some((nx, ny))
    }
}

impl Node {
    fn new(x: usize, y: usize) -> Self {
        Node {
            position: Position { x, y },
            cost: 0,
            cheat: None,
        }
    }

    fn neighbors(&self, map: &Map) -> Vec<Node> {
        let cols = map.entries.len();
        let rows = map.entries[0].len();

        // just return neighbors that don't involve collisions
        if self.cheat.is_some() {
            return DELTAS
                .into_iter()
                .filter_map(|(dx, dy)| {
                    let (x, y) =
                        next_position(self.position.x, self.position.y, dx, dy, rows, cols)?;
                    Some(Node {
                        position: Position { x, y },
                        cost: self.cost + 1,
                        cheat: self.cheat.clone(),
                    })
                })
                .collect();
        }

        // we haven't done collissions yet, so we need to provide both paths that
        // don't involve collisions and ones that do.
        DELTAS
            .into_iter()
            .filter_map(|(dx, dy)| {
                let (x, y) = next_position(self.position.x, self.position.y, dx, dy, rows, cols)?;
                if matches!(map.entries[y][x], MapEntry::Wall) {
                    let (nx, ny) = next_position(x, y, dx, dy, rows, cols)?;
                    if matches!(map.entries[ny][nx], MapEntry::Wall) {
                        None
                    } else {
                        Some(Node {
                            position: Position { x: nx, y: ny },
                            cost: self.cost + 2,
                            cheat: Some(Cheat {
                                start: Position { x, y },
                                end: Position { x: nx, y: ny },
                            }),
                        })
                    }
                } else {
                    Some(Node {
                        position: Position { x, y },
                        cost: self.cost + 1,
                        cheat: self.cheat.clone(),
                    })
                }
            })
            .collect()
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cost
            .cmp(&other.cost)
            .then(self.position.cmp(&other.position))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

fn part1() -> anyhow::Result<()> {
    // Part 1 Reasoning:
    //
    // Off the bat, my first idea is to model things using Dijkstra's
    // algorithm with the behavior of what is considered to be a neighbor
    // changing only after the path consumes its two collision disables.
    //
    // A* wouldn't really work as I don't think we can come up with a
    // good heuristic, so (again) my first thought is to go back to a
    // form of dijkstra's modified to try to track the notion of
    // having cheated in our path with differences in enighbor computation
    // before and after having done a cheat on this pass.
    let cli = Cli::parse();
    let map = parse_input(cli.input)?;
    print_map(&map);

    let mut frontier = BinaryHeap::new();
    let start_node = Node::new(map.start.x, map.start.y);
    frontier.push(start_node);

    let mut solutions: Vec<Node> = Vec::new();
    let mut visited: HashSet<(Position, Option<Cheat>)> = HashSet::new();
    while let Some(node) = frontier.pop() {
        // Are we at the end?  Then we have a complete path, record it and
        // carry on
        if map.end == node.position {
            solutions.push(node);
            continue;
        }

        let neighbors = node.neighbors(&map);
        for neighbor in neighbors {
            if visited.contains(&(neighbor.position, neighbor.cheat.clone())) {
                continue;
            }

            visited.insert((neighbor.position, neighbor.cheat.clone()));
            frontier.push(neighbor);
        }
    }

    let full_length = map
        .entries
        .iter()
        .flat_map(|r| {
            r.iter()
                .filter(|e| matches!(e, MapEntry::Road | MapEntry::End))
        })
        .count();
    println!("Full Cost: {full_length}");
    let mut solutions_by_cost: HashMap<usize, usize> = HashMap::new();
    for node in solutions {
        let saved = full_length - node.cost;
        let entry = solutions_by_cost.entry(saved).or_default();
        *entry += 1;
    }

    for (cost, solutions) in solutions_by_cost.iter().sorted() {
        println!("{cost}: {solutions}");
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    part1()?;
    Ok(())
}
