use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    usize,
};

type TopoMap = Vec<Vec<u8>>;

fn parse_input<P: AsRef<Path>>(path: P) -> anyhow::Result<TopoMap> {
    let full_path = PathBuf::from(".").join("inputs").join(path);
    let f = File::open(full_path)?;
    let reader = BufReader::new(f);
    let topo: TopoMap = reader
        .lines()
        .filter_map(|l| l.ok())
        .map(|l| l.bytes().map(|b| b - b'0').collect())
        .collect();
    Ok(topo)
}

fn trailheads_for_map(map: &TopoMap) -> Vec<(usize, usize)> {
    map.iter()
        .enumerate()
        .map(|(row_idx, row)| {
            row.iter().enumerate().filter_map(move |(col_idx, b)| {
                if *b == 0 {
                    Some((row_idx, col_idx))
                } else {
                    None
                }
            })
        })
        .flatten()
        .collect()
}

fn valid_position(
    map: &TopoMap,
    position: (Option<usize>, Option<usize>),
) -> Option<(usize, usize)> {
    let (Some(row), Some(col)) = position else {
        return None;
    };

    if row < map.len() && col < map[0].len() {
        Some((row, col))
    } else {
        None
    }
}

fn find_walkable_trails(
    map: &TopoMap,
    level: u8,
    position: (usize, usize),
) -> HashSet<(usize, usize)> {
    let elevation = map[position.0][position.1];
    let mut res = HashSet::new();

    if level != elevation {
        return res;
    }

    // for _ in 0..level {
    //     print!(" ");
    // }
    // println!("{}:{} => {level}", position.0, position.1);

    if level == 9 {
        res.insert(position);
        return res;
    }

    let left = (position.0.into(), position.1.checked_add_signed(-1));
    let right = (position.0.into(), position.1.checked_add_signed(1));
    let up = (position.0.checked_add_signed(-1), position.1.into());
    let down = (position.0.checked_add_signed(1), position.1.into());

    let positions = [left, right, up, down];
    for pos in positions {
        if let Some(pos) = valid_position(map, pos) {
            // update our result witht the set union of positions
            res.extend(&find_walkable_trails(map, level + 1, pos));
        }
    }

    res
}

fn score_trails(map: &TopoMap, level: u8, position: (usize, usize)) -> usize {
    let elevation = map[position.0][position.1];

    if level != elevation {
        return 0;
    }

    // for _ in 0..level {
    //     print!(" ");
    // }
    // println!("{}:{} => {level}", position.0, position.1);

    if level == 9 {
        return 1;
    }

    let left = (position.0.into(), position.1.checked_add_signed(-1));
    let right = (position.0.into(), position.1.checked_add_signed(1));
    let up = (position.0.checked_add_signed(-1), position.1.into());
    let down = (position.0.checked_add_signed(1), position.1.into());

    let positions = [left, right, up, down];
    let mut res = 0;
    for pos in positions {
        if let Some(pos) = valid_position(map, pos) {
            // update our result witht the set union of positions
            res += score_trails(map, level + 1, pos);
        }
    }

    res
}

fn score_trailhead(map: &TopoMap, trailhead: (usize, usize)) -> usize {
    find_walkable_trails(map, 0, trailhead).len()
}

fn rate_trailhead(map: &TopoMap, trailhead: (usize, usize)) -> usize {
    score_trails(map, 0, trailhead)
}

fn main() -> anyhow::Result<()> {
    let map = parse_input("d10.txt")?;
    let trailheads = trailheads_for_map(&map);
    println!("There are {} trailheads", trailheads.len());

    // By Score (Part 1)
    for trailhead in trailheads.iter() {
        let score = score_trailhead(&map, *trailhead);
        println!("{trailhead:?} => {score}")
    }
    let sum: usize = trailheads.iter().map(|th| score_trailhead(&map, *th)).sum();
    println!("Total Score: {sum}");

    // By Rating (Part 2)
    for trailhead in trailheads.iter() {
        let rating = rate_trailhead(&map, *trailhead);
        println!("{trailhead:?} => {rating}")
    }
    let sum: usize = trailheads.iter().map(|th| rate_trailhead(&map, *th)).sum();
    println!("Total Rating: {sum}");

    Ok(())
}
