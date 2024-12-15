use std::{fmt::Display, path::Path};

use aoc::input_lines;

#[derive(Debug, Clone, Copy)]
struct Position {
    x: usize,
    y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Object {
    Empty,
    Robot,
    Box,
    Wall,
}

impl Object {
    fn as_char(&self) -> char {
        match self {
            Self::Empty => '.',
            Self::Robot => '@',
            Self::Wall => '#',
            Self::Box => 'O',
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

enum Movement {
    Up,
    Down,
    Left,
    Right,
}

impl Movement {
    fn from_char(c: char) -> Movement {
        match c {
            '^' => Movement::Up,
            '<' => Movement::Left,
            '>' => Movement::Right,
            'v' => Movement::Down,
            other => panic!("Illegal motion {other}"),
        }
    }
}

impl Display for Movement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Movement::Up => '^',
            Movement::Down => 'v',
            Movement::Left => '<',
            Movement::Right => '>',
        })
    }
}

type Map = Vec<Vec<Object>>;
type Motions = Vec<Movement>;

fn parse_input<P: AsRef<Path>>(path: P) -> anyhow::Result<(Map, Motions)> {
    let mut lines = input_lines(path)?;

    // Parse the map
    let mut map: Vec<Vec<Object>> = Default::default();
    while let Some(line) = lines.next() {
        if line == "" {
            break; // newline, end of map
        }

        let objs: Vec<Object> = line.chars().map(|c| {
            match c {
                '#' => Object::Wall,
                '.' => Object::Empty,
                '@' => Object::Robot,
                'O' => Object::Box,
                other => panic!("Unexpected map item {other}"),
            }
        }).collect();
        map.push(objs);
    }

    // Parse the robot directions
    let mut movements = Vec::new();
    for movements_line in lines {
        movements.extend(movements_line.chars().map(|c| Movement::from_char(c)));
    }

    Ok((map, movements))
}

fn find_robot(map: &Map) -> Position {
    for (row_idx, row) in map.iter().enumerate() {
        for (col_idx, obj) in row.iter().enumerate() {
            if *obj == Object::Robot {
                return Position {
                    x: col_idx,
                    y: row_idx,
                }
            }
        }
    }

    panic!("Where's our robot?");
}

fn shift_boxes(map: &mut Map, box_x: usize, box_y: usize, delta_x: isize, delta_y: isize) -> bool {
    let next_x = (box_x as isize + delta_x) as usize;
    let next_y = (box_y as isize + delta_y) as usize;
    let next_obj = map[next_y][next_x];
    match next_obj {
        Object::Empty => {
            map[box_y][box_x] = Object::Empty;
            map[next_y][next_x] = Object::Box;
            true
        }
        Object::Wall => {
            false
        }
        Object::Robot => {
            panic!("Didn't expect that")
        }
        Object::Box => {
            let pushed = shift_boxes(map, next_x, next_y, delta_x, delta_y);
            if pushed {
                map[box_y][box_x] = Object::Empty;
                map[next_y][next_x] = Object::Box;
            }
            pushed
        }
    }
}

fn clear_screen() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

fn print_map(map: &Map) {
    for row in map.iter() {
        for obj in row {
            print!("{obj}");
        }
        println!("");
    }
}

fn simulate(map: &mut Map, movements: &[Movement]) {
    let mut robo = find_robot(map);
    for (i, movement) in movements.iter().enumerate() {
        let (delta_x, delta_y) = match movement {
            Movement::Up => (0, -1),
            Movement::Down => (0, 1),
            Movement::Left => (-1, 0),
            Movement::Right => (1, 0),
        };

        let next_y = (robo.y as isize + delta_y) as usize;
        let next_x = (robo.x as isize + delta_x) as usize;
        let obj_at_next_pos = map[next_y][next_x];
        match obj_at_next_pos {
            Object::Empty => {
                map[robo.y][robo.x] = Object::Empty;
                robo.x = next_x;
                robo.y = next_y;
                map[robo.y][robo.x] = Object::Robot;
            }
            Object::Wall => {
                // do nothing; robot doesn't get to move.
            }
            Object::Box => {
                // potentially shift box(es) by delta
                let shifted = shift_boxes(map, next_x, next_y, delta_x, delta_y);
                if shifted {
                    map[robo.y][robo.x] = Object::Empty;
                    robo.x = next_x;
                    robo.y = next_y;
                    map[robo.y][robo.x] = Object::Robot;
                }
            }
            Object::Robot => {
                panic!("Roboception!");
            }
        }

        println!("Enter for next...");
        let mut _s = String::new();
        // std::io::stdin().read_line(&mut _s).unwrap();
        clear_screen();
        println!("Movement    {movement} ({} / {})", i + 1, movements.len());
        print_map(map);
    }
}

fn compute_gps(map: &Map) -> usize {
    let mut gps_sum: usize = 0;
    for y in 0..map.len() {
        for x in 0..map[y].len() {
            if map[y][x] == Object::Box {
                gps_sum += 100 * y + x;
            }
        }
    }
    gps_sum
}

fn main() -> anyhow::Result<()> {
    let (mut map, movements) = parse_input("d15.txt")?;
    clear_screen();
    println!("Initial Map ({} moves)", movements.len());
    print_map(&map);
    simulate(&mut map, &movements);

    println!("GPS: {}", compute_gps(&map));

    Ok(())
}
