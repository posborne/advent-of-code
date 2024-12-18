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

#[derive(Debug, Clone, PartialEq, Eq)]
enum MapItem {
    Wall,
    Empty,
    Start,
    End,
    Reindeer(HashSet<Direction>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    fn idx(&self) -> usize {
        match self {
            Self::Up => 0,
            Self::Down => 1,
            Self::Left => 2,
            Self::Right => 3,
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

mod dfs {
    use super::*;

    // memoization (of a sort)
    type Cache = HashMap<(usize, usize, Direction, Direction), Option<usize>>;

    #[inline(always)]
    fn find_optimal_path_dfs_from_check_direction(
        map: &Map,
        x: usize,
        y: usize,
        direction: Direction,
        mut rudolph: Reindeer,
        cache: &mut Cache,
    ) -> Option<usize> {
        let cache_key = (x, y, rudolph.direction, direction);
        let cached = cache.get(&cache_key);
        if let Some(cached_value) = cached {
            return *cached_value;
        }

        let (nx, ny) = match direction {
            Direction::Up => (x, y - 1),
            Direction::Down => (x, y + 1),
            Direction::Left => (x - 1, y),
            Direction::Right => (x + 1, y),
        };

        // If the next spot isn't free or a place we've previously been
        // while going the same direction, this is a dead end.
        let item = map[y][x].clone();
        match item {
            MapItem::Wall => return None,
            MapItem::Reindeer(ref dirs) => {
                if dirs.contains(&direction) {
                    return None;
                }
            }
            MapItem::Empty | MapItem::Start | MapItem::End => (),
        }

        let turns = rudolph.direction.turns_to_face(direction);
        let this_cost = 1000 * turns + 1;

        let mut map = map.clone();
        let new_item = match item {
            MapItem::Reindeer(mut dirs) => {
                dirs.insert(direction);
                MapItem::Reindeer(dirs)
            }
            _ => MapItem::Reindeer(HashSet::from([direction])),
        };
        map[y][x] = new_item;
        rudolph = Reindeer {
            x: nx,
            y: ny,
            direction,
        };

        // recurse
        let value =
            find_optimal_path_dfs_from(map, rudolph, cache).map(|remcost| this_cost + remcost);
        cache.insert(cache_key, value);
        value
    }

    fn find_optimal_path_dfs_from(map: Map, rudolph: Reindeer, cache: &mut Cache) -> Option<usize> {
        if cli().animate {
            std::thread::sleep(std::time::Duration::from_millis(cli().delay_animation_ms));
            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
            println!("\n{map}");
        }

        // are we done?  If so, cool!
        if map[rudolph.y][rudolph.x] == MapItem::End {
            return Some(0);
        }

        let costs = &[
            find_optimal_path_dfs_from_check_direction(
                &map,
                rudolph.x,
                rudolph.y,
                Direction::Left,
                rudolph,
                cache,
            ),
            find_optimal_path_dfs_from_check_direction(
                &map,
                rudolph.x,
                rudolph.y,
                Direction::Right,
                rudolph,
                cache,
            ),
            find_optimal_path_dfs_from_check_direction(
                &map,
                rudolph.x,
                rudolph.y,
                Direction::Up,
                rudolph,
                cache,
            ),
            find_optimal_path_dfs_from_check_direction(
                &map,
                rudolph.x,
                rudolph.y,
                Direction::Down,
                rudolph,
                cache,
            ),
        ];

        // just bubble up the lowest cost branch, if any
        costs.iter().filter_map(|c| *c).min()
    }

    pub fn find_optimal_path_dfs(map: &Map) -> Option<usize> {
        // we're just going to go the brute force approach and try
        // it all recursively to see how that plays out before going
        // with A* or similar as it might not really be necessary.
        let rudolph = find_rudolph(map);
        let mut cache = Default::default();
        find_optimal_path_dfs_from(map.clone(), rudolph, &mut cache)
    }
}

mod dijkstra {
    use std::collections::BinaryHeap;

    use super::*;

    #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
    struct Vertex {
        x: usize,
        y: usize,
        direction: Direction,
    }

    impl Ord for Vertex {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            other
                .x
                .cmp(&self.x)
                .then(other.y.cmp(&self.y))
                .then(other.direction.idx().cmp(&self.direction.idx()))
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

    const DIRECTIONS: [Direction; 4] = [
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

    // Using Dijkstra's algorithm to find the lowest cost path
    //
    // Dijkstra's algorithm, at first blush, sounds like a great fit
    // for a maze solver.  The cost of changing directions, however, puts
    // a little wrench into things.
    //
    // The first step is to build the directed graph which we could do, but
    // we're going to try to work directly off the map structure as part of
    // this to avoid that prework to see how that treats us.
    pub fn find_optimal_path_using_dijkstra(map: &Map) -> Option<usize> {
        // dist[y][x][d] => current shortest path from start -> node (starting from a given direction)
        // There are 4 directions for each map position
        // TODO: issue - does this encode enough information about the direction we came from
        //       prior to this node?  Probably not.
        let mut dist: Vec<Vec<Vec<usize>>> = (0..map.len())
            .map(|_| (0..map[0].len()).map(|_| vec![usize::MAX; 4]).collect())
            .collect();

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

        let mut dist: HashMap<Vertex, usize> = HashMap::new();
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

            // If we've reached the end, we've found the optimal route.
            if map[y][x] == MapItem::End {
                return Some(cost);
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
    let optimal_cost = dijkstra::find_optimal_path_using_dijkstra(&map);
    println!("Optimal Path Cost: {optimal_cost:?}");
    Ok(())
}
