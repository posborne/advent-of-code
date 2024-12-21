use std::path::Path;

use aoc::input_lines;
use clap::Parser;

#[derive(Debug, Clone)]
struct Inputs {
    towels: Vec<String>,
    patterns: Vec<String>,
}

fn parse_input<P: AsRef<Path>>(path: P) -> anyhow::Result<Inputs> {
    let mut lines = input_lines(path)?;
    let towels: Vec<String> = lines
        .next()
        .unwrap()
        .split(", ")
        .map(|t| t.trim().to_string())
        .collect();
    let _ = lines.next();
    let patterns: Vec<String> = lines.into_iter().collect();

    Ok(Inputs { towels, patterns })
}

#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value = "d19.txt")]
    input: String,
}

fn pattern_possible(pattern: &str, sorted_towels: &[&str], depth: usize) -> bool {
    // base case, the remaining pattern is an available towel; if not
    // then we know that the longest matching subpattern would at most
    // be the length of the closest match (though might be none).

    for _ in 0..depth { print!("  "); }
    println!("pattern: {pattern}");

    // base case
    let closest_towel = match sorted_towels.binary_search(&pattern) {
        Ok(_) => return true,
        Err(idx) => {
            sorted_towels[idx.saturating_add_signed(-1)]
        }
    };

    let in_common = pattern
        .chars()
        .zip(closest_towel.chars())
        .take_while(|(p, ct)| p == ct)
        .count();

    // work backwards from the largest possible subpattern
    for pivot in (0..in_common).rev() {
        let (subpattern, remainder) = pattern.split_at(pivot + 1);
        for _ in 0..depth { print!(" "); }
        println!("sub: {subpattern}/{remainder}");

        if !sorted_towels.binary_search(&subpattern).is_ok() {
            continue;
        }

        // we matched a subpattern, if the rest works out we're home free!
        if pattern_possible(remainder, sorted_towels, depth + 1) {
            return true;
        }
    }

    return false;
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let inputs = parse_input(cli.input)?;

    // Brainstorming for Part 1:
    // - Work through the target pattern in pieces with an expanding frontier; from
    //   any given character we have a few new paths that diverge that we could
    //   use to move forward to make progress...
    //   * Match a longer substring
    //   * Match the current substring
    //
    // This feels a little like other search problems at first blush where we might
    // want to have a queue or similar with some kind of SearchState object and
    // where we, from a given position, push additional things to check onto the
    // frontier (doing a sort of bfs).
    //
    // This would work greedily on larger substrings until none can be found and then
    // back off again; this would recurse but be somewhat limited by how long of an
    // pattern we're dealing with (at least for p1).
    let mut patterns_possible = 0;
    let mut towels_sorted: Vec<&str> = inputs.towels.iter().map(|t| t.as_ref()).collect();
    towels_sorted.sort();
    println!("Sorted: {towels_sorted:?}\n\n");
    for pattern in inputs.patterns.iter() {
        if pattern_possible(pattern, &towels_sorted, 0) {
            patterns_possible += 1;
        }
    }

    println!("Possible Patterns: {patterns_possible} / {}", inputs.patterns.len());

    Ok(())
}
