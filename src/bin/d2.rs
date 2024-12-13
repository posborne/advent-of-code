use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

fn parse_input<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<Vec<i32>>> {
    let input_path = PathBuf::from(".")
        .join("inputs")
        .join(path);
    let f = File::open(input_path)?;
    let reader = BufReader::new(f);
    let records = reader
        .lines()
        .filter_map(|l| Some(l.ok()?))
        .map(|l| {
            l.split_whitespace()
                .filter_map(|chunk| chunk.parse::<i32>().ok()?.into())
                .collect::<Vec<i32>>()
        })
        .collect::<Vec<Vec<i32>>>();
    Ok(records)
}

mod p1 {
    fn is_report_safe_increasing(report: &[i32]) -> bool {
        let mut prev = report[0];
        for &cur in &report[1..] {
            let delta = cur - prev;
            if delta <= 0 || delta > 3 {
                return false;
            }
            prev = cur;
        }

        return true;
    }

    fn is_report_safe_decreasing(report: &[i32]) -> bool {
        let mut prev = report[0];
        for &cur in &report[1..] {
            let delta = cur - prev;
            if delta >= 0 || delta < -3 {
                return false;
            }
            prev = cur;
        }

        return true;
    }

    fn is_report_safe(report: &[i32]) -> bool {
        // The levels are either all increasing or all decreasing.
        // Any two adjacent levels differ by at least one and at most three.
        if report[1] > report[0] {
            is_report_safe_increasing(report)
        } else {
            is_report_safe_decreasing(report)
        }
    }

    pub fn part1() -> anyhow::Result<()> {
        let records = super::parse_input("d2-p1.txt")?;
        let safe_count = records
            .into_iter()
            .filter(|r| is_report_safe(r.as_slice()))
            .count();

        println!("Safe Count: {safe_count}");
        Ok(())
    }
}

mod p2 {

    fn check_series<'a>(mut series: impl Iterator<Item = &'a i32> + Clone) -> bool {
        let mut increasing: Option<bool> = None;
        let mut prev = match series.next() {
            Some(v) => v,
            None => return true,
        };
        while let Some(cur) = series.next() {
            let is_increasing = increasing.get_or_insert_with(|| cur > prev);
            let delta = if *is_increasing { cur - prev } else { prev - cur };
            if delta <= 0 || delta > 3 {
                return false;
            }
            prev = cur;
        }

        return true; // no failure case found
    }

    fn is_report_safe_fault_tolerant(report: &[i32]) -> bool {
        // fuck it, we'll do it live; just try every permutation combination
        // of the report series starting with the base case and then the
        // ones with one element removed.

        if check_series(report.iter()) {
            return true;
        }

        for i in 0..report.len() {
            let series = report[0..i].iter().chain(&report[i + 1..]);
            let res = check_series(series.clone());
            if res {
                return true;
            }
        }

        return false;
    }

    pub fn part2() -> anyhow::Result<()> {
        let reports = super::parse_input("d2-p1.txt")?;
        let mut pass = 0;
        for report in reports {
            let res = is_report_safe_fault_tolerant(&report);
            if res {
                pass += 1;
            }
        }

        println!("Safe: {pass}");
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    p1::part1()?;
    p2::part2()?;
    Ok(())
}
