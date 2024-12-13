use regex::{Captures, Regex};
use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
struct Movement {
    x: usize,
    y: usize,
}

#[derive(Debug, Clone)]
struct ClawMachine {
    a: Movement,
    b: Movement,
    prize_location: (usize, usize),
}

fn parse_xy_from_capture(caps: &Captures) -> anyhow::Result<(usize, usize)> {
    let (xstr, ystr) = match (caps.name("x"), caps.name("y")) {
        (Some(xstr), Some(ystr)) => (xstr.as_str(), ystr.as_str()),
        (xres, yres) => {
            return Err(anyhow::anyhow!(
                "X/Y missing from capture: caps={caps:?}, xres={xres:?}, yres={yres:?}"
            ));
        }
    };

    let (x, y) = match (xstr.parse::<usize>(), ystr.parse::<usize>()) {
        (Ok(x), Ok(y)) => (x, y),
        (xres, yres) => {
            return Err(anyhow::anyhow!("X/Y Integer Parsing failed, xstr={xstr:?}, xres={xres:?}, ystr={ystr:?}, yres={yres:?}"));
        }
    };

    Ok((x, y))
}

fn parse_input<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<ClawMachine>> {
    let full_path = PathBuf::from(".").join("inputs").join(path);
    let f = File::open(full_path)?;
    let reader = BufReader::new(f);
    let mut machines = vec![];
    let mut lines = reader.lines();
    let button_re = Regex::new(r"Button [A|B]: X[+](?<x>\d+), Y[+](?<y>\d+)")?;
    let prize_re = Regex::new(r"Prize: X=(?<x>\d+), Y=(?<y>\d+)")?;
    while let (Some(Ok(a)), Some(Ok(b)), Some(Ok(prize)), _) =
        (lines.next(), lines.next(), lines.next(), lines.next())
    {
        let a_movement = button_re
            .captures(&a)
            .ok_or_else(|| anyhow::anyhow!("Button A Parse Fail"))
            .map(|caps| parse_xy_from_capture(&caps))??;

        let b_movement = button_re
            .captures(&b)
            .ok_or_else(|| anyhow::anyhow!("Button B Parse Fail"))
            .map(|caps| parse_xy_from_capture(&caps))??;

        let prize_location = prize_re
            .captures(&prize)
            .ok_or_else(|| anyhow::anyhow!("Prize parse fail"))
            .map(|caps| parse_xy_from_capture(&caps))??;

        let machine = ClawMachine {
            a: Movement {
                x: a_movement.0,
                y: a_movement.1,
            },
            b: Movement {
                x: b_movement.0,
                y: b_movement.1,
            },
            prize_location,
        };
        machines.push(machine);
    }

    Ok(machines)
}

fn find_optimal_naive(machine: &ClawMachine) -> Option<(usize, usize)> {
    let mut working: HashSet<(usize, usize)> = HashSet::new();
    for a_hits in 0..100 {
        for b_hits in 0..100 {
            let position = (
                machine.a.x * a_hits + machine.b.x * b_hits,
                machine.a.y * a_hits + machine.b.y * b_hits,
            );
            // TODO: could exit earlier here with cmps
            if position == machine.prize_location {
                working.insert((a_hits, b_hits));
            }
        }
    }

    working.into_iter().min_by_key(|(a, b)| a * 3 + b)
}

fn main() -> anyhow::Result<()> {
    let machines = parse_input("d13.txt")?;
    let mut tokens = 0;
    for machine in machines {
        if let Some((a, b)) = find_optimal_naive(&machine) {
            tokens += a * 3 + b;
        }
    }
    println!("Tokens: {tokens:?}");
    Ok(())
}
