// Example Input:
// 190: 10 19
// 3267: 81 40 27
// 83: 17 5
// 156: 15 6
// 7290: 6 8 6 15
// 161011: 16 10 13
// 192: 17 8 14
// 21037: 9 7 18 13
// 292: 11 6 16 20

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

#[derive(Debug, Copy, Clone)]
enum Operator {
    Mul,
    Add,
    Concat,
}

#[derive(Debug, Clone)]
struct Input {
    result: u64,
    operands: Vec<u64>,
}

fn operator_permutations(operands_len: usize, with_concat: bool) -> Vec<Vec<Operator>> {
    // TODO: could definitely memoize or make assumptions that it is the same for the run...
    let mut permutations = Vec::new();
    if operands_len == 1 {
        return permutations; // base case
    }

    if operands_len == 2 {
        permutations.push(Vec::from(&[Operator::Add]));
        permutations.push(Vec::from(&[Operator::Mul]));
        if with_concat {
            permutations.push(Vec::from(&[Operator::Concat]));
        }
        return permutations;
    }

    // compute the next two combos and then recurse
    let rem = operator_permutations(operands_len - 1, with_concat);
    for perm in rem {
        let mut add_perm = Vec::from(&[Operator::Add]);
        let mut mul_perm = Vec::from(&[Operator::Mul]);
        add_perm.extend(perm.iter());
        mul_perm.extend(perm.iter());
        permutations.push(add_perm);
        permutations.push(mul_perm);
        if with_concat {
            let mut concat_perm = Vec::from(&[Operator::Concat]);
            concat_perm.extend(perm.iter());
            permutations.push(concat_perm);
        }
    }

    //println!("{operands_len} - {permutations:?}");
    permutations
}

impl Input {
    fn compute_operators(&self, with_concat: bool) -> Vec<Vec<Operator>> {
        let mut successful: Vec<Vec<Operator>> = Vec::new();
        //println!("");
        'ordering: for op_ordering in operator_permutations(self.operands.len(), with_concat) {
            let mut computed_res = self.operands[0];
            // print!("{}: {computed_res}", self.result);
            for (operand, operator) in self.operands[1..].iter().zip(&op_ordering) {
                match operator {
                    Operator::Add => {
                        //print!(" + {operand}");
                        computed_res = match computed_res.checked_add(*operand) {
                            Some(res) => res,
                            None => continue 'ordering, // overflow
                        };
                    }
                    Operator::Mul => {
                        //print!(" * {operand}");
                        computed_res = match computed_res.checked_mul(*operand) {
                            Some(res) => res,
                            None => continue 'ordering, // overflow
                        }
                    }
                    Operator::Concat => {
                        let concatted_str = format!("{computed_res}{operand}");
                        computed_res = match concatted_str.parse::<u64>() {
                            Ok(v) => v,
                            Err(e) => {
                                eprintln!("{e} - {concatted_str:?}");
                                continue 'ordering;
                            } // overflow probably
                        };
                    }
                }
            }

            //println!(" = {computed_res}");
            if computed_res == self.result {
                successful.push(op_ordering);
            }
        }

        successful
    }
}

fn parse_input<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<Input>> {
    let full_path = PathBuf::from(".").join("inputs").join(path);
    let f = File::open(full_path)?;
    let reader = BufReader::new(f);
    let parsed_inputs = reader
        .lines()
        .filter_map(|l| l.ok())
        .map(|l| {
            let (l, r) = l.split_once(":").expect("SplitOnce");
            let result = l.parse::<u64>().expect("Parse reuslt");
            let operands = r
                .split_whitespace()
                .filter_map(|o| {
                    o.parse::<u64>()
                        .inspect_err(|e| panic!("Operand parse error: {e:?}"))
                        .ok()
                })
                .collect::<Vec<u64>>();
            Input { result, operands }
        })
        .collect();
    Ok(parsed_inputs)
}

fn main() -> anyhow::Result<()> {
    let parsed_inputs = parse_input("d7-p1.txt")?;
    let functional_res_sum: u64 = parsed_inputs
        .iter()
        .filter(|i| i.compute_operators(false).len() > 0)
        .map(|i| {
            // println!("Good -> {i:?}");
            i.result
        })
        .sum();
    println!("Part1 - Functional Sum: {functional_res_sum:?}");

    let functional_res_sum: u64 = parsed_inputs
        .iter()
        .filter(|i| i.compute_operators(true).len() > 0)
        .map(|i| {
            // println!("Good -> {i:?}");
            i.result
        })
        .sum();
    println!("Part 2 - Functional Sum: {functional_res_sum:?}");

    Ok(())
}
