use std::path::Path;

use aoc::input_lines;
use regex::Regex;

#[derive(Debug, Clone)]
struct Robot {
    x: isize,
    y: isize,
    vx: isize,
    vy: isize,
}

fn parse_input<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<Robot>> {
    // example line: p=0,4 v=3,-3
    let robo_re = Regex::new(r"p=(?<x>\d+),(?<y>\d+) v=(?<vx>[-]?\d+),(?<vy>[-]?\d+)")?;
    let robots = input_lines(path)?.enumerate().filter_map(|(_idx, l) | {
        let caps = robo_re.captures(&l)?;
        let x = caps.name("x")?.as_str().parse().ok()?;
        let y = caps.name("y")?.as_str().parse().ok()?;
        let vx = caps.name("vx")?.as_str().parse().ok()?;
        let vy = caps.name("vy")?.as_str().parse().ok()?;
        Some(Robot {
            x,
            y,
            vx,
            vy,
        })
    })
    .collect();

    Ok(robots)
}

#[derive(Debug)]
struct RobotPosition {
    x: isize,
    y: isize,
}

fn simulate_robot(robot: &mut Robot, xmax: isize, ymax: isize, seconds: usize) {
    for _ in 0..seconds {
        robot.x = (robot.x + robot.vx) % xmax;
        robot.y = (robot.y + robot.vy) % ymax;

        if robot.x < 0 {
            robot.x += xmax;
        }

        if robot.y < 0 {
            robot.y += ymax;
        }
    }
}

// pick up here tomorrow; it seems like this is probably another dynamic programming
// problem as we'll start to see repititions in positions across robots with the same
// velocities.
//
// Alternatively, there may be a more pure mathematical solution that avoid actual
// simulation; that certainly shouldn't be necessary for Part 1, but something tells
// me it will be required for Part 2.  In keeping with my general approaach, however,
// we'll just start with a solution as brute as possible and see how that fares and
// then step up are game as required.
fn simulate(robots: &mut [Robot], xmax: isize /* cols */, ymax: isize /* rows */, seconds: usize) -> Vec<RobotPosition> {
    let mut positions = Vec::new();
    for robot in robots {
        simulate_robot(robot, xmax, ymax, seconds);
        positions.push(RobotPosition {
            x: robot.x,
            y: robot.y,
        })
    }
    positions
}

fn compute_safety_factory(positions: &[RobotPosition], xmax: isize, ymax: isize) -> usize {
    let mut tl = 0;
    let mut tr = 0;
    let mut bl = 0;
    let mut br = 0;
    let xmid = ((xmax - 1) / 2) as isize;
    let ymid = ((ymax - 1) / 2) as isize;
    for pos in positions {
        // left side
        if pos.x < xmid {
            if pos.y < ymid {
                tl += 1;
            }
            if pos.y > ymid {
                bl += 1;
            }
        }

        // right side
        if pos.x > xmid {
            if pos.y < ymid {
                tr += 1;
            }
            if pos.y > ymid {
                br += 1;
            }
        }
    }

    tl * tr * bl * br
}

fn main() -> anyhow::Result<()> {
    let mut robots = parse_input("d14.txt")?;
    let xmax = 101;
    let ymax = 103;
    let seconds = 100;
    let positions = simulate(&mut robots, xmax, ymax, seconds);
    println!("\nSeconds={seconds}, Positions={positions:?}");
    for y in 0..ymax {
        for x in 0..xmax {
            let present = positions.iter().filter(|p| p.x == x && p.y == y).count();
            if present == 0 {
                print!(".");
            } else {
                print!("{present}");
            }
        }
        println!("");
    }
    let sf = compute_safety_factory(&positions, xmax, ymax);
    println!("Safety Factory: {sf}");
    Ok(())
}
