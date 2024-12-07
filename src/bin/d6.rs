// If there is something directly in front of you, turn right 90 degrees.
// Otherwise, take a step forward.

// How many distinct positions will the guard visit before leaving the mapped area?

use std::{
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
enum MapPosition {
    Empty,
    Obstacle,
    Visited,
    Guard(Direction),
}

type Map = Vec<Vec<MapPosition>>;

impl TryFrom<char> for MapPosition {
    type Error = String;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        let pos = match value {
            '.' => Self::Empty,
            '#' => Self::Obstacle,
            'X' => Self::Visited,
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
            Self::Visited => 'X',
            Self::Guard(direction) => direction.as_char(),
        };
        write!(f, "{}", c)
    }
}

fn parse_input<P: AsRef<Path>>(path: P) -> anyhow::Result<Map> {
    let full_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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

fn simulate_movements(mut map: Map) -> Map {
    // find guard position
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
    loop {
        // only required for the first case in the present impl
        map[guard_position.row][guard_position.col] = MapPosition::Visited;

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
            (Some(r), Some(c)) if c < col_count && r < row_count => {
                (r as usize, c as usize)
            }
            _ => break, // the guard has left the building
        };

        let next_map_element = map[next_row][next_col];
        match next_map_element {
            MapPosition::Empty | MapPosition::Visited => {
                // mark next spot as visited and put the guard in this pos
                map[next_row][next_col] = MapPosition::Visited;
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
    map
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
        .map(|r| r.iter().filter(|&p| matches!(*p, MapPosition::Visited)))
        .flatten()
        .count()
}

fn main() -> anyhow::Result<()> {
    let map = parse_input("d6-p1.txt")?;
    print_map(&map);
    let map = simulate_movements(map);
    print_map(&map);
    let visited = positions_visited(&map);
    println!("Positions Visited: {visited}");

    Ok(())
}
