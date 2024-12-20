use std::{
    collections::{BinaryHeap, HashSet},
    fmt::Display,
    path::Path,
};

use aoc::input_lines;
use clap::Parser;

#[derive(Debug, Copy, Clone)]
enum MapEntry {
    Open,
    Corrupted,
}

impl Display for MapEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Open => '.',
                Self::Corrupted => '#',
            }
        )
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    x: usize,
    y: usize,
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.y.cmp(&other.y).then(self.x.cmp(&other.x))
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn parse_input<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<Position>> {
    let corruption: Vec<Position> = input_lines(path)?
        .map(|line| {
            let (x, y) = line.split_once(',').expect("Expected comma");
            Position {
                x: x.parse::<usize>().unwrap(),
                y: y.parse::<usize>().unwrap(),
            }
        })
        .collect();
    Ok(corruption)
}

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short, long, default_value = "d18.txt")]
    input: String,

    #[arg(short, long, default_value_t = 70)]
    dimensions: usize,

    #[arg(short, long, default_value_t = 1024)]
    bytes: usize,
}

const DELTAS: [(isize, isize); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];

#[derive(Debug, Clone, PartialEq, Eq)]
struct Node {
    // x/y
    position: Position,
    // cost to move to this position from the start
    cost: usize,
    // estimated cost from this position to the goal
    estimated_remaining_cost: usize,
    // estimate total cost (if computed)
    estimated_total_cost: usize,
    // previous node position
    prev: Option<Position>,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            estimated_total_cost: usize::MAX,
            position: Default::default(),
            cost: 0,
            estimated_remaining_cost: 0,
        }
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
        Some(self.cmp(other))
    }
}

fn init_2darray<T: Default>(cols: usize, rows: usize) -> Vec<Vec<T>> {
    (0..cols)
        .map(|_| (0..rows).map(|_| Default::default()).collect())
        .collect()
}

fn solve_maze_using_astar(map: &[Vec<MapEntry>]) -> Option<Node> {
    let mut open_set = BinaryHeap::new();
    let start_node = Node::default();
    open_set.push(start_node);
    let goal = Position {
        x: map.len() - 1,
        y: map.len() - 1,
    };

    let mut nodes: Vec<Vec<Node>> = (0..map.len())
        .map(|y| {
            (0..map.len())
                .map(|x| Node {
                    position: Position { x, y },
                    ..Default::default()
                })
                .collect()
        })
        .collect();
    let mut visited = init_2darray::<bool>(map.len(), map.len());

    while let Some(Node {
        position: Position { x, y },
        cost,
        ..
    }) = open_set.pop()
    {
        println!("Visiting y={y}, x={x}");
        visited[y][x] = true;

        let successors = DELTAS.iter().filter_map(|(dx, dy)| {
            let nx = x.checked_add_signed(*dx)?;
            let ny = y.checked_add_signed(*dy)?;
            if nx >= map.len() || ny >= map.len() {
                return None;
            }
            if !matches!(map[ny][nx], MapEntry::Open) {
                None
            } else {
                Some((nx, ny))
            }
        });

        for (nx, ny) in successors {
            println!("ny={ny}, nx={nx}");
            let node = &mut nodes[ny][nx];

            if (nx, ny) == (goal.x, goal.y) {
                node.cost = cost + 1;
                return Some(node.clone());
            }

            // E.g. "g"
            let ncost = 1 + cost;

            // Since we can only move in 4 directions, we want to use
            // the manhattan distance here which is just the sum
            // of the differences between points and goal on our grid.
            //
            // E.g. "h"
            let estimated_cost_to_goal = {
                // we know goal is >= nx/ny so abs not required.
                (goal.y - ny) + (goal.x - nx)
            };

            // e.g. "f"
            let estimated_total_cost = ncost + estimated_cost_to_goal;

            if node.estimated_total_cost == usize::MAX
                || estimated_total_cost > node.estimated_total_cost && !visited[ny][nx]
            {
                node.estimated_total_cost = estimated_total_cost;
                node.cost = ncost;
                node.estimated_remaining_cost = estimated_cost_to_goal;
                node.prev = Some(Position {
                    x, y
                });
                open_set.push(node.clone());
            }
        }
    }

    None
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let corruption = parse_input(cli.input)?;
    let mut map_with_corruption: Vec<Vec<MapEntry>> = (0..cli.dimensions)
        .map(|_y| (0..cli.dimensions).map(|_x| MapEntry::Open).collect())
        .collect();

    for pos in corruption.iter().take(cli.bytes) {
        map_with_corruption[pos.y][pos.x] = MapEntry::Corrupted;
    }

    let mut node = solve_maze_using_astar(&map_with_corruption).expect("Expected Solution");
    println!("Node: {:?}", node);
    println!("Cost: {}", node.cost);

    let mut positions: HashSet<Position> = HashSet::new();
    while let Some(next) = node.prev {
        positions.insert(next);
    }

    Ok(())
}
