use std::{cell::LazyCell, collections::HashMap, path::Path};

use aoc::input_lines;
use clap::Parser;
use itertools::Itertools;

fn parse_input<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<Vec<char>>> {
    Ok(input_lines(path)?
        .into_iter()
        .map(|line| line.chars().collect())
        .collect())
}

/*
+---+---+---+
| 7 | 8 | 9 |
+---+---+---+
| 4 | 5 | 6 |
+---+---+---+
| 1 | 2 | 3 |
+---+---+---+
    | 0 | A |
    +---+---+
*/
const NUMBER_PAD: LazyCell<HashMap<char, Position>> = LazyCell::new(|| {
    [
        ['7', '8', '9'],
        ['4', '5', '6'],
        ['1', '2', '3'],
        [' ', '0', 'A'],
    ]
    .into_iter()
    .enumerate()
    .flat_map(|(y, row)| {
        row.into_iter()
            .enumerate()
            .map(move |(x, key)| (key, Position { x, y }))
    })
    .collect()
});

/*
    +---+---+
    | ^ | A |
+---+---+---+
| < | v | > |
+---+---+---+
*/
const DIRECTIONAL_PAD: LazyCell<HashMap<char, Position>> = LazyCell::new(|| {
    [[' ', '^', 'A'], ['<', 'v', '>']]
        .into_iter()
        .enumerate()
        .flat_map(|(y, row)| {
            row.into_iter()
                .enumerate()
                .map(move |(x, key)| (key, Position { x, y }))
        })
        .collect()
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: usize,
    y: usize,
}

impl Position {
    fn delta(&self, other: &Position) -> Delta {
        let dx = other.x as isize - self.x as isize;
        let dy = other.y as isize - self.y as isize;
        Delta { dx, dy }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Delta {
    dx: isize,
    dy: isize,
}

type CacheKey = (usize, char, char);
type Cache = HashMap<CacheKey, usize>;

fn key_cost(cache: &Cache, robot_depth: usize, key_start: char, key_end: char) -> usize {
    if robot_depth == 0 {
        1
    } else {
        *cache
            .get(&(robot_depth, key_start, key_end))
            .unwrap_or_else(|| {
                panic!("invalid key doing memo lookup {robot_depth} {key_start} {key_end}");
            })
    }
}

fn keypresses_cost(cache: &mut Cache, robot_depth: usize, key_seq: &str) -> usize {
    format!("A{key_seq}")
        .chars()
        .tuple_windows()
        .map(|(key_start, key_end)| key_cost(cache, robot_depth, key_start, key_end))
        .sum()
}

fn populate_cache_for_robot(
    cache: &mut Cache,
    robot_depth: usize,
    keypad: &HashMap<char, Position>,
) {
    for (&start_key, start_pos) in keypad.iter() {
        for (&end_key, end_pos) in keypad.iter() {
            let delta = start_pos.delta(&end_pos);
            let horizontal_dist = delta.dx.abs() as usize;
            let vertical_dist = delta.dy.abs() as usize;

            let horizontal_keys = if end_pos.x > start_pos.x { ">" } else { "<" }.repeat(horizontal_dist);
            let vertical_keys = if end_pos.y < start_pos.y { "^" } else { "v" }.repeat(vertical_dist);

            let horizontal_key_seq = format!("{horizontal_keys}{vertical_keys}A");
            let vertical_key_seq = format!("{vertical_keys}{horizontal_keys}A");

            let Position { x: x_blank, y: y_blank } = keypad[&' '];
            let min_horizontal = if (end_pos.x, start_pos.y) != (x_blank, y_blank) {
                keypresses_cost(cache, robot_depth - 1, &horizontal_key_seq)
            } else {
                usize::MAX
            };

            let min_vertical = if (start_pos.x, end_pos.y) != (x_blank, y_blank) {
                keypresses_cost(cache, robot_depth - 1, &vertical_key_seq)
            } else {
                usize::MAX
            };

            cache.insert(
                (robot_depth, start_key, end_key),
                min_horizontal.min(min_vertical),
            );
        }
    }
}

fn build_cache(num_robots: usize) -> Cache {
    let mut cache: Cache = HashMap::new();

    // Cache moves for as many layers of robots as we have
    for robot in 1..=num_robots {
        populate_cache_for_robot(&mut cache, robot, &DIRECTIONAL_PAD);
    }

    // Add the final numeric keypad layer
    populate_cache_for_robot(&mut cache, num_robots + 1, &NUMBER_PAD);

    cache
}

fn compute_complexity(presses: usize, code: &[char]) -> usize {
    let digits: String = code.iter().filter(|c| c.is_digit(10)).collect();
    let num_value: usize = digits.parse().expect("Failed to parse as numeric value");
    println!(
        "{} => {} * {num_value}",
        code.iter().collect::<String>(),
        presses
    );
    return num_value * presses;
}

fn solve_code_for_keypresses(code: &[char], num_robots: usize) -> usize {
    let chars: String = code.iter().collect();
    let mut cache = build_cache(num_robots);
    keypresses_cost(&mut cache, num_robots + 1, &chars)
}

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    robots: usize,
}

fn main() -> anyhow::Result<()> {
    // NOTE: solution here based on review of work by ecyrbe after getting a bit stuck...
    // https://gist.github.com/ecyrbe/155bbe4baf80964913a579691447e192
    //
    // I did rework some parts of it a bit but it was heavily influenced as I
    // retranscibed the work done in that solution while getting a better grasp
    // of how to approach the memoization in this one; should have gotten
    // there on my own but the brain was moving a bit slow.

    let cli = Cli::parse();
    let final_codes = parse_input(cli.input)?;
    let mut sum: usize = 0;
    for code in final_codes {
        let presses = solve_code_for_keypresses(&code, cli.robots);
        println!("{}: {presses}", code.iter().collect::<String>());
        let complexity = compute_complexity(presses, &code);
        sum += complexity;
    }

    println!("Total Complexity: {sum}");

    Ok(())
}
