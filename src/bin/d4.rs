use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

fn parse_input<P>(path: P) -> anyhow::Result<Vec<String>>
where
    P: AsRef<Path>,
{
    let path = PathBuf::from(".")
        .join("inputs")
        .join(path);
    let f = File::open(path)?;
    let reader = BufReader::new(f);
    let puzzle = reader
        .lines()
        .filter_map(|l| l.ok())
        .collect::<Vec<String>>();
    Ok(puzzle)
}

mod p1 {
    use super::*;

    const XMAS: &str = "XMAS";

    fn find_ltr(puzzle: &[String]) -> usize {
        puzzle.iter().map(|line| line.matches(XMAS).count()).sum()
    }

    fn find_rtl(puzzle: &[String]) -> usize {
        puzzle
            .iter()
            .map(|line| line.chars().rev().collect::<String>())
            .map(|revline| revline.matches(XMAS).count())
            .sum()
    }

    fn find_in_rows(puzzle: &[String]) -> usize {
        find_ltr(puzzle) + find_rtl(puzzle)
    }

    fn find_in_cols(puzzle: &[String]) -> usize {
        let cols: Vec<String> = (0..puzzle.len())
            .map(|col_idx| {
                puzzle
                    .iter()
                    .map(|row| row.chars().nth(col_idx).unwrap())
                    .collect::<String>()
            })
            .collect();

        // now that we've flipped the puzzle, it's the
        // same puzzle as doing a row search
        find_ltr(&cols) + find_rtl(&cols)
    }

    fn tl_to_br_diagonals(puzzle: &[String]) -> Vec<String> {
        // for this first pass, assume rows/cols are equal;
        // we'll address that issue later.
        let col_count = puzzle[0].len();
        let row_count = puzzle.len();
        let mut diags = Vec::new();

        // first pass fills half the diagonals; there's probably a
        // better way to do this, but this is how it ended up after
        // my first idea of an algorithm on paper.
        for col_idx in 0..col_count {
            let mut diag = String::new();
            for row_idx in 0..(row_count - col_idx) {
                // row/col at same index is on the ltr diagonal
                let row = &puzzle[row_idx];
                if let Some(c) = row.chars().nth(row_idx + col_idx) {
                    diag.push(c);
                }
            }
            diags.push(diag);
        }

        // second pass is to fill in the the remaining one diagonal
        // per row
        for row_idx in 1..row_count {
            let mut diag = String::new();
            for level_idx in 0..(col_count - row_idx) {
                let row = &puzzle[row_idx + level_idx];
                if let Some(c) = row.chars().nth(level_idx) {
                    diag.push(c);
                }
            }
            diags.push(diag);
        }

        diags
    }

    fn tr_to_bl_diagonals(puzzle: &[String]) -> Vec<String> {
        // flipping the rows around and using the same tl->br algo
        // gives us the tr->bl results we want
        let flipped_puzzle = puzzle.iter().rev().cloned().collect::<Vec<String>>();

        tl_to_br_diagonals(&flipped_puzzle)
    }

    fn find_diagonal(puzzle: &[String]) -> usize {
        // do some transform to get iterators over diagonals;
        // we can break down the types of diagnols into:
        // 1. TL to BR
        // 2. BR to TL (1 reversed)
        // 3. TR to BL
        // 4. BL to TR (3 reversed)
        let tl_to_br = tl_to_br_diagonals(&puzzle);
        let tr_to_bl = tr_to_bl_diagonals(&puzzle);

        // case 1 and 2n
        find_ltr(&tl_to_br) + find_rtl(&tl_to_br) + find_ltr(&tr_to_bl) + find_rtl(&tr_to_bl)
    }

    pub fn part1() -> anyhow::Result<()> {
        let puzzle = parse_input("d4-p1.txt")?;
        let total = find_in_rows(&puzzle) + find_in_cols(&puzzle) + find_diagonal(&puzzle);
        println!("Found XMAS {total} times");
        Ok(())
    }
}

mod p2 {
    use crate::parse_input;

    // In this part, we're looking for X-MAS as in "M A S" in the form
    // of an X (sigh).  For this one, we're just going to brute search
    // for 3x3 grids that have one of the possible valid sets of positions
    // present
    //
    // Those are the following 4 combinations:
    //
    // M . M | S . M | S . S | M . S
    // . A . | . A . | . A . | . A .
    // S . S | S . M | M . M | M . S
    //
    // Which we'll encode as offsets from our top-left position (0, 0)
    const PATTERNS: &[&[(usize, usize, char)]] = &[
        &[
            (0, 0, 'M'),
            (0, 2, 'M'),
            (1, 1, 'A'),
            (2, 0, 'S'),
            (2, 2, 'S'),
        ],
        &[
            (0, 0, 'S'),
            (0, 2, 'M'),
            (1, 1, 'A'),
            (2, 0, 'S'),
            (2, 2, 'M'),
        ],
        &[
            (0, 0, 'S'),
            (0, 2, 'S'),
            (1, 1, 'A'),
            (2, 0, 'M'),
            (2, 2, 'M'),
        ],
        &[
            (0, 0, 'M'),
            (0, 2, 'S'),
            (1, 1, 'A'),
            (2, 0, 'M'),
            (2, 2, 'S'),
        ],
    ];

    pub fn part2() -> anyhow::Result<()> {
        let puzzle = parse_input("d4-p1.txt")?;
        let puzarr = puzzle
            .into_iter()
            .map(|r| r.chars().collect::<Vec<char>>())
            .collect::<Vec<Vec<char>>>();

        let row_count = puzarr.len();
        let col_count = puzarr[0].len();
        let mut matches = 0;
        for row_idx in 0..(row_count - 2) {
            for col_idx in 0..(col_count - 2) {
                'pattern: for &pattern in PATTERNS {
                    for (xoff, yoff, candidate_c) in pattern {
                        let c = puzarr[row_idx + xoff][col_idx + yoff];
                        if c != *candidate_c {
                            continue 'pattern; // go to the next candidate
                        }
                    }

                    // if we made it through, we have a match
                    matches += 1;
                }
            }
        }

        println!("Found {matches} matches!");

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    p1::part1()?;
    p2::part2()?;
    Ok(())
}
