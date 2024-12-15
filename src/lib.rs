use std::{
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

#[allow(unused)]
pub fn print_2darr<T>(data: &[Vec<T>])
where
    T: Display,
{
    for row in data.iter() {
        for item in row.iter() {
            print!("{item}");
        }
        println!("");
    }
}

#[allow(unused)]
pub fn input_lines<P>(path: P) -> anyhow::Result<impl Iterator<Item = String>>
where
    P: AsRef<Path>,
{
    let full_path = PathBuf::from("inputs").join(path);
    let f = File::open(full_path)?;
    let reader = BufReader::new(f);
    let iter = reader.lines().filter_map(|l| {
        l.inspect_err(|e| eprintln!("Unexpected error reading input lines: {e:?}"))
            .ok()
    });
    Ok(iter)
}
