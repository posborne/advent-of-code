use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Deref, DerefMut},
    path::Path,
};

use aoc::input_lines;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MapItem {
    Wall,
    Empty,
    Start,
    End,
    Reindeer(Direction),
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
            Self::Reindeer(dir) => dir.as_char(),
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
            '>' => Self::Reindeer(Direction::Right),
            'v' => Self::Reindeer(Direction::Down),
            '<' => Self::Reindeer(Direction::Left),
            '^' => Self::Reindeer(Direction::Up),
            c => panic!("Unknown char {c}"),
        }
    }
}

impl Display for MapItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_char())
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

// memoization (of a sort)
type Cache = HashMap<(usize, usize, Direction, Direction), Option<usize>>;

#[inline(always)]
fn check_direction(
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

    if !matches!(map[ny][nx], MapItem::Empty | MapItem::End) {
        return None; // nothing pushed into costs
    }

    let turns = rudolph.direction.turns_to_face(direction);
    let this_cost = 1000 * turns + 1;

    let mut map = map.clone();
    map[y][x] = MapItem::Reindeer(direction);
    rudolph = Reindeer {
        x: nx,
        y: ny,
        direction,
    };

    // recurse
    let value = find_optimal_path_from(map, rudolph, cache).map(|remcost| this_cost + remcost);
    cache.insert(cache_key, value);
    value
}

fn find_optimal_path_from(map: Map, rudolph: Reindeer, cache: &mut Cache) -> Option<usize> {
    // are we done?  If so, cool!
    if map[rudolph.y][rudolph.x] == MapItem::End {
        return Some(0);
    }

    let costs = &[
        check_direction(&map, rudolph.x, rudolph.y, Direction::Left, rudolph, cache),
        check_direction(&map, rudolph.x, rudolph.y, Direction::Right, rudolph, cache),
        check_direction(&map, rudolph.x, rudolph.y, Direction::Up, rudolph, cache),
        check_direction(&map, rudolph.x, rudolph.y, Direction::Down, rudolph, cache),
    ];

    // just bubble up the lowest cost branch, if any
    costs.iter().filter_map(|c| *c).min()
}

fn find_optimal_path(map: &Map) -> Option<usize> {
    // we're just going to go the brute force approach and try
    // it all recursively to see how that plays out before going
    // with A* or similar as it might not really be necessary.
    let rudolph = find_rudolph(map);
    let mut cache = Default::default();
    find_optimal_path_from(map.clone(), rudolph, &mut cache)
}

fn main() -> anyhow::Result<()> {
    let map = parse_input("d16.txt")?;
    let optimal_cost = find_optimal_path(&map).expect("Expected a solution");
    println!("Optimal Path Cost: {optimal_cost}");
    Ok(())
}
