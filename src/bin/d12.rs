use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

fn parse_input<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<Vec<char>>> {
    let full_path = PathBuf::from(".")
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

#[derive(Debug, Clone)]
struct CropArea {
    _crop: char,
    members: HashSet<(usize, usize)>,
    row_count: usize,
    col_count: usize,
}

const NEIGHBOR_OFFSETS: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Side {
    offset_x: isize,
    offset_y: isize,
    origin_x: usize,
    origin_y: usize,
}

impl CropArea {
    fn has_perimeter_at_offset(&self, x: usize, y: usize, x_off: isize, y_off: isize) -> bool {
        if !self.members.contains(&(x, y)) {
            return false;
        }

        let (neigh_x, neigh_y) =
            (x.checked_add_signed(x_off), y.checked_add_signed(y_off));
        match (neigh_x, neigh_y) {
            (Some(nx), Some(ny)) if nx < self.row_count && ny < self.col_count => {
                !self.members.contains(&(nx, ny))
            }
            _ => true, // all other cases this is a perimeter wall
        }
    }

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
            let count = NEIGHBOR_OFFSETS
                .iter()
                .filter(|(x_off, y_off)| {
                    self.has_perimeter_at_offset(x, y, *x_off, *y_off)
                })
                .count();
            perimeter += count;
        }

        perimeter
    }

    fn price(&self) -> usize {
        self.area() * self.perimeter()
    }

    fn bulk_price(&self) -> usize {
        self.sides() * self.area()
    }

    fn sides(&self) -> usize {
        // for the bulk price, we multiple the area by the number of "sides" that
        // are continguous.  To count this algorithmically we'll consider that there
        // are two tipes of sides, horizontal and vertical.
        //
        // A side can be uniquely identified by the combination of:
        // 1. The direction and row/column combination
        // 2. It's origin point where we consider the leftmost row/col to be
        //    the origin of a horizontal feature and the topmost row/col to
        //    be the origin of a vertical feature.
        let mut sides: HashSet<Side> = HashSet::new();
        for (x, y) in self.members.iter().cloned() {
            for (x_off, y_off) in NEIGHBOR_OFFSETS {
                if !self.has_perimeter_at_offset(x, y, x_off, y_off) {
                    continue;
                }

                // so, we know there's a side here -- we need to drill in to find
                // the origin of this side to see if it already exits or we need to add
                // add to our accounting.
                let mut origin = (x, y);
                if x_off != 0 {
                    // vertical
                    let mut cand_y = y;
                    loop {
                        if !self.has_perimeter_at_offset(x, cand_y, x_off, y_off) {
                            break;
                        }

                        origin = (x, cand_y);
                        cand_y = match cand_y.checked_add_signed(-1) {
                            Some(v) => v,
                            None => break,
                        };
                    }
                } else {
                    // horizontal
                    let mut cand_x = x;
                    loop {
                        if !self.has_perimeter_at_offset(cand_x, y, x_off, y_off) {
                            break;
                        }

                        origin = (cand_x, y);
                        cand_x = match cand_x.checked_add_signed(-1) {
                            Some(v) => v,
                            None => break,
                        };
                    }
                }

                // NOTE: there is opportunity for memoization at a few places in these equations.
                // Now, see if we need to do an additian
                let side = Side {
                    offset_x: x_off,
                    offset_y: y_off,
                    origin_x: origin.0,
                    origin_y: origin.1,
                };
                sides.insert(side);
            }
        }

        sides.len()
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
                _crop: crop,
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
    let crop_areas = find_crop_areas(&plots);
    let total_price: usize = crop_areas.iter().map(|ca| ca.price()).sum();
    println!("Total Price: {total_price}");

    let bulk_price: usize = crop_areas.iter().map(|ca| ca.bulk_price()).sum();
    println!("Bulk Price: {bulk_price}"); // 802799 is too low

    Ok(())
}
