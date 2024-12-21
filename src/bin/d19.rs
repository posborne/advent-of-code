use std::{collections::HashMap, path::Path};

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

type Cache<'a> = HashMap<&'a str, usize>;

fn patterns_possible<'a>(
    pattern: &'a str,
    sorted_towels: &'a [&'a str],
    depth: usize,
    verbose: bool,
    cache: &mut Cache<'a>,
) -> usize {
    // base case, the remaining pattern is an available towel; if not
    // then we know that the longest matching subpattern would at most
    // be the length of the closest match (though might be none).
    if verbose {
        for _ in 0..depth {
            print!("  ");
        }
        println!("pattern: {pattern}");
    }

    if let Some(cached) = cache.get(&pattern) {
        return *cached;
    }

    let mut possible = 0;

    // base case
    let closest_towel = match sorted_towels.binary_search(&pattern) {
        Ok(idx) => {
            possible += 1;
            sorted_towels[idx.saturating_add_signed(-1)]
        }
        Err(idx) => sorted_towels[idx.saturating_add_signed(-1)],
    };

    let in_common = pattern
        .chars()
        .zip(closest_towel.chars())
        .take_while(|(p, ct)| p == ct)
        .count();

    // work backwards from the largest possible subpattern
    for pivot in (0..in_common).rev() {
        let (subpattern, remainder) = pattern.split_at(pivot + 1);

        if verbose {
            for _ in 0..depth {
                print!(" ");
            }
            println!("sub: {subpattern}/{remainder}");
        }

        if !sorted_towels.binary_search(&subpattern).is_ok() {
            continue;
        }

        // we matched a subpattern, if the rest works out we're home free!
        let remaining_possible =
            patterns_possible(remainder, sorted_towels, depth + 1, verbose, cache);
        if remaining_possible > 0 {
            possible += remaining_possible;
        }
    }

    cache.insert(pattern, possible);
    return possible;
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
    let mut ok_patterns = 0;
    let mut patterns_count = 0;
    let mut towels_sorted: Vec<&str> = inputs.towels.iter().map(|t| t.as_ref()).collect();
    towels_sorted.sort();
    println!("Sorted: {towels_sorted:?}\n\n");
    for pattern in inputs.patterns.iter() {
        println!("Working on pattern: {pattern}");
        let mut cache = Default::default();
        let patterns = patterns_possible(pattern, &towels_sorted, 0, false, &mut cache);
        if patterns > 0 {
            ok_patterns += 1;
        }
        patterns_count += patterns;
    }

    println!("Note: 482106311433668 is too low");
    println!(
        "Passing Patterns: {ok_patterns} / {}",
        inputs.patterns.len()
    );
    println!("Possible Patterns: {patterns_count}");

    Ok(())
}
