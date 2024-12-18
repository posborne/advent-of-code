use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    ops::{Deref, DerefMut},
    path::Path,
    sync::OnceLock,
};

use aoc::input_lines;
use clap::Parser;
use colored::Colorize;
use dijkstra::{Vertex, DIRECTIONS};

#[derive(Debug, Clone, PartialEq, Eq)]
enum MapItem {
    Wall,
    Empty,
    Start,
    End,
    Reindeer(HashSet<Direction>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Reindeer {
    x: usize,
    y: usize,
    direction: Direction,
}

impl Direction {
    fn as_char(&self) -> char {
        match self {
            Self::Up => '^',
            Self::Down => 'v',
            Self::Left => '<',
            Self::Right => '>',
        }
    }

    fn dx_dy(&self) -> (isize, isize) {
        match self {
            Self::Up => (0, -1),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
        }
    }

    fn opposite_direction(&self) -> Direction {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    fn turns_to_face(&self, other: Direction) -> usize {
        if self == &other {
            0
        } else if self.opposite_direction() == other {
            2
        } else {
            1
        }
    }
}

impl MapItem {
    fn as_char(&self) -> char {
        match self {
            Self::Wall => '#',
            Self::Empty => '.',
            Self::Start => 'S',
            Self::End => 'E',
            Self::Reindeer(dirs) => match dirs.len() {
                1 => dirs.iter().nth(0).unwrap().as_char(),
                _ => '+',
            },
        }
    }
}

impl From<char> for MapItem {
    fn from(c: char) -> Self {
        match c {
            '#' => Self::Wall,
            '.' => Self::Empty,
            'S' => Self::Start,
            'E' => Self::End,
            '>' => Self::Reindeer(HashSet::from([Direction::Right])),
            'v' => Self::Reindeer(HashSet::from([Direction::Down])),
            '<' => Self::Reindeer(HashSet::from([Direction::Left])),
            '^' => Self::Reindeer(HashSet::from([Direction::Up])),
            c => panic!("Unknown char {c}"),
        }
    }
}

impl Display for MapItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = self.as_char();
        match self {
            Self::Reindeer(_) => write!(f, "{}", format!("{c}").blue()),
            _ => write!(f, "{c}"),
        }
    }
}

#[derive(Debug, Clone)]
struct Map(Vec<Vec<MapItem>>);

impl DerefMut for Map {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for Map {
    type Target = Vec<Vec<MapItem>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.0.iter() {
            for item in row {
                write!(f, "{item}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

fn parse_input<P: AsRef<Path>>(path: P) -> anyhow::Result<Map> {
    let map: Vec<Vec<MapItem>> = input_lines(path)?
        .map(|line| line.chars().map(|c| MapItem::from(c)).collect())
        .collect();

    Ok(Map(map))
}

fn find_rudolph(map: &Map) -> Reindeer {
    let rudolph = map
        .iter()
        .enumerate()
        .find_map(|(y, row)| {
            row.iter().enumerate().find_map(|(x, item)| {
                if *item == MapItem::Start {
                    Some(Reindeer {
                        x,
                        y,
                        direction: Direction::Right, // east
                    })
                } else {
                    None
                }
            })
        })
        .expect("Map should have a start");
    rudolph
}

mod dijkstra {
    use std::{collections::BinaryHeap, hash::Hash};

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Vertex {
        pub x: usize,
        pub y: usize,
        pub direction: Direction,
    }

    impl Ord for Vertex {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.x.cmp(&other.x)
                .then_with(|| self.y.cmp(&other.y))
                .then_with(|| self.direction.cmp(&other.direction))
        }
    }

    impl PartialOrd for Vertex {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(&other))
        }
    }

    struct Edge {
        next_position: Vertex,
        cost: usize,
    }

    pub const DIRECTIONS: [Direction; 4] = [
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ];

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    struct State {
        position: Vertex,
        cost: usize,
    }

    impl Ord for State {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            other
                .cost
                .cmp(&self.cost)
                .then_with(|| self.position.cmp(&other.position))
        }
    }

    impl PartialOrd for State {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(&other))
        }
    }

    fn build_adjancy_map(map: &Map) -> HashMap<Vertex, Vec<Edge>> {
        let mut adjacencies: HashMap<Vertex, Vec<Edge>> = HashMap::new();
        for (y, row) in map.iter().enumerate() {
            for (x, val) in row.iter().enumerate() {
                if matches!(val, MapItem::Empty | MapItem::Start | MapItem::End) {
                    for cur_direction in DIRECTIONS {
                        let mut adjacent = Vec::with_capacity(4);
                        for move_direction in DIRECTIONS {
                            let (dx, dy) = move_direction.dx_dy();
                            let (nx, ny) = ((x as isize + dx) as usize, (y as isize + dy) as usize);
                            let nmap = &map[ny][nx];
                            if *nmap == MapItem::Wall {
                                continue; // not really an edge here
                            }
                            let number_turns_to_face = cur_direction.turns_to_face(move_direction);
                            let edge_cost = number_turns_to_face * 1000 + 1;
                            let edge = Edge {
                                next_position: Vertex {
                                    x: nx,
                                    y: ny,
                                    direction: move_direction,
                                },
                                cost: edge_cost,
                            };
                            adjacent.push(edge);
                        }
                        let this_vertex = Vertex {
                            x,
                            y,
                            direction: cur_direction,
                        };
                        adjacencies.insert(this_vertex, adjacent);
                    }
                }
            }
        }
        adjacencies
    }

    // Using Dijkstra's algorithm to find the lowest cost path
    //
    // Dijkstra's algorithm, at first blush, sounds like a great fit
    // for a maze solver.  The cost of changing directions, however, puts
    // a little wrench into things.
    //
    // For our input problem we model the maze in 3d space where our dimensions
    // are:
    // - y: column
    // - x: row
    // - z: The direction we are facing when we moved to the vertex
    //
    // The weight on the edges between adjacent nodes in the graph are the associated
    // cost which will be 1 plus the 1000 * number of turns to get to the required z
    // value for the node.  We include the modeling of a direct move backwards (though
    // we could safely preclude this case) as this will always have a cost of 2002 and
    // wouldn't ever realistically be selected.
    pub fn find_optimal_path_using_dijkstra(
        map: &Map,
    ) -> Option<(usize, HashMap<Vertex, Vertex>, Vertex)> {
        let adjacencies = build_adjancy_map(map);
        let mut dist: HashMap<Vertex, usize> = HashMap::new();
        let mut prev: HashMap<Vertex, Vertex> = HashMap::new();
        let mut pq = BinaryHeap::new();

        for vertex in adjacencies.keys() {
            dist.insert(*vertex, usize::MAX);
        }

        let rudolph = find_rudolph(map);
        let rudolph_position = Vertex {
            x: rudolph.x,
            y: rudolph.y,
            direction: rudolph.direction,
        };
        dist.insert(rudolph_position, 0);
        pq.push(State {
            position: rudolph_position,
            cost: 0,
        });

        // examine the "frontier" with lowest cost nodes first
        while let Some(State { position, cost }) = pq.pop() {
            let Vertex { x, y, .. } = position;

            // If we've reached the end, we've found an optimal route; for part
            // 2 we want to find the count of spots along any of the optimal
            // routes.
            if map[y][x] == MapItem::End {
                return Some((cost, prev, position));
            }

            // If we've found a better way, don't use this one
            if cost > dist[&position] {
                continue;
            }

            // for each adjacent node (which we can find out by consulting the map),
            // see if there's a lower cost route.
            for edge in adjacencies[&position].iter() {
                let next = State {
                    position: edge.next_position,
                    cost: edge.cost + cost,
                };

                if next.cost < dist[&next.position] {
                    pq.push(next);
                    dist.insert(next.position, next.cost);
                    prev.insert(next.position, position);
                }
            }
        }

        // Not reachable
        None
    }
}

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short, long, default_value = "d16.txt")]
    input: String,

    #[arg(short, long, action)]
    animate: bool,

    #[arg(short, long, default_value_t = 5)]
    delay_animation_ms: u64,
}

fn cli() -> &'static Cli {
    static CLI: OnceLock<Cli> = OnceLock::new();
    CLI.get_or_init(|| Cli::parse())
}

fn main() -> anyhow::Result<()> {
    let map = parse_input(&cli().input)?;
    let (optimal_cost, prev_map, finish) =
        dijkstra::find_optimal_path_using_dijkstra(&map).unwrap();
    let mut cur = Some(finish);
    let mut path: Vec<Vertex> = Vec::new();
    while let Some(v) = cur {
        path.push(v);
        cur = prev_map.get(&v).cloned();
    }

    for (y, row) in map.iter().enumerate() {
        for (x, entry) in row.iter().enumerate() {
            let directions: Vec<Direction> = DIRECTIONS
                .iter()
                .filter(|&&d| {
                    let key = Vertex { x, y, direction: d };
                    path.contains(&key)
                })
                .cloned()
                .collect();
            match directions.len() {
                0 => print!("{entry}"),
                1 => print!("{}", format!("{}", directions[0].as_char()).blue()),
                _ => print!("{}", format!("+").red()),
            }
        }
        println!("");
    }

    println!("Optimal Path Cost: {optimal_cost:?}");
    Ok(())
}
