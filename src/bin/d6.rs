// If there is something directly in front of you, turn right 90 degrees.
// Otherwise, take a step forward.

// How many distinct positions will the guard visit before leaving the mapped area?

use std::{
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use enumset::{EnumSet, EnumSetType};

#[derive(Debug, EnumSetType)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl TryFrom<char> for Direction {
    type Error = String;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        let dir = match value {
            '^' => Self::Up,
            'v' => Self::Down,
            '>' => Self::Right,
            '<' => Self::Left,
            other => {
                return Err(format!("'{other}' not an expected direction input"));
            }
        };
        Ok(dir)
    }
}

impl Direction {
    fn as_char(&self) -> char {
        match *self {
            Direction::Up => '^',
            Direction::Down => 'v',
            Direction::Left => '<',
            Direction::Right => '>',
        }
    }
}

#[derive(Debug, Clone)]
enum MapPosition {
    Empty,
    Obstacle,
    Visited(EnumSet<Direction>),
    Guard(Direction),
}

type Map = Vec<Vec<MapPosition>>;

impl TryFrom<char> for MapPosition {
    type Error = String;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        let pos = match value {
            '.' => Self::Empty,
            '#' => Self::Obstacle,
            // assumption: visited will not be present in inputs
            guard => Self::Guard(Direction::try_from(guard)?),
        };
        Ok(pos)
    }
}

impl Display for MapPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match *self {
            Self::Empty => '.',
            Self::Obstacle => '#',
            Self::Visited(dirset) => {
                let mapping: &[(EnumSet<Direction>, char)] = &[
                    (EnumSet::from(Direction::Up), '^'),
                    (EnumSet::from(Direction::Down), 'v'),
                    (EnumSet::from(Direction::Left), '<'),
                    (EnumSet::from(Direction::Right), '>'),
                    ((Direction::Up | Direction::Down), '|'),
                    ((Direction::Left | Direction::Right), '-'),
                    ((Direction::Up | Direction::Right), 'L'),
                    ((Direction::Up | Direction::Left), '/'), // just making stuff up now
                    ((Direction::Down | Direction::Right), '%'),
                    ((Direction::Down | Direction::Left), '!'),
                ];

                mapping
                    .iter()
                    .find_map(|(s, c)| if dirset == *s { Some(*c) } else { None })
                    .unwrap_or_else(|| '+')
            }
            Self::Guard(direction) => direction.as_char(),
        };
        write!(f, "{}", c)
    }
}

fn parse_input<P: AsRef<Path>>(path: P) -> anyhow::Result<Map> {
    let full_path = PathBuf::from(".")
        .join("inputs")
        .join(path);
    let f = File::open(full_path)?;
    let reader = BufReader::new(f);
    let map = reader
        .lines()
        .into_iter()
        .filter_map(|l| l.ok())
        .map(|l| {
            l.chars()
                .map(|c| {
                    MapPosition::try_from(c).unwrap_or_else(|e| {
                        panic!("Unexpected parse error on {e:?}");
                    })
                })
                .collect::<Vec<MapPosition>>()
        })
        .collect::<Vec<Vec<MapPosition>>>();
    Ok(map)
}

fn simulate_movements(orig_map: &Map) -> Option<Map> {
    // find guard position
    let mut map = orig_map.clone();
    struct GuardPosition {
        row: usize,
        col: usize,
    }
    let Some((mut guard_position, mut guard_direction)) =
        map.iter().enumerate().find_map(|(row_idx, row)| {
            row.iter()
                .enumerate()
                .find_map(|(col_idx, pos)| match pos {
                    MapPosition::Guard(dir) => Some((col_idx, *dir)),
                    _ => None,
                })
                .map(|(col_idx, dir)| {
                    let pos = GuardPosition {
                        row: row_idx,
                        col: col_idx,
                    };
                    (pos, dir)
                })
        })
    else {
        panic!("No guard position found in map input!");
    };

    // perform the walk, updating our map until we leave the map area
    let row_count = map.len();
    let col_count = map[0].len();
    map[guard_position.row][guard_position.col] =
        MapPosition::Visited(EnumSet::from(guard_direction));
    loop {
        let (delta_row, delta_col) = match guard_direction {
            Direction::Up => (-1, 0),
            Direction::Down => (1, 0),
            Direction::Left => (0, -1),
            Direction::Right => (0, 1),
        };

        let (next_row, next_col) = match (
            guard_position.row.checked_add_signed(delta_row),
            guard_position.col.checked_add_signed(delta_col),
        ) {
            (Some(r), Some(c)) if c < col_count && r < row_count => (r as usize, c as usize),
            _ => break, // the guard has left the building
        };

        let next_map_element = &mut map[next_row][next_col];
        match next_map_element {
            MapPosition::Visited(dirset) => {
                // if we've already visited this position in the same direction, then
                // we have a cycle.  Exit with 'None' as a sentinel for a cycle.
                if dirset.contains(guard_direction) {
                    return None;
                }

                // add this direction to the set
                *next_map_element = MapPosition::Visited(*dirset | guard_direction);
                guard_position.row = next_row;
                guard_position.col = next_col;
            }
            MapPosition::Empty => {
                // mark next spot as visited and put the guard in this pos
                *next_map_element = MapPosition::Visited(EnumSet::from(guard_direction));
                guard_position.row = next_row;
                guard_position.col = next_col;
            }
            MapPosition::Obstacle => {
                // change direction guard is facing but the guard
                // doesn't move this pass.
                guard_direction = match guard_direction {
                    Direction::Up => Direction::Right,
                    Direction::Right => Direction::Down,
                    Direction::Down => Direction::Left,
                    Direction::Left => Direction::Up,
                }
            }
            MapPosition::Guard(_) => panic!("Go home guard, you're drunk"),
        }
    }

    // return back the map we mutated in place
    Some(map)
}

fn print_map(map: &Map) {
    for row in map {
        for pos in row {
            print!("{pos}");
        }
        println!("");
    }
}

fn positions_visited(map: &Map) -> usize {
    map.iter()
        .map(|r| r.iter().filter(|&p| matches!(*p, MapPosition::Visited(_))))
        .flatten()
        .count()
}

fn find_single_obstacle_positions(
    orig_map: &Map,
    map_with_visits: &Map,
) -> Vec<(usize, usize, Map)> {
    // TODO: find the number of single obstacles we could place
    //       into the map to cause the guard to get stuck
    //       indefinitely.
    //
    // Initial reasoning:
    //
    // There's probably a clever solution here, but it seems like
    // brute force will probably still work here.  We've got two
    // problems to solve:
    //
    // 1. Determine halting: If we ever have a position where the
    //    guard passes through a second time going the same direction
    //    we've got a cycle.  Alternatively, we could go dirty and
    //    just call it at a number of iteractions.  A BitSet for visiting
    //    a position should be easy enough, however.
    // 2. Starting with the successful run, we can just place obstacles
    //    to block the cardinal direction of a move and test each of
    //    those.

    let mut single_obstacle_positions: Vec<(usize, usize, Map)> = Vec::new();
    let visited_positions = map_with_visits
        .iter()
        .enumerate()
        .map(|(ridx, r)| {
            r.iter().enumerate().filter_map(move |(cidx, c)| {
                if matches!(*c, MapPosition::Visited(_)) {
                    Some((ridx, cidx))
                } else {
                    None
                }
            })
        })
        .flatten()
        .collect::<Vec<(usize, usize)>>();

    for (row, col) in visited_positions {
        // create a map with each position visited having an obstacle
        // and see if we end up with a cycle when simulated
        let mut map = orig_map.clone();
        if matches!(map[row][col], MapPosition::Guard(_)) {
            continue; // special case
        }

        map[row][col] = MapPosition::Obstacle;
        let res = simulate_movements(&map);
        if res.is_none() {
            single_obstacle_positions.push((row, col, map));
        }
    } 

    single_obstacle_positions
}

fn main() -> anyhow::Result<()> {
    let orig_map = parse_input("d6-p1.txt")?;
    print_map(&orig_map);
    let map_with_visits =
        simulate_movements(&orig_map).expect("Base map unexpectedly simulated a cycle");
    print_map(&map_with_visits);
    let visited = positions_visited(&map_with_visits);
    println!("Positions Visited: {visited}");

    println!("");
    println!("");
    let obstacle_sim_results = find_single_obstacle_positions(&orig_map, &map_with_visits);
    println!("Single obstacle scenario count: {}", obstacle_sim_results.len());

    Ok(())
}
