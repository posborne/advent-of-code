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
    let mut a_hits = 0;
    while a_hits * machine.a.x <= machine.prize_location.0
        && a_hits * machine.a.y <= machine.prize_location.1
    {
        let mut b_hits = 0;
        loop {
            let position = (
                machine.a.x * a_hits + machine.b.x * b_hits,
                machine.a.y * a_hits + machine.b.y * b_hits,
            );
            // TODO: could exit earlier here with cmps
            if position.0 > machine.prize_location.0 || position.1 > machine.prize_location.1 {
                break; // dead end
            } else if position == machine.prize_location {
                working.insert((a_hits, b_hits));
            }
            b_hits += 1;
        }
        a_hits += 1;
    }

    working.into_iter().min_by_key(|(a, b)| a * 3 + b)
}

fn main() -> anyhow::Result<()> {
    let mut machines = parse_input("d13-example1.txt")?;
    let mut tokens = 0;
    for machine in machines.iter() {
        if let Some((a, b)) = find_optimal_naive(machine) {
            tokens += a * 3 + b;
        }
    }
    println!("Part 1 Tokens: {tokens:?}");

    // now add 10000000000000 to X/Y of the inputs for part 2
    //
    // Doing this brute force is too compuationally expensive as expected;
    // for tomorrow, probably look to see what we can do with optimizing
    // a system of equations based on the mathematical system of equations
    // we get for a/b hits, p_x/p_y (prize_loc), and m_x,m_y (machine x/y cal.)
    //
    // On paper I think we have the following, though my math is rusty.
    // - a = px/mx - b
    // - a = py/my - b
    //
    // As a constraint, we only care about the set of all positive integers;
    // if we can find all solutions somehow, we can probably stil get away
    // with finding the optimal solution (based on cost) in a naive fashion.
    //
    // Probably combine this into a simpler system of equations.
    for machine in machines.iter_mut() {
        machine.prize_location.0 += 10000000000000;
        machine.prize_location.1 += 10000000000000;
    }
    tokens = 0;
    for machine in machines.iter() {
        if let Some((a, b)) = find_optimal_naive(machine) {
            tokens += a * 3 + b;
        }
    }
    println!("Part 2 Tokens: {tokens:?}");

    Ok(())
}
