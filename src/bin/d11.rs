use std::{
    fs::File, path::{Path, PathBuf}
};

// with the replacement going on, at first blush I'm getting the feeling that
// we want some kind of balanced binary tree sort of thing...  That may not
// be right, however, so let's send it naive first.
fn parse_input<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<usize>> {
    let full_path = PathBuf::from(".")
        .join("inputs")
        .join(path);
    let s = std::io::read_to_string(File::open(full_path)?)?;
    Ok(s.split_whitespace()
        .map(|stone| stone.parse::<usize>().unwrap())
        .collect::<Vec<usize>>())
}

// Rules:
//
// 1. If the stone is engraved with the number 0, it is replaced by a stone
//    engraved with the number 1.
// 2. If the stone is engraved with a number that has an even number of digits,
//    it is replaced by two stones. The left half of the digits are engraved on
//    the new left stone, and the right half of the digits are engraved on the
//    new right stone. (The new numbers don't keep extra leading zeroes: 1000
//    would become stones 10 and 0.)
// 3. If none of the other rules apply, the stone is replaced by a new stone;
//    the old stone's number multiplied by 2024 is engraved on the new stone.
#[memoize::memoize]
fn count(stone: usize, generation: usize) -> usize {
    if generation == 0 {
        return 1;
    }

    // Rule 1
    if stone == 0 {
        return count(1, generation - 1);
    }

    // Rule 2
    // the log base 10 of a number is the number of digits
    let digits = stone.ilog10() + 1;
    if digits % 2 == 0 {
        // left digits
        let mut num = stone;
        let mut left = 0;
        let mut right = 0;
        for dig_idx in 0..digits {
            let digit = num % 10;
            let mid_idx = digits / 2;
            let pow = if dig_idx < mid_idx {
                dig_idx
            } else {
                // ex: for a 4-digit number we want to raise digit
                // with index 3 (the last digit) by 1.  The mid_idx
                // copmuted is 2 (4 / 2), so we want dig_idx - mid_idx
                dig_idx - mid_idx
            };
            num /= 10;
            if dig_idx < mid_idx {
                right += digit * 10usize.pow(pow);
            } else {
                left += digit * 10usize.pow(pow);
            }
        }

        // recurse on our left and right digits
        return count(left, generation - 1) + count(right, generation - 1);
    }

    // Rule 3
    count(stone * 2024, generation - 1)
}

fn main() -> anyhow::Result<()> {
    let stones = parse_input("d11.txt")?;
    println!("Stones: {stones:?}");

    // Blink 25 times
    println!("Part 1:");
    let count_25: usize = stones.iter().map(|stone| count(*stone, 25)).sum();
    println!("Blink 25: Count = {count_25}");

    // Now blink another 50 times...
    println!("\n\nPart 2:");
    let count_75: usize = stones.iter().map(|stone| count(*stone, 75)).sum();
    println!("Blink 75: Count = {count_75}");

    Ok(())
}
