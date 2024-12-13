use std::{fs::File, io::BufReader, path::{Path, PathBuf}};

const MUL_RE: &str  = r"mul\((?<a>\d+),(?<b>\d+)\)";
const DO_RE: &str = r"do\(\)";
const DONT_RE: &str = r"don\'t\(\)";

fn parse_input<P: AsRef<Path>>(input_path: P) -> anyhow::Result<String> {
    let full_input_path = PathBuf::from(".").join("inputs").join(input_path);
    let f = File::open(full_input_path)?;
    let mut reader = BufReader::new(f);
    Ok(std::io::read_to_string(&mut reader)?)
}

mod p1 {
    use super::*;

    pub fn part1() -> anyhow::Result<()> {
        let input = parse_input("d3-p1.txt")?;
        // looking to match instances like 'sum(123,456)'
        let re = regex::RegexBuilder::new(MUL_RE).multi_line(true).build()?;
        let mut muls: Vec<(u32, u32)> = Vec::new();
        for cap in re.captures_iter(input.as_str()) {
            let a = cap["a"].parse::<u32>()?;
            let b = cap["b"].parse::<u32>()?;
            muls.push((a, b));
        }
        let sum: u32 = muls.into_iter().map(|(a, b)| {
            a * b
        }).sum();
        println!("Part1: Sum of muls: {sum}");
        Ok(())
    }
}

mod p2 {
    use regex::Regex;

    use super::*;

    pub fn part2() -> anyhow::Result<()> {
        let combo_re = Regex::new(&format!("(?<mul>{MUL_RE})|(?<do>{DO_RE})|(?<dont>{DONT_RE})"))?;
        let input = parse_input("d3-p1.txt")?;

        // use the match set for the first pass to figure out enabled/disabled; if
        // things are enabled then parse out the mul match
        let mut muls: Vec<(u32, u32)> = Vec::new();
        let mut enabled = true;
        for caps in combo_re.captures_iter(&input) {
            if let Some(_mul) = caps.name("mul") {
                let a = caps["a"].parse::<u32>()?;
                let b = caps["b"].parse::<u32>()?;
                if enabled {
                    muls.push((a, b));
                }
            }  else if let Some(_do) = caps.name("do") {
                enabled = true;
            } else if let Some (_dont) = caps.name("dont") {
                enabled = false;
            }
        }

        let sum: u32 = muls.into_iter().map(|(a, b)| a * b).sum();
        println!("Part2: Sum of enabled muls: {sum}");
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    p1::part1()?;
    p2::part2()?;
    Ok(())
}
