use std::{
    collections::{HashMap, HashSet},
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

    #[arg(short, long, default_value_t = 100)]
    threshold_picoseconds: usize,

    #[arg(short, long, default_value_t = 2)]
    cheat_duration: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Cheat {
    start: Position,
    end: Position,
}

const DELTAS: [(isize, isize); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];
fn manhattan_distance(p1: &Position, p2: &Position) -> usize {
    p1.x.abs_diff(p2.x) + p1.y.abs_diff(p2.y)
}

fn solve() -> anyhow::Result<()> {
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
    //
    // ---
    //
    // Updated thinking:
    //
    // After that approach turning into a bit of a quagmire, I think there's
    // a more straightforward approach (reddit hints reading general tips)
    // which is to just walk the path and record the distance to the end from
    // that point.  Then, for each point, see if there is another piece of
    // road with a manhattan distance of 2 away that has a lower cost; that
    // difference is the picoseconds saved.
    let cli = Cli::parse();
    let map = parse_input(cli.input)?;
    print_map(&map);

    // walk the map from the end back to the start with the step
    // along the way being the cost (which we record)
    let mut visited: HashSet<Position> = HashSet::new();
    let mut road_costs: HashMap<Position, usize> = HashMap::new();
    let mut next_position = Some(map.end);
    let mut cost = 0;
    while let Some(position) = next_position {
        visited.insert(position);
        road_costs.insert(position, cost);
        if position == map.start {
            break;
        }

        next_position = DELTAS
            .into_iter()
            .filter_map(|(dx, dy)| {
                let x = position.x.checked_add_signed(dx)?;
                let y = position.y.checked_add_signed(dy)?;
                let pos = Position { x, y };
                let entry = map.entries[y][x];
                if visited.contains(&pos) || !matches!(entry, MapEntry::Road | MapEntry::Start) {
                    return None;
                }
                Some(pos)
            })
            .nth(0);
        cost += 1;
    }

    let mut shortcuts: Vec<(Cheat, usize)> = Vec::new();
    for (position, cost) in road_costs.iter() {
        for (tpos, tcost) in road_costs.iter() {
            let dist = manhattan_distance(position, tpos);
            if dist <= cli.cheat_duration
                && tcost < cost
                && cost - tcost - dist >= cli.threshold_picoseconds
            {
                let savings = cost - tcost - dist;
                let cheat = Cheat {
                    start: position.clone(),
                    end: tpos.clone(),
                };
                shortcuts.push((cheat, savings))
            }
        }
    }

    let mut shortcuts_by_savings: HashMap<usize, usize> = HashMap::new();
    for (_cheat, cost) in shortcuts {
        let entry = shortcuts_by_savings.entry(cost).or_default();
        *entry += 1;
    }

    for (savings, solutions) in shortcuts_by_savings.iter().sorted() {
        println!("{savings}: {solutions}");
    }

    let cheats_saving_gt_treshold: usize = shortcuts_by_savings
        .iter()
        .filter(|(savings, _count)| **savings >= cli.threshold_picoseconds)
        .map(|(_savings, count)| *count)
        .sum();

    println!(
        "Cheats (duration <= {}) saving >= {} picoseconds = {cheats_saving_gt_treshold}",
        cli.cheat_duration, cli.threshold_picoseconds
    );

    Ok(())
}

fn main() -> anyhow::Result<()> {
    solve()?;
    Ok(())
}
