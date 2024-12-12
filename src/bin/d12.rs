use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use aoc::print_2darr;

fn parse_input<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<Vec<char>>> {
    let full_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("inputs")
        .join(path);
    let f = File::open(full_path)?;
    let reader = BufReader::new(f);
    let data = reader
        .lines()
        .filter_map(|l| l.ok())
        .map(|l| l.chars().collect())
        .collect();
    Ok(data)
}

struct CropArea {
    crop: char,
    members: HashSet<(usize, usize)>,
    row_count: usize,
    col_count: usize,
}

impl CropArea {
    fn area(&self) -> usize {
        return self.members.len();
    }

    fn perimeter(&self) -> usize {
        // start naive; look at each and see if it has neighbors in each of the
        // four directions.  This is quadratic and might be a problem for
        // part 2 (I am assuming).  For now, let's do it dumb and see how that
        // plays out...
        let mut perimeter = 0;
        for (x, y) in self.members.iter().cloned() {
            let count = [(-1, 0), (1, 0), (0, -1), (0, 1)]
                .iter()
                .filter(|(x_off, y_off)| {
                    let (neigh_x, neigh_y) =
                        (x.checked_add_signed(*x_off), y.checked_add_signed(*y_off));
                    match (neigh_x, neigh_y) {
                        (Some(nx), Some(ny)) if nx < self.row_count && ny < self.col_count => {
                            !self.members.contains(&(nx, ny))
                        }
                        _ => true, // all other cases this is a perimeter wall
                    }
                })
                .count();
            perimeter += count;
        }

        perimeter
    }

    fn price(&self) -> usize {
        self.area() * self.perimeter()
    }
}

fn find_adjacent_crops(
    plot: &[Vec<char>],
    area_crop: char,
    row_idx: usize,
    col_idx: usize,
    found: &mut HashSet<(usize, usize)>,
) {
    let this_crop = plot[row_idx][col_idx];
    if this_crop != area_crop {
        return;
    }

    found.insert((row_idx, col_idx));
    for (row_offset, col_offset) in [(-1, 0), (1, 0), (0, 1), (0, -1)] {
        let next_row_idx = row_idx.checked_add_signed(row_offset);
        let next_col_idx = col_idx.checked_add_signed(col_offset);
        if let (Some(next_row_idx), Some(next_col_idx)) = (next_row_idx, next_col_idx) {
            if next_row_idx < plot.len() && next_col_idx < plot[0].len() {
                if !found.contains(&(next_row_idx, next_col_idx)) {
                    // recurse on our neighbors
                    find_adjacent_crops(plot, area_crop, next_row_idx, next_col_idx, found);
                }
            }
        }
    }
}

// iterate through the plot
fn find_crop_areas(plot: &[Vec<char>]) -> Vec<CropArea> {
    let row_count = plot.len();
    let col_count = plot[0].len();
    let mut crop_areas: Vec<CropArea> = Vec::new();
    for row_idx in 0..row_count {
        for col_idx in 0..col_count {
            let crop = plot[row_idx][col_idx];
            // if this position is already accounted for, move past it
            if crop_areas
                .iter()
                .find(|ca| ca.members.contains(&(row_idx, col_idx)))
                .is_some()
            {
                continue;
            }

            // we have a new croparea, let's find our friends
            let mut crop_members = HashSet::new();
            find_adjacent_crops(plot, crop, row_idx, col_idx, &mut crop_members);
            crop_areas.push(CropArea {
                crop,
                members: crop_members,
                row_count,
                col_count,
            })
        }
    }

    crop_areas
}

fn main() -> anyhow::Result<()> {
    let plots = parse_input("d12.txt")?;
    // print_2darr(&plots);
    let crop_areas = find_crop_areas(&plots);
    let mut total_price = 0;
    for ca in crop_areas {
        // let area = ca.area();
        // let perimeter = ca.perimeter();
        let price = ca.price();
        total_price += price;
        // println!("{}: area={area}, perimeter={perimeter}, price={price}", ca.crop);
    }
    println!("Total Price: {total_price}");
    Ok(())
}
