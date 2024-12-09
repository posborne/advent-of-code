use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

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
    let positions = reader
        .lines()
        .filter_map(|l| l.ok())
        .map(|l| {
            l.chars()
                .map(|c| match c {
                    '.' => AntMapPosition::Blank,
                    c => AntMapPosition::Antenna(c),
                })
                .collect::<Vec<AntMapPosition>>()
        })
        .collect::<Vec<Vec<AntMapPosition>>>();
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

fn in_bounds(
    x: Option<usize>,
    y: Option<usize>,
    row_count: usize,
    col_count: usize,
) -> Option<(usize, usize)> {
    match (x, y) {
        (Some(x), Some(y)) if x < row_count && y < col_count => Some((x, y)),
        _ => None,
    }
}

fn insert_resonant_ext(
    output: &mut Vec<(usize, usize)>,
    a: (Option<usize>, Option<usize>),
    b: (Option<usize>, Option<usize>),
    a_deltas: (isize, isize),
    b_deltas: (isize, isize),
    row_count: usize,
    col_count: usize,
) {
    println!("insert_resonant_ext: a={a:?}, b={b:?}");
    let a_val = in_bounds(a.0, a.1, row_count, col_count);
    let b_val = in_bounds(b.0, b.1, row_count, col_count);
    if a_val.is_none() && b_val.is_none() {
        return; // we're done done
    }

    let aprime = if let Some(a) = a_val {
        output.push(a);
        (
            a.0.checked_add_signed(a_deltas.0),
            a.1.checked_add_signed(a_deltas.1),
        )
    } else {
        (None, None)
    };

    let bprime = if let Some(b) = b_val {
        output.push(b);
        (
            b.0.checked_add_signed(b_deltas.0),
            b.1.checked_add_signed(b_deltas.1),
        )
    } else {
        (None, None)
    };

    // recurse to carry on the resonance
    insert_resonant_ext(
        output, aprime, bprime, a_deltas, b_deltas, row_count, col_count,
    )
}

fn insert_resonant(
    output: &mut Vec<(usize, usize)>,
    a: (usize, usize),
    b: (usize, usize),
    row_count: usize,
    col_count: usize,
) {
    let (a_x, a_y, b_x, b_y) = (a.0 as isize, a.1 as isize, b.0 as isize, b.1 as isize);
    let delta_x = a_x.abs_diff(b_x) as isize;
    let delta_y = a_y.abs_diff(b_y) as isize;
    let (a_deltas, b_deltas) = match (a.0 > b.0, a.1 > b.1) {
        (true, true) => {
            // a is to the right and below b:
            // ---
            // b
            //   a
            (
                (delta_x, delta_y), // bottom right
                (-delta_x, -delta_y),
            ) // top left
        }
        (true, false) => {
            // a is to the right and above b:
            // ---
            //   a
            // b
            ((delta_x, -delta_y), (-delta_x, delta_y))
        }
        (false, true) => {
            // a is to the left and below b
            // ---
            //   b
            // a
            ((-delta_x, delta_y), (delta_x, -delta_y))
        }
        (false, false) => {
            // a is to the left and above b
            // ---
            // a
            //   b
            ((-delta_x, -delta_y), (delta_x, delta_y))
        }
    };
    insert_resonant_ext(
        output,
        (Some(a.0), Some(a.1)),
        (Some(b.0), Some(b.1)),
        a_deltas,
        b_deltas,
        row_count,
        col_count,
    )
}

fn compute_antinodes(
    ant_positions: &Vec<(usize, usize)>,
    row_count: usize,
    col_count: usize,
) -> Vec<(usize, usize)> {
    let mut antinode_positions = Vec::new();
    for ((a_x, a_y), (b_x, b_y)) in ant_positions.iter().tuple_combinations() {
        insert_resonant(
            &mut antinode_positions,
            (*a_x, *a_y),
            (*b_x, *b_y),
            row_count,
            col_count,
        );
    }

    println!("ANTINODE POSITIONS: {antinode_positions:?}");
    antinode_positions
}

fn main() -> anyhow::Result<()> {
    let inputs = parse_input("d8-p1.txt")?;
    let ant_positions = ant_positions(&inputs);
    let row_count = inputs.len();
    let col_count = inputs[0].len();
    let mut unique_antinodes: HashSet<(usize, usize)> = HashSet::new();
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
                    (false, false) => '.',
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
