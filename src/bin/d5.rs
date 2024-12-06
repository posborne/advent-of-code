use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

#[derive(Debug)]
struct OrderingRule {
    first: usize,
    second: usize,
}

impl Display for OrderingRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}", self.first, self.second)
    }
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

fn part1_and_2() -> anyhow::Result<()> {
    let inputs = parse_inputs("d5-p1.txt")?;
    let Inputs {
        ordering_rules,
        page_orderings,
    } = inputs;

    let mut good_orderings = Vec::new();
    let mut bad_orderings = Vec::new();
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
                bad_orderings.push(page_ordering);
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

    let reordered_updates = fix_page_orderings(&ordering_rules, bad_orderings.as_slice());
    let reordered_pages_mid_sum: usize = reordered_updates
        .into_iter()
        .map(|ordering| {
            let mid = ordering[ordering.len() / 2];
            mid
        })
        .sum();
    println!("Part 2: sum of reordered middle pages: {reordered_pages_mid_sum}");

    Ok(())
}

fn fix_page_ordering(rules: &[OrderingRule], bad_ordering: &[usize]) -> Vec<usize> {
    // we might have multiple changes needed, so what algorithm should we use?
    //
    // Naively, we could just swap the first/second if there is a violation until
    // things work, but that may not halt.  Instead, we do a first pass on the
    // rules to determine a required ordering with the dependencies in the rules.

    // filter rules to be only those that apply to our series
    let mut filtered_rules = Vec::new();
    for rule in rules {
        let mut first_seen = false;
        let mut second_seen = false;
        for &page in bad_ordering {
            if page == rule.first {
                first_seen = true;
            }
            if page == rule.second {
                second_seen = true;
            }
        }
        if first_seen && second_seen {
            filtered_rules.push(rule);
        }
    }

    // println!("Series: {bad_ordering:?}");
    // print!("Rules: ");
    // for rule in filtered_rules.iter() {
    //     print!("{rule}, ");
    // }
    // println!("");

    // compute dependency graph using a
    #[derive(Debug, Default, Clone)]
    struct Dependencies {
        comes_before: HashSet<usize>,
        comes_after: HashSet<usize>,
    }
    let mut dependencies: HashMap<usize, Dependencies> = bad_ordering
        .iter()
        .map(|pagenum| {
            (
                *pagenum,
                Dependencies {
                    ..Default::default()
                },
            )
        })
        .collect();
    for rule in filtered_rules.iter() {
        let OrderingRule { first, second } = rule;
        dependencies
            .get_mut(&first)
            .expect("Should be present")
            .comes_before
            .insert(*second);
        dependencies
            .get_mut(&second)
            .expect("Should be present")
            .comes_after
            .insert(*first);
    }

    // see if there is a page that is not dictated to come after any other
    let mut good_ordering: Vec<usize> = Vec::new();
    // let mut rear = VecDeque::new();
    'outer: while dependencies.len() > 0 {
        /*
            println!("Ordered Series: {good_ordering:?}");
            println!("New State for each node:");
            for (pagenum, dep) in dependencies.iter() {
                println!(
                    "  {pagenum}: is_followed_by={:?}, follows={:?}",
                    dep.comes_before, dep.comes_after
                );
            }
        }
         */

        let depclone = dependencies.clone(); // for iteration
        for (pagenum, dep) in depclone {
            // if there's a node with not required to be behind any other node,
            // shove it at the tail of the front.
            if dep.comes_after.is_empty() {
                // println!("");
                // println!("{pagenum} does not follow anything");
                good_ordering.push(pagenum);
                dependencies.remove(&pagenum);
                for follower in dep.comes_before.iter() {
                    let follower = dependencies.get_mut(&follower).unwrap();
                    follower.comes_after.remove(&pagenum);
                }
                continue 'outer;
            }
        }

        panic!("Did a pass with no progress!");
    }

    good_ordering
}

fn fix_page_orderings(rules: &[OrderingRule], bad_orderings: &[&Vec<usize>]) -> Vec<Vec<usize>> {
    bad_orderings
        .into_iter()
        .map(|ordering| fix_page_ordering(rules, &ordering))
        .collect::<Vec<Vec<usize>>>()
}

fn main() -> anyhow::Result<()> {
    part1_and_2()?;
    Ok(())
}
