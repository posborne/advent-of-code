use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

#[derive(Debug)]
struct OrderingRule {
    first: usize,
    second: usize,
}

#[derive(Debug)]
struct Inputs {
    ordering_rules: Vec<OrderingRule>,
    page_orderings: Vec<Vec<usize>>,
}

fn parse_inputs<P: AsRef<Path>>(path: P) -> anyhow::Result<Inputs> {
    let full_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("inputs")
        .join(path);
    let f = File::open(full_path)?;
    let reader = BufReader::new(f);

    // orderings are first until blank line
    let mut ordering_rules: Vec<OrderingRule> = Vec::new();
    let mut line_iter = reader.lines();
    while let Some(line) = line_iter.next() {
        let line = line?;
        if line == "" {
            break; // end of section
        }
        let (first, second) = line
            .split_once("|")
            .ok_or_else(|| anyhow::anyhow!("Failed to parse line: {line}"))?;
        ordering_rules.push(OrderingRule {
            first: first.parse().expect("Number parsing fail"),
            second: second.parse().expect("Number parsing fail"),
        });
    }

    let mut page_orderings = Vec::new();
    while let Some(line) = line_iter.next() {
        let line = line?;
        let ordering = line
            .split(",")
            .map(|num| num.parse::<usize>().expect("Number parsing fail!"))
            .collect::<Vec<usize>>();
        page_orderings.push(ordering);
    }

    Ok(Inputs {
        ordering_rules,
        page_orderings,
    })
}

fn part1() -> anyhow::Result<()> {
    let inputs = parse_inputs("d5-p1.txt")?;
    let Inputs {
        ordering_rules,
        page_orderings,
    } = inputs;

    let mut good_orderings = Vec::new();
    'page_ordering: for page_ordering in page_orderings.iter() {
        for ordering_rule in ordering_rules.iter() {
            let mut first_seen = false;
            let mut second_seen = false;
            let mut second_seen_first = false;
            for &page in page_ordering {
                if page == ordering_rule.first {
                    first_seen = true;
                }

                if page == ordering_rule.second {
                    second_seen = true;
                    if !first_seen {
                        second_seen_first = true;
                    }
                }
            }

            if first_seen && second_seen && second_seen_first {
                continue 'page_ordering;
            }
        }
        good_orderings.push(page_ordering);
    }

    let middle_pages_sum: usize = good_orderings
        .into_iter()
        .map(|ordering| {
            if ordering.len() % 2 != 1 {
                panic!("Expected odd number of pages");
            }

            let mid = ordering[ordering.len() / 2];
            mid
        })
        .sum();

    println!("Part 1: um of good ordering middle pages: {middle_pages_sum}");

    Ok(())
}

fn main() -> anyhow::Result<()> {
    part1()?;
    Ok(())
}
