use std::{collections::{HashSet, VecDeque}, fmt::Display, path::Path, time::Duration};

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
    BoxLeft,
    BoxRight,
    Wall,
}

impl Object {
    fn as_char(&self) -> char {
        match self {
            Self::Empty => '.',
            Self::Robot => '@',
            Self::Wall => '#',
            Self::Box => 'O',
            Self::BoxLeft => '[',
            Self::BoxRight => ']',
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

fn parse_input<P: AsRef<Path>>(path: P, part2: bool) -> anyhow::Result<(Map, Motions)> {
    let mut lines = input_lines(path)?;

    // Parse the map
    let mut map: Vec<Vec<Object>> = Default::default();
    while let Some(line) = lines.next() {
        if line == "" {
            break; // newline, end of map
        }

        let objs = if !part2 {
            line.chars().map(|c| {
                match c {
                    '#' => Object::Wall,
                    '.' => Object::Empty,
                    '@' => Object::Robot,
                    'O' => Object::Box,
                    other => panic!("Unexpected map item {other}"),
                }
            }).collect()
        } else {
            line.chars().flat_map(|c| {
                match c {
                    '#' => &[Object::Wall, Object::Wall],
                    '.' => &[Object::Empty, Object::Empty],
                    '@' => &[Object::Robot, Object::Empty],
                    'O' => &[Object::BoxLeft, Object::BoxRight],
                    other => panic!("Unexpected map item {other}"),
                }
            }).cloned().collect()
        };

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
    let cur = map[box_y][box_x];
    match next_obj {
        Object::Empty => {
            map[box_y][box_x] = Object::Empty;
            map[next_y][next_x] = cur;
            true
        }
        Object::Wall => {
            false
        }
        Object::Robot => {
            panic!("Didn't expect that")
        }
        Object::BoxLeft | Object::BoxRight => {
            if delta_y == 0 {
                // We can treat left/right movement the same as a regular box
                let pushed = shift_boxes(map, next_x, next_y, delta_x, delta_y);
                if pushed {
                    map[box_y][box_x] = Object::Empty;
                    map[next_y][next_x] = cur;
                }
                pushed
            } else {
                panic!("This case not handled here")
            }
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

fn next_position(map: &Map, x: usize, y: usize, delta_x: isize, delta_y: isize) -> (usize, usize) {
    let next_x = (x as isize + delta_x) as usize;
    let next_y = (y as isize + delta_y) as usize;
    (next_x, next_y)
}

fn next_obj(map: &Map, x: usize, y: usize, delta_x: isize, delta_y: isize) -> Object {
    let (next_x, next_y) = next_position(map, x, y, delta_x, delta_y);
    return map[next_y][next_x]
}

fn shiftable_boxes(map: &Map, me_x: usize, me_y: usize, delta_x: isize, delta_y: isize) -> Option<VecDeque<((usize, usize), (usize, usize))>> {
    let mut shiftable = VecDeque::new();
    let me = map[me_y][me_x];

    // at each step, we want to see if both the right and left side of this box can be
    // shifted by the delta_y (only case we're dealing with).
    let (peer_x, peer_y) = match me {
        Object::BoxLeft => (me_x + 1, me_y),
        Object::BoxRight => (me_x - 1, me_y),
        _ => panic!("Unexpected contents!"),
    };

    // Add this position and our peer to the set
    shiftable.push_front(((me_x, me_y), (peer_x, peer_y)));

    match (next_obj(map, me_x, me_y, delta_x, delta_y), next_obj(map, peer_x, peer_y, delta_x, delta_y)) {
        (Object::Empty, Object::Empty) => {
            // there's space to move into, huzzah!
            return Some(shiftable)
        }
        (Object::Wall, _) | (_, Object::Wall) => {
            // dead end, bubble up that we cannot be pushed
            return None;
        }
        (Object::BoxLeft | Object::BoxRight, Object::BoxLeft | Object::BoxRight) => {
            // hit both boxes sides
            let (me_next_x, me_next_y) = next_position(map, me_x, me_y, delta_x, delta_y);
            let me_shiftable = shiftable_boxes(map, me_next_x, me_next_y, delta_x, delta_y);

            let (peer_next_x, peer_next_y) = next_position(map, peer_x, peer_y, delta_x, delta_y);
            let peer_shiftable = shiftable_boxes(map, peer_next_x, peer_next_y, delta_x, delta_y);

            let (Some(me_children), Some(peer_children)) = (me_shiftable, peer_shiftable) else {
                return None; // one subtree was a dead end
            };

            // push the items lower down in the tree onto the front of our deque
            for item in me_children.into_iter().chain(peer_children) {
                shiftable.push_front(item);
            }
        }
        (Object::BoxLeft | Object::BoxRight, Object::Empty) => {
            // me has a box but not peer
            let (me_next_x, me_next_y) = next_position(map, me_x, me_y, delta_x, delta_y);
            let me_shiftable = shiftable_boxes(map, me_next_x, me_next_y, delta_x, delta_y);
            match me_shiftable {
                Some(me_children) => {
                    for item in me_children {
                        shiftable.push_front(item);
                    }
                }
                None => return None,
            }
        }
        (Object::Empty, Object::BoxLeft | Object::BoxRight) => {
            // peer has a box but not me
            let (peer_next_x, peer_next_y) = next_position(map, peer_x, peer_y, delta_x, delta_y);
            let peer_shiftable = shiftable_boxes(map, peer_next_x, peer_next_y, delta_x, delta_y);
            match peer_shiftable {
                Some(set) => {
                    for item in set {
                        shiftable.push_front(item);
                    }
                }
                None => return None
            }
        }
        (other_me, other_peer) => {
            panic!("Unexpected case! me={other_me}, peer={other_peer}");
        }
    }

    Some(shiftable)
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

fn shift(map: &mut Map, x: usize, y: usize, delta_x: isize, delta_y: isize) {
    let (nx, ny) = next_position(map, x, y, delta_x, delta_y);
    map[ny][nx] = map[y][x];
    map[y][x] = Object::Empty;
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
            Object::Box | Object::BoxLeft | Object::BoxRight => {
                // potentially shift box(es) by delta
                if delta_y == 0 {
                    let shifted = shift_boxes(map, next_x, next_y, delta_x, delta_y);
                    if shifted {
                        map[robo.y][robo.x] = Object::Empty;
                        robo.x = next_x;
                        robo.y = next_y;
                        map[robo.y][robo.x] = Object::Robot;
                    }
                } else {
                    // need to do a dfs to see if we can shift
                    if let Some(shiftables) = shiftable_boxes(map, next_x, next_y, delta_x, delta_y) {
                        // shift each of the shiftables down; in theory, at least,
                        // the ordering of the deque we get should mean that the free
                        // spaces end up in the right spot expect for the robot.
                        let mut moved: HashSet<(usize, usize)> = HashSet::new();

                        // this could probably be avoided by doing something less dumb in the recursion
                        // chain but it should work...
                        use itertools::Itertools;
                        let depth_first = shiftables.into_iter().sorted_by_key(|((_, ay), (_, _))| *ay as isize * -delta_y);
                        for ((ax, ay), (bx, by)) in depth_first {
                            if !moved.contains(&(ax, ay)) {
                                shift(map, ax, ay, delta_x, delta_y);
                                moved.insert((ax, ay));
                            }

                            if !moved.contains(&(bx, by)) {
                                shift(map, bx, by, delta_x, delta_y);
                                moved.insert((bx, by));
                            }
                        }

                        map[robo.y][robo.x] = Object::Empty;
                        robo.x = next_x;
                        robo.y = next_y;
                        map[robo.y][robo.x] = Object::Robot;
                    }
                }
            }
            Object::Robot => {
                panic!("Roboception!");
            }
        }

        println!("Enter for next...");
        let mut _s = String::new();
        // std::io::stdin().read_line(&mut _s).unwrap();
        std::thread::sleep(Duration::from_millis(5));
        clear_screen();
        println!("Movement    {movement} ({} / {})", i + 1, movements.len());
        print_map(map);
    }
}

fn compute_gps(map: &Map) -> usize {
    let mut gps_sum: usize = 0;
    for y in 0..map.len() {
        for x in 0..map[y].len() {
            if matches!(map[y][x], Object::Box | Object::BoxLeft) {
                gps_sum += 100 * y + x;
            }
        }
    }
    gps_sum
}

#[allow(unused)]
fn part1() -> anyhow::Result<()> {
    let (mut map, movements) = parse_input("d15.txt", false)?;
    clear_screen();
    println!("Initial Map ({} moves)", movements.len());
    print_map(&map);
    simulate(&mut map, &movements);
    println!("GPS: {}", compute_gps(&map));
    Ok(())
}

fn part2() -> anyhow::Result<()> {
    let (mut map, movements) = parse_input("d15.txt", true)?;
    clear_screen();
    println!("Initial Map ({} moves)", movements.len());
    print_map(&map);
    simulate(&mut map, &movements);
    println!("GPS: {}", compute_gps(&map));
    Ok(())
}

fn main() -> anyhow::Result<()> {
    // part1()?;
    part2()?;
    Ok(())
}
