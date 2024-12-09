use std::{collections::{HashMap, HashSet}, fs::File, io::{BufRead, BufReader}, path::{Path, PathBuf}};

use itertools::Itertools;

#[derive(Debug, Clone)]
enum AntMapPosition {
    Blank,
    Antenna(char),
}

fn parse_input<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<Vec<AntMapPosition>>> {
    let full_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("inputs")
        .join(path);
    let f = File::open(full_path)?;
    let reader = BufReader::new(f);
    let positions = reader.lines().filter_map(|l| l.ok()).map(|l| {
        l.chars().map(|c| match c {
            '.' => AntMapPosition::Blank,
            c => AntMapPosition::Antenna(c),
        }).collect::<Vec<AntMapPosition>>()
    }).collect::<Vec<Vec<AntMapPosition>>>();
    Ok(positions)
}

fn ant_positions(inputs: &Vec<Vec<AntMapPosition>>) -> HashMap<char, Vec<(usize, usize)>> {
    let mut antennas_with_positions: HashMap<char, Vec<(usize, usize)>> = HashMap::new();
    for row_idx in 0..inputs.len() {
        for col_idx in 0..inputs[0].len() {
            if let AntMapPosition::Antenna(c) = inputs[row_idx][col_idx] {
                let entry = antennas_with_positions.entry(c).or_default();
                entry.push((row_idx, col_idx));
            }
        }
    }
    antennas_with_positions
}

fn compute_antinodes(ant_positions: &Vec<(usize, usize)>, row_count: usize, col_count: usize) -> Vec<(usize, usize)> {
    let mut antinode_positions = Vec::new();
    for ((a_x, a_y), (b_x, b_y)) in ant_positions.iter().tuple_combinations() {
        let delta_x = a_x.abs_diff(*b_x);
        let delta_y = a_y.abs_diff(*b_y);

        // there's probably a more elegant way to do this...
        let ((a1_x, a1_y), (a2_x, a2_y)) = match (a_x > b_x, a_y > b_y) {
            (true, true) => {
                // a is to the right and below b:
                // ---
                // b
                //   a
                let tl = (b_x.checked_sub(delta_x), b_y.checked_sub(delta_y));
                let br = (a_x.checked_add(delta_x), a_y.checked_add(delta_y));
                (tl, br)
            }
            (true, false) => {
                // a is to the right and above b:
                // ---
                //   a
                // b
                let tr = (a_x.checked_add(delta_x), a_y.checked_sub(delta_y));
                let bl = (b_x.checked_sub(delta_x), b_y.checked_add(delta_y));
                (tr, bl)
            }
            (false, true) => {
                // a is to the left and below b
                // ---
                //   b
                // a
                let tr = (b_x.checked_add(delta_x), b_y.checked_sub(delta_y));
                let bl = (a_x.checked_sub(delta_x), a_y.checked_add(delta_y));
                (tr, bl)
            }
            (false, false) => {
                // a is to the left and above b
                // ---
                // a
                //   b
                let tl = (a_x.checked_sub(delta_x), a_y.checked_sub(delta_y));
                let br = (b_x.checked_add(delta_x), b_y.checked_add(delta_y));
                (tl, br)
            }
        };

        for p in &[(a1_x, a1_y), (a2_x, a2_y)] {
            match (p.0, p.1) {
                (Some(x), Some(y)) if x < row_count && y < col_count => {
                    antinode_positions.push((x, y));
                }
                _ => {} // out of bounds
            }
        }
    }

    antinode_positions
}

fn main() -> anyhow::Result<()> {
    let inputs = parse_input("d8-p1.txt")?;
    let ant_positions = ant_positions(&inputs);
    let row_count = inputs.len();
    let col_count = inputs[0].len();
    let mut unique_antinodes: HashSet::<(usize, usize)> = HashSet::new();
    for (ant, positions) in ant_positions.iter() {
        println!("Antenna '{ant}' @ ({positions:?})");
        let anti_positions = compute_antinodes(positions, row_count, col_count);

        for anti_pos in anti_positions.iter() {
            unique_antinodes.insert(*anti_pos);
        }

        for row in 0..inputs.len() {
            for col in 0..inputs[0].len() {
                let in_positions = positions.contains(&(row, col));
                let in_anti = anti_positions.contains(&(row, col));
                let c = match (in_positions, in_anti) {
                    (true, true) => '*',
                    (true, false) => *ant,
                    (false, true) => 'X',
                    (false, false) => '.'
                };
                print!("{c}");
            }
            println!("");
        }
        println!("");
    }

    println!("Unique Antinode Positions: {}", unique_antinodes.len());

    Ok(())
}
