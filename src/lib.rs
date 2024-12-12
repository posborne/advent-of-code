use std::fmt::Display;

#[allow(unused)]
pub fn print_2darr<T>(data: &[Vec<T>])
    where T: Display
{
    for row in data.iter() {
        for item in row.iter() {
            print!("{item}");
        }
        println!("");
    }
}
