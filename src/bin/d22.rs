use std::{path::Path};

use aoc::input_lines;
use clap::Parser;

fn parse_input<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<usize>> {
    Ok(input_lines(path)?.map(|l| {
        l.parse::<usize>().unwrap()
    }).collect())
}

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short, long)]
    input: String,

    #[arg(short, long, default_value = None)]
    secret: Option<usize>,
}

fn mix(secret: usize, number: usize) -> usize {
    number ^ secret
}

fn prune(secret: usize) -> usize {
    secret % 16777216
}

fn simulate(base_secret: usize, generations: usize) -> usize {
    let mut secret = base_secret;
    for _gen in 0..generations {
        // mul 64
        let mut value = secret * 64;
        secret = prune(mix(value, secret));

        // div 32
        value = secret / 32;
        secret = prune(mix(value, secret));

        // mul 2048
        value = secret * 2048;
        secret = prune(mix(value, secret));
    }

    secret
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    if let Some(secret) = cli.secret {
        let res = simulate(secret, 10);
        println!("Final Generation: {res}");
        return Ok(());
    }
    let input = parse_input(cli.input)?;
    let mut sum_of_secrets = 0;
    for secret in input {
        let nth_secret = simulate(secret, 2000);
        sum_of_secrets += nth_secret;
        println!("{secret}: {nth_secret}");
    }
    println!("Sum: {sum_of_secrets}");
    Ok(())
}
