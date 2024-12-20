use std::{
    collections::{BinaryHeap, HashSet, VecDeque},
    fmt::Display,
    path::Path,
};

use aoc::input_lines;
use clap::Parser;
use colored::Colorize;

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
    /// x/y
    position: Position,

    /// cost to move to this position from the start
    cost: usize,

    /// estimated cost from this position to the goal
    estimated_cost_to_goal: usize,

    /// Estimated cost from start to goal along this point
    estimated_total_cost: usize,

    /// previous node position to here
    prev: Option<Box<Node>>,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            estimated_total_cost: usize::MAX,
            position: Default::default(),
            cost: 0,
            estimated_cost_to_goal: usize::MAX,
            prev: None,
        }
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.estimated_cost_to_goal.cmp(&other.estimated_cost_to_goal)
            .then(self.position.cmp(&other.position))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn find_neighbors(map: &[Vec<MapEntry>], position: Position) -> Vec<Position> {
    DELTAS
        .iter()
        .filter_map(move |(dx, dy)| {
            let nx = position.x.checked_add_signed(*dx)?;
            let ny = position.y.checked_add_signed(*dy)?;
            if nx >= map.len() || ny >= map.len() {
                return None;
            }
            if !matches!(map[ny][nx], MapEntry::Open) {
                None
            } else {
                Some(Position { x: nx, y: ny })
            }
        })
        .collect()
}

fn solve_maze_using_astar(map: &[Vec<MapEntry>]) -> Option<VecDeque<Position>> {
    let mut frontier = BinaryHeap::new();
    let start_node = Node::default();
    frontier.push(start_node);
    let goal = Position {
        x: map.len() - 1,
        y: map.len() - 1,
    };
    let mut visited: HashSet<Position> = HashSet::new();

    while let Some(node) = frontier.pop() {
        let Position { x, y } = node.position;
        visited.insert(node.position);

        // Are we at the goal?
        if (x, y) == (goal.x, goal.y) {
            // compute the path
            let mut path = VecDeque::new();
            let mut next_node = Some(Box::new(node.clone()));
            while let Some(cur_node) = next_node {
                path.push_back(cur_node.position);
                next_node = cur_node.prev;
            }
            return Some(path);
        }

        for neigh_position in find_neighbors(map, node.position) {
            if visited.contains(&neigh_position) {
                continue;
            }

            let cost = node.cost + 1;
            let estimated_cost_to_goal = (goal.y - y) + (goal.x - x); //  manhattan
            let estimated_total_cost = estimated_cost_to_goal + cost;

            let prev = Box::new(node.clone());
            let neigh_node = Node {
                position: neigh_position,
                cost,
                estimated_cost_to_goal,
                estimated_total_cost,
                prev: Some(prev),
            };

            if let Some(existing_in_frontier) = frontier
                .iter()
                .filter(|n| n.position == neigh_position)
                .nth(0)
            {
                if neigh_node.cost > existing_in_frontier.cost {
                    continue; // skip this node withotu adding to frontier, dead end path to the node
                }
            }

            // add this node to the frontier in priority order (see Ord/PartialOrd)
            frontier.push(neigh_node);
        }
    }

    None
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let corruption = parse_input(cli.input)?;
    let mut map: Vec<Vec<MapEntry>> = (0..cli.dimensions)
        .map(|_y| (0..cli.dimensions).map(|_x| MapEntry::Open).collect())
        .collect();

    for pos in corruption.iter().take(cli.bytes) {
        map[pos.y][pos.x] = MapEntry::Corrupted;
    }

    let path = solve_maze_using_astar(&map).expect("Expected Solution");
    println!("Path: {:?}", path);

    for y in 0..cli.dimensions {
        for x in 0..cli.dimensions {
            let entry = map[y][x];
            let pos = Position { x, y };
            let in_path = path.contains(&pos);
            let s = match (in_path, entry) {
                (true, _) => format!("{}", 'O').blue(),
                (false, MapEntry::Open) => ".".into(),
                (false, MapEntry::Corrupted) => "x".into(),
            };
            print!("{s}");
        }
        println!("");
    }

    // The cost is the path length - 1 (the # of moves)
    println!("Cost: {}", path.len() - 1);

    Ok(())
}
