use std::{
    fs::File,
    path::{Path, PathBuf},
};

// with the replacement going on, at first blush I'm getting the feeling that
// we want some kind of balanced binary tree sort of thing...  That may not
// be right, however, so let's send it naive first.
fn parse_input<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<usize>> {
    let full_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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
fn blink(input_stones: &[usize]) -> Vec<usize> {
    let mut output = Vec::new();
    for &stone in input_stones {
        // Rule 1
        if stone == 0 {
            output.push(1);
            continue;
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
            output.push(left);
            output.push(right);
            continue;
        }

        // Rule 3
        let new_stone = stone * 2024;
        output.push(new_stone);
    }

    output
}

fn main() -> anyhow::Result<()> {
    let mut stones = parse_input("d11.txt")?;
    println!("Stones: {stones:?}");
    for i in 1..=25 {
        stones = blink(&stones);
        println!("Blink {i}: Count = {}", stones.len());
    }
    Ok(())
}
