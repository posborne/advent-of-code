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

// Doing this brute force is too compuationally expensive as expected;
// for tomorrow, probably look to see what we can do with optimizing
// a system of equations based on the mathematical system of equations
// we get for a/b hits, p_x/p_y (prize_loc), and m_x,m_y (machine x/y cal.)
//
// On paper I think we have the following, though my math is rusty.
// - b = (px - a * mxa) / mxb
// - b = (py - a * mya) / myb
//
// As a constraint, we only care about the set of all positive integers;
// if we can find all solutions somehow, we can probably stil get away
// with finding the optimal solution (based on cost) in a naive fashion.
//
// This implementation iterates through values of a and solve for b
// and we discard all solutions that don't have an integer solution
// for both x and y.
//
// This should be linear time at least, but does still seem to be
// slow as we're dealing with big numbers...
//
// More math to do, keeping this in for reference.
#[allow(unused)]
fn find_optimal_constrain_single_variable(machine: &ClawMachine) -> Option<(usize, usize)> {
    let mut working: HashSet<(usize, usize)> = HashSet::new();
    let mut a_hits = 0;

    // just do float conversion up from to clean things up
    let px_f = machine.prize_location.0 as f64;
    let mx_a_f = machine.a.x as f64;
    let mx_b_f = machine.b.x as f64;
    while a_hits * machine.a.x <= machine.prize_location.0
        && a_hits * machine.a.y <= machine.prize_location.1
    {
        if a_hits % 100_000_000 == 0 {
            println!("{a_hits}");
        }
        // solve for b using fixed a
        let b_x = (px_f - a_hits as f64 * mx_a_f) / mx_b_f;
        println!("b_x = {b_x}");
        if b_x.fract() == 0.0 {
            // there's a solution for b_x that is an integer but that only matters
            // if we end up with that solution being correct for y as well
            let b_hits = b_x.floor() as usize;
            let y = a_hits * machine.a.y + b_hits * machine.b.y;
            if y == machine.prize_location.1 {
                // we have a match!
                working.insert((a_hits, b_hits));
            }
        }
        a_hits += 1;
    }

    working.into_iter().min_by_key(|(a, b)| a * 3 + b)
}

// Let's look at our two equations a bit more and solve for a/b further by
// combining our two equations...
//
//     a * ax + b * bx = px
//     a * ay + b * by = py
//
// So, we have a system of linear equestions with two variables.  How do we
// solve this?  Well, I am real rusty on this and found some people talking
// about Cramer's Rule for 2x2 Linear Systems, so I went with that.
// https://en.wikipedia.org/wiki/Cramer%27s_rule
//
//         | ax bx |
//     D = | ay by |
//
// Where D is the coefficient determinant for our input as a
// 2x2 matrix.  Computing the determinant involves subtracting the products
// of its diagonals. So:
//
//     D = ax * by - ay * bx
//
// Then Cramer's rule says that we can find the value of a given variable
// by dividing that variable's determinant by the coefficient-determinant's
// value where the variable's determinant is computed by replacing the
// variables column with our solutions (rhs - px and py) as follows:
//
//    D(b) = | ax px | = ax * py - ay * px
//           | ay py |
//
// b = D(b) / D = (ax*by - ay*bx) / (ax*py - ay*px)
//
// All of those variables are fixed, so we can calculate b and check to see
// if it is an integer.  If it is, then we just need to see if the computed
// value for a works out right as well.
fn find_optimal_using_math(machine: &ClawMachine) -> Option<(usize, usize)> {
    // just do float conversion up from to clean things up
    let px = machine.prize_location.0 as f64;
    let py = machine.prize_location.1 as f64;
    let ax = machine.a.x as f64;
    let ay = machine.a.y as f64;
    let bx = machine.b.x as f64;
    let by = machine.b.y as f64;

    // do the craamer rule 2x2 calculation shown above
    let det = ax * by - ay * bx;

    // to see if the solution works as integer, just cast and do a final check; if
    // we had a float solution then the final checks won't line up
    let a = ((px * by - py * bx) / det) as usize;
    let b = ((ax * py - ay * px) / det) as usize;

    if a * machine.a.x + b * machine.b.x == machine.prize_location.0
        && a * machine.a.y + b * machine.b.y == machine.prize_location.1
    {
        Some((a, b))
    } else {
        None
    }
}

fn main() -> anyhow::Result<()> {
    let mut machines = parse_input("d13.txt")?;
    let mut tokens = 0;
    for machine in machines.iter() {
        if let Some((a, b)) = find_optimal_naive(machine) {
            tokens += a * 3 + b;
        }
    }
    println!("Part 1 Tokens: {tokens:?}");

    // now add 10000000000000 to X/Y of the inputs for part 2
    for machine in machines.iter_mut() {
        machine.prize_location.0 += 10_000_000_000_000;
        machine.prize_location.1 += 10_000_000_000_000;
    }
    tokens = 0;
    for machine in machines.iter() {
        if let Some((a, b)) = find_optimal_using_math(machine) {
            tokens += a * 3 + b;
        }
    }
    println!("Part 2 Tokens: {tokens:?}");

    Ok(())
}
