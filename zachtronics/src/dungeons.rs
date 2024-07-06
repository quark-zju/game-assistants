#![allow(dead_code)]

use std::io;
use std::io::Read;
use std::ops::ControlFlow;

static mut DEBUG: bool = false;
macro_rules! dprintln {
    ($($t:tt)*) => {
        if unsafe { DEBUG } {
            eprintln!($($t)*);
        }
    }
}
macro_rules! or {
    ($e:expr, $($s:tt)*) => {
        match $e {
            Some(v) => v,
            None => $($s)*,
        }
    };
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
enum GoalCell {
    Empty,
    Monster,
    Chest,
}

impl Default for GoalCell {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
enum StateCell {
    Undecided,
    Empty,
    Wall,
}

impl Default for StateCell {
    fn default() -> Self {
        Self::Undecided
    }
}

const N: u8 = 8;
const NS: usize = N as usize;
const DIRECTIONS: [(i8, i8); 4] = [(0, -1), (-1, 0), (1, 0), (0, 1)];
const CHEST_CENTER_OFFSETS: [(i8, i8); 9] = [
    (0, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];
const CHEST_WALL_OFFSETS: [(i8, i8); 12] = [
    (-2, -1),
    (-2, 0),
    (-2, 1),
    (-1, -2),
    (-1, 2),
    (0, -2),
    (0, 2),
    (1, -2),
    (1, 2),
    (2, -1),
    (2, 0),
    (2, 1),
];

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Pos {
    y: u8,
    x: u8,
}

impl Pos {
    fn abs_diff(self, rhs: Pos) -> u8 {
        self.x.abs_diff(rhs.x) + self.y.abs_diff(rhs.y)
    }
    fn checked_add(self, dx: i8, dy: i8) -> Option<Self> {
        let x = (self.x as i8) + dx;
        let y = (self.y as i8) + dy;
        if x < 0 || y < 0 || x >= N as i8 || y >= N as i8 {
            None
        } else {
            Some(Self {
                x: x as _,
                y: y as _,
            })
        }
    }
    fn next(self) -> Option<Self> {
        if self.x + 1 == N {
            if self.y + 1 == N {
                None
            } else {
                Some(Self {
                    x: 0,
                    y: self.y + 1,
                })
            }
        } else {
            Some(Self {
                x: self.x + 1,
                y: self.y,
            })
        }
    }
}

#[derive(Debug)]
struct Goal {
    grid: [[GoalCell; NS]; NS],
    sum_columns: [u8; NS],
    sum_rows: [u8; NS],
    // Derived
    monster_positions: Vec<Pos>,
    chest_positions: Vec<Pos>,
    // Config
    multi_solution: bool,
}

#[derive(Clone, Debug)]
struct State<'a> {
    goal: &'a Goal,
    sum_columns: [u8; NS],
    sum_rows: [u8; NS],
    grid: [[StateCell; NS]; NS],
    last_chest_okay_positions: Vec<Option<Pos>>,

    solution_count: usize,
    search_count: usize,
}

fn char_to_u8(ch: char) -> u8 {
    let v = ch as u8;
    if v >= b'0' && v <= b'9' {
        let v = v - b'0';
        return v;
    }
    return 0;
}

impl Goal {
    fn read_from_ascii(input: &mut dyn Read) -> io::Result<Self> {
        let mut s = String::new();
        let mut sum_columns = [0u8; NS];
        let mut sum_rows = [0u8; NS];
        let mut grid = [[GoalCell::Empty; NS]; NS];
        let mut monster_positions = Vec::new();
        let mut chest_positions = Vec::new();
        input.read_to_string(&mut s)?;
        for (i, line) in s
            .lines()
            .filter(|l| !l.trim().is_empty())
            .enumerate()
            .take(NS + 1)
        {
            if i == 0 {
                for (x, ch) in line.trim().chars().enumerate().take(NS) {
                    sum_columns[x] = char_to_u8(ch);
                }
            } else {
                let y = i - 1;
                sum_rows[y] = char_to_u8(line.chars().next().unwrap_or('0'));
                let row = &line[1..];
                for (x, ch) in row.chars().enumerate().take(NS) {
                    grid[y][x] = match ch {
                        'M' => {
                            monster_positions.push(Pos {
                                y: y as _,
                                x: x as _,
                            });
                            GoalCell::Monster
                        }
                        'C' => {
                            chest_positions.push(Pos {
                                y: y as _,
                                x: x as _,
                            });
                            GoalCell::Chest
                        }
                        _ => GoalCell::Empty,
                    }
                }
            }
        }
        let multi_solution = std::env::var_os("M").is_some();
        let v = Self {
            grid,
            sum_columns,
            sum_rows,
            monster_positions,
            chest_positions,
            multi_solution,
        };
        Ok(v)
    }
    fn get(&self, p: Pos) -> GoalCell {
        self.grid[p.y as usize][p.x as usize]
    }
}

impl<'a> State<'a> {
    fn from_goal(goal: &'a Goal) -> Self {
        Self {
            goal,
            sum_columns: Default::default(),
            sum_rows: Default::default(),
            grid: Default::default(),

            last_chest_okay_positions: vec![None; goal.chest_positions.len()],

            solution_count: 0,
            search_count: 0,
        }
    }

    fn print_grid(&self, pos: Pos) {
        for y in 0..=pos.y {
            let n = if y == pos.y { pos.x + 1 } else { N };
            for x in 0..n {
                let c = match self.get(Pos { x, y }) {
                    StateCell::Undecided => '?',
                    StateCell::Empty => '.',
                    StateCell::Wall => '#',
                };
                eprint!("{}", c);
            }
            eprintln!();
        }
    }

    fn search(&mut self, pos: Pos) -> ControlFlow<(), ()> {
        dprintln!("Search {pos:?}");
        self.search_count += 1;

        let orig_state = self.get(pos);
        let orig_sum_row = self.sum_rows[pos.y as usize];
        let orig_sum_column = self.sum_columns[pos.x as usize];
        'outer_loop: for v in [StateCell::Empty, StateCell::Wall] {
            if v == StateCell::Wall
                && self.goal.grid[pos.y as usize][pos.x as usize] != GoalCell::Empty
            {
                continue;
            }

            self.grid[pos.y as usize][pos.x as usize] = v;
            if v == StateCell::Wall {
                self.sum_rows[pos.y as usize] += 1;
                self.sum_columns[pos.x as usize] += 1;
            }
            if unsafe { DEBUG } {
                self.print_grid(pos);
            }

            // Check sum_rows (incremental)
            {
                let sum = self.sum_rows[pos.y as usize];
                let sum_goal = self.goal.sum_rows[pos.y as usize];
                let rest = (N) - 1 - pos.x;
                if sum + rest < sum_goal || sum > sum_goal {
                    // Bad: Cannot satisfy sum_row.
                    dprintln!(" Bad sum_row {sum}..+{rest} not in {sum_goal}");
                    continue 'outer_loop;
                }
            }

            // Check sum_columns (incremental)
            {
                let sum = self.sum_columns[pos.x as usize];
                let sum_goal = self.goal.sum_columns[pos.x as usize];
                let rest = (N) - 1 - pos.y;
                if sum + rest < sum_goal || sum > sum_goal {
                    // Bad: Cannot satisfy sum_columns.
                    dprintln!(" Bad sum_col {sum}..+{rest} not in {sum_goal}");
                    continue 'outer_loop;
                }
            }

            // Check hallway (incremental)
            {
                if v == StateCell::Empty && pos.x >= 1 && pos.y >= 1 {
                    let sum = self.sum_area(pos.x - 1, pos.y - 1, pos.x, pos.y);
                    if sum == 0 {
                        // Is it part of a chest room?
                        let mut in_chest_room = false;
                        for cpos in &self.last_chest_okay_positions {
                            let cpos = or!(cpos, continue);
                            if pos.x >= cpos.x
                                && pos.y >= cpos.y
                                && pos.x <= cpos.x + 1
                                && pos.y <= cpos.y + 1
                            {
                                in_chest_room = true;
                                break;
                            }
                        }
                        // TODO: Consider other chest locations (is it needed?)
                        if !in_chest_room {
                            dprintln!(" Bad hallway {pos:?}");
                            continue 'outer_loop;
                        }
                    }
                }
            }

            // Check monstor (incremental)
            {
                for p in &self.goal.monster_positions {
                    let d = p.abs_diff(pos);
                    if d == 1 {
                        let mut known_empty = 0;
                        let mut known_block = 0;
                        for (dx, dy) in DIRECTIONS {
                            match p.checked_add(dx, dy) {
                                None => known_block += 1,
                                Some(p) => match self.get(p) {
                                    StateCell::Empty => known_empty += 1,
                                    StateCell::Wall => known_block += 1,
                                    _ => (),
                                },
                            }
                        }
                        if known_empty > 1 || known_block == 4 {
                            dprintln!(" Bad monstor {p:?}");
                            continue 'outer_loop;
                        }
                    }
                }
            }

            // Check dead end (incremental)
            //  .
            // .#
            {
                for (dx, dy) in [(-1, 0), (0, -1)] {
                    if let Some(p) = pos.checked_add(dx, dy) {
                        if self.goal.get(p) != GoalCell::Monster && self.is_dead_end(p) {
                            dprintln!(" Bad dead end {p:?}");
                            continue 'outer_loop;
                        }
                    }
                }
            }

            // Check chest area.
            {
                for (i, cpos) in self.goal.chest_positions.iter().enumerate() {
                    if !(pos.x + 3 >= cpos.x
                        && pos.x <= cpos.x + 3
                        && pos.y + 3 >= cpos.y
                        && pos.y <= cpos.y + 3)
                    {
                        // No need to check - change out of chest range.
                        continue;
                    }

                    let last_okay_pos = self.last_chest_okay_positions[i];
                    if let Some(cpos) = last_okay_pos {
                        if self.is_chest_room_valid(cpos) {
                            // Still valid - no need to check each position.
                            dprintln!("  Chest {i} still looks okay");
                            continue;
                        }
                    }

                    let mut chest_okay = true;
                    for (dx, dy) in CHEST_CENTER_OFFSETS {
                        let cpos = or!(cpos.checked_add(dx, dy), continue);
                        dprintln!(" Check chest {i} {cpos:?} offset {dx} {dy}");
                        chest_okay = false;
                        if self.is_chest_room_valid(cpos) {
                            dprintln!("  Chest {i} looks okay");
                            self.last_chest_okay_positions[i] = Some(cpos);
                            chest_okay = true;
                            break;
                        }
                    }

                    if !chest_okay {
                        dprintln!(" Bad chest {i} {cpos:?}");
                        continue 'outer_loop;
                    }
                }
            }

            if let Some(next_pos) = pos.next() {
                self.search(next_pos)?;
            } else if self.is_hallway_connected() {
                eprintln!("Found solution:");
                self.solution_count += 1;
                self.print_grid(pos);
                if !self.goal.multi_solution {
                    return ControlFlow::Break(());
                }
            }
        }

        self.sum_rows[pos.y as usize] = orig_sum_row;
        self.sum_columns[pos.x as usize] = orig_sum_column;
        self.grid[pos.y as usize][pos.x as usize] = orig_state;
        ControlFlow::Continue(())
    }

    fn is_hallway_connected(&self) -> bool {
        // Find an empty cell to start flood fill.
        let mut expected_count = 0;
        let mut to_visit = Vec::new();
        for y in 0..N {
            for x in 0..N {
                let p = Pos { x, y };
                if self.get(p) == StateCell::Empty {
                    if to_visit.is_empty() {
                        to_visit.push(p);
                    }
                    expected_count += 1;
                }
            }
        }
        // Flood fill.
        let mut connected = [[false; NS]; NS];
        let mut actual_count = 0;
        while let Some(pos) = to_visit.pop() {
            if connected[pos.y as usize][pos.x as usize] {
                continue;
            }
            connected[pos.y as usize][pos.x as usize] = true;
            actual_count += 1;
            for (dy, dx) in DIRECTIONS {
                let dpos = or!(pos.checked_add(dx, dy), continue);
                if self.get(dpos) == StateCell::Empty {
                    to_visit.push(dpos);
                }
            }
        }
        if actual_count != expected_count {
            dprintln!(" Not connected: {actual_count} != {expected_count}");
            return false;
        }
        true
    }

    fn is_dead_end(&self, p: Pos) -> bool {
        if self.get(p) == StateCell::Wall {
            return false;
        }
        let mut block = 0;
        for (dx, dy) in DIRECTIONS {
            match p.checked_add(dx, dy) {
                None => block += 1,
                Some(p) => {
                    if self.get(p) == StateCell::Wall {
                        block += 1;
                    }
                }
            }
        }
        block >= 3
    }

    fn get(&self, p: Pos) -> StateCell {
        self.grid[p.y as usize][p.x as usize]
    }

    // Count walls
    fn sum_area(&self, x1: u8, y1: u8, x2: u8, y2: u8) -> u8 {
        let mut sum = 0;
        for y in y1..=y2 {
            for x in x1..=x2 {
                sum += (self.grid[y as usize][x as usize] == StateCell::Wall) as u8;
            }
        }
        sum
    }

    // Check chest room. cpos is the center.
    fn is_chest_room_valid(&self, cpos: Pos) -> bool {
        let left_top = or!(cpos.checked_add(-1, -1), return false);
        let bottom_right = or!(cpos.checked_add(1, 1), return false);

        // No wall in the left_top to bottom_right region.
        for y in left_top.y..=bottom_right.y {
            for x in left_top.x..=bottom_right.x {
                let p = Pos { x, y };
                if self.get(p) == StateCell::Wall {
                    dprintln!("  Wall at {p:?}");
                    return false;
                }
            }
        }

        // Count surrounding walls.
        let mut openings = 0;
        let mut undecided = 0;
        for (dy, dx) in CHEST_WALL_OFFSETS {
            let pos = or!(cpos.checked_add(dx, dy), continue);
            match self.get(pos) {
                StateCell::Undecided => undecided += 1,
                StateCell::Empty => openings += 1,
                StateCell::Wall => {}
            }
        }
        // Too many openings.
        if openings > 1 {
            dprintln!("  Too many openings {openings}");
            return false;
        }
        // Fully closed.
        if openings == 0 && undecided == 0 {
            dprintln!("  No openings");
            return false;
        }
        true
    }
}

pub(crate) fn main() {
    unsafe {
        DEBUG = std::env::var_os("D").is_some();
    }
    let mut input = std::io::stdin();
    let goal = Goal::read_from_ascii(&mut input).unwrap();
    // dbg!(&goal);
    let mut state = State::from_goal(&goal);
    state.search(Pos { x: 0, y: 0 });
    if goal.multi_solution {
        let c = state.solution_count;
        eprintln!("Found {c} solution(s).");
    }
    eprintln!("Searches: {}", state.search_count);
}

#[test]
fn test_pos_ord() {
    assert!(Pos { x: 1, y: 0 } > Pos { x: 0, y: 0 });
    assert!(Pos { x: 0, y: 1 } > Pos { x: 0, y: 0 });
    assert!(Pos { x: 0, y: 1 } > Pos { x: 1, y: 0 });
}
