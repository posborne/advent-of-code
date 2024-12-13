use std::{
    fs::File,
    io::{BufRead, BufReader},
    iter::zip,
    path::{Path, PathBuf},
};

fn parse_input<P: AsRef<Path>>(file: P) -> anyhow::Result<Vec<(i32, i32)>> {
    let f = File::open(PathBuf::from(".").join("inputs").join(file.as_ref()))?;
    let buf = BufReader::new(f);
    let pairs = buf
        .lines()
        .into_iter()
        .filter_map(|l| l.ok())
        .map(|l| {
            l.split_whitespace()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        })
        .filter_map(|parts| {
            let mut it = parts.into_iter();
            Some((it.next()?, it.next()?))
        })
        .filter_map(|(a, b)| {
            let a: i32 = a.parse().ok()?;
            let b: i32 = b.parse().ok()?;
            Some((a, b))
        })
        .collect::<Vec<(i32, i32)>>();
    Ok(pairs)
}

fn pairs_to_cols(pairs: Vec<(i32, i32)>) -> (Vec<i32>, Vec<i32>) {
    let left = pairs.iter().map(|(a, _)| *a).collect::<Vec<i32>>();
    let right = pairs.iter().map(|(_, b)| *b).collect::<Vec<i32>>();
    (left, right)
}

fn part1() -> anyhow::Result<()> {
    let (mut left, mut right) = pairs_to_cols(parse_input("d1-p1.txt")?);
    left.sort();
    right.sort();

    let total_distance: i32 = zip(left, right).map(|(a, b)| (a - b).abs()).sum();

    println!("Total Distance: {total_distance}");
    Ok(())
}

fn part2() -> anyhow::Result<()> {
    let (left, right) = pairs_to_cols(parse_input("d1-p2.txt")?);

    // we'll just do this naive in quadratic time
    let mut similarity_score = 0;
    for a in left.iter() {
        let mut dups = 0;
        for b in right.iter() {
            if *a == *b {
                dups += 1;
            }
        }
        similarity_score += dups * a;
    }

    println!("Similary Score: {similarity_score}");
    Ok(())
}

fn main() -> anyhow::Result<()> {
    part1()?;
    part2()?;
    Ok(())
}
