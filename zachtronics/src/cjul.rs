//! The Zachtronis Solitaire Collection - Cluj Solitaire
//! Solver.
//!
//! Input example:
//!
//! 7  v 8  k 6  k
//! 10 t 7  7 7  8
//! d  9 v  9 k  8
//! d  d 9 10 9  t
//! 6  v t 10 10 6
//! t  d k  8 6  v
//!
//! Spaces and '1' are optional.

use crate::dprintln;
use crate::or;
use crate::util::is_verbose;
use crate::util::NVec;

use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Write;
use std::hash::Hash;
use std::ops::Add;
use std::ops::Sub;

#[derive(Copy, Clone, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Span(Card, u8);

#[derive(Copy, Clone, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Card(u8);

#[derive(Copy, Clone, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Column(NVec<Span, 6>, Option<Card>);

#[derive(Copy, Clone, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
struct State([Column; 6]);

/// How to transfer one state to another State.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct TransferStep {
    from_column: Column,
    to_column: Column,
    card_count: u8,
    to_slot: bool,
    // Is this really useful?
    step_score: u8,
}

impl TransferStep {
    fn explain(&self, state: &State) -> String {
        let (col1, col2) = self.find_col1_col2(state);
        let cards = state.0[col1 as usize].explain_last_n_cards(self.card_count);
        format!(
            "Move {} from {} -> {}. {} to {}.",
            cards,
            col1 + 1,
            col2 + 1,
            self.from_column,
            self.to_column
        )
    }

    fn apply(&self, state: &mut State) {
        let (col1, col2) = self.find_col1_col2(state);
        state.unchecked_apply_moving_to(col1, col2, self.card_count, self.to_slot, 0);
    }

    fn find_col1_col2(&self, state: &State) -> (u8, u8) {
        let col1 = state.find_column_index(&self.from_column, None);
        let col2 = state.find_column_index(&self.to_column, Some(col1));
        (col1, col2)
    }
}

const CARD_STRS: [&str; 9] = ["6", "7", "8", "9", "10", "V", "D", "K", "T"];

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = CARD_STRS[self.0 as usize];
        f.write_str(s)
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.len() {
            1 => self.0.fmt(f),
            _ => write!(f, "{:?}-{:?}", self.top(), self.bottom()),
        }
    }
}

impl fmt::Debug for Column {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)?;
        if let Some(c) = self.1 {
            write!(f, "+{:?}", c)?;
        }
        Ok(())
    }
}

impl fmt::Display for Column {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_dead() {
            return f.write_str("[..]");
        }
        let mut cards = Vec::new();
        for span in self.0.as_ref() {
            for i in (span.bottom().0..=span.top().0).rev() {
                cards.push(format!("{:?}", Card(i)));
            }
        }
        if let Some(c) = self.1 {
            cards.push(format!("{:?}", c));
        }
        let card_str = cards.join(" ");
        write!(f, "[{}]", card_str)
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('[')?;
        for (i, column) in self.0.iter().enumerate() {
            if i != 0 {
                f.write_char(' ')?;
            }
            column.fmt(f)?;
        }
        f.write_char(']')
    }
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Card {
    fn from_str(s: &str) -> Card {
        let s = s.to_ascii_uppercase();
        let v = match s.as_str() {
            "6" => 0,
            "7" => 1,
            "8" => 2,
            "9" => 3,
            "10" | "0" | "1" => 4,
            "V" => 5,
            "D" => 6,
            "K" => 7,
            "T" => 8,
            _ => panic!("unknwon card str: {}", s),
        };
        Self(v)
    }
}

impl Sub<u8> for Card {
    type Output = Self;

    fn sub(self, rhs: u8) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl Add<u8> for Card {
    type Output = Self;

    fn add(self, rhs: u8) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Span {
    fn top(self) -> Card {
        self.0
    }
    fn len(self) -> u8 {
        self.1
    }
    fn bottom(self) -> Card {
        Card(self.0 .0 + 1 - self.1)
    }
    fn can_accept_card(self, card: Card) -> bool {
        card + 1 == self.bottom()
    }
    fn accept_span_size(self, span: Span) -> u8 {
        let b = self.bottom();
        let c = if b > span.bottom() && span.top() + 1 >= b {
            b.0 - span.bottom().0
        } else {
            0
        };
        c
    }
    fn extend_size(&mut self, len: u8) {
        self.1 += len;
    }
    fn shrink_size(&mut self, len: u8) {
        self.1 -= len;
    }
    fn from_card(card: Card) -> Self {
        Span(card, 1)
    }
}

impl Column {
    fn is_dead(&self) -> bool {
        self.0.len() == 1 && self.0[0].len() == 9
    }
    fn is_empty(&self) -> bool {
        self.0.len() == 0 && self.card_slot().is_none()
    }
    fn card_slot(&self) -> Option<Card> {
        self.1
    }
    fn movable_span(&self) -> Option<Span> {
        if self.is_dead() {
            return None;
        }
        match self.card_slot() {
            Some(c) => Some(Span::from_card(c)),
            None => self.0.last().cloned(),
        }
    }
    /// For "explain" use-case.
    fn explain_last_n_cards(&self, mut n: u8) -> String {
        let mut cards = Vec::new();
        if let Some(c) = self.1 {
            if n > 0 {
                n -= 1;
                cards.push(format!("{:?}", c));
            }
        }
        let span = self.0.last().unwrap();
        for i in (span.bottom().0..=span.top().0).take(n as usize) {
            cards.push(format!("{:?}", Card(i)));
        }
        cards.reverse();
        format!("[{}]", cards.join(" "))
    }
}

impl State {
    fn normalize(&mut self) {
        self.0.sort_unstable();
        self.validate();
    }

    fn parse(s: &str) -> Self {
        let mut m: [[Card; 6]; 6] = Default::default();
        let mut card_counts: HashMap<Card, usize> = HashMap::new();
        let mut card_total = 0;
        for (i, line) in s.lines().enumerate().take(6) {
            for (j, c) in line
                .chars()
                .filter(|c| !c.is_whitespace() && *c != '1')
                .enumerate()
                .take(6)
            {
                let c = Card::from_str(&c.to_string());
                *card_counts.entry(c).or_default() += 1;
                if card_counts[&c] > 4 {
                    panic!("too many {:?} cards", c);
                }
                card_total += 1;
                m[i][j] = c;
            }
        }
        if card_total != 36 {
            panic!("wrong number of cards {} (expect 36)", card_total);
        }
        Self::from_grid(m)
    }

    fn from_grid(m: [[Card; 6]; 6]) -> State {
        let mut s = State::default();
        for col in 0..6 {
            for row in 0..6 {
                let card = m[row][col];
                let cards = &mut s.0[col].0;
                match cards.pop() {
                    Some(mut span) => {
                        if span.can_accept_card(card) {
                            span.extend_size(1);
                            cards.push(span);
                        } else {
                            cards.push(span);
                            cards.push(Span::from_card(card));
                        }
                    }
                    None => {
                        cards.push(Span::from_card(card));
                    }
                }
            }
        }
        dprintln!("Parsed State: {:?}", &s);
        s
    }

    /// Transfer to other states.
    fn next_states<'a>(&'a self) -> impl Iterator<Item = (State, TransferStep)> + 'a {
        (0..6).flat_map(move |col1| {
            (0..6).flat_map(move |col2| self.next_states_by_moving(col1, col2).into_iter())
        })
    }

    // Generate state moving to another column.
    fn next_states_by_moving(&self, col1: u8, col2: u8) -> Vec<(State, TransferStep)> {
        let c1 = &self.0[col1 as usize];
        let c2 = &self.0[col2 as usize];
        let mut result = Vec::new();
        if col1 == col2 || c1.is_dead() || c2.is_dead() || c2.card_slot().is_some() {
            return result;
        }
        let span1 = or!(c1.movable_span(), return result);
        match c2.movable_span() {
            Some(span2) => {
                let n = span2.accept_span_size(span1);
                if n > 0 {
                    // Do not use free slot.
                    dprintln!(" Move {} cards from {} to {}", n, col1, col2);
                    let mut new_state = *self;
                    let step = new_state.unchecked_apply_moving_to(col1, col2, n, false, 1);
                    result.push((new_state, step));
                }
                if n != 1 && c1.card_slot().is_none() {
                    // Use free slot.
                    dprintln!(" Move 1 card from {} to {} (slot)", col1, col2);
                    let mut new_state = *self;
                    let step = new_state.unchecked_apply_moving_to(col1, col2, 1, true, 2);
                    // dprintln!("  {:?}\n  => {:?}", self, &new_state);
                    result.push((new_state, step));
                }
            }
            None => {
                // Use free stack.
                assert!(c2.is_empty());
                for n in 1..=span1.len() {
                    dprintln!(" Move {} cards from {} to {}", n, col1, col2);
                    let mut new_state = *self;
                    let step = new_state.unchecked_apply_moving_to(col1, col2, n, false, 3);
                    result.push((new_state, step));
                }
            }
        }
        result
    }

    /// Move n cards from col1 to col2 with checking.
    fn unchecked_apply_moving_to(
        &mut self,
        col1: u8,
        col2: u8,
        n: u8,
        slot2: bool,
        step_score: u8,
    ) -> TransferStep {
        let step = TransferStep {
            from_column: self.0[col1 as usize],
            to_column: self.0[col2 as usize],
            card_count: n,
            to_slot: slot2,
            step_score,
        };
        let mut span = match self.0[col1 as usize].1.take() {
            Some(card) => {
                assert_eq!(n, 1);
                Span::from_card(card)
            }
            None => self.0[col1 as usize].0.pop().unwrap(),
        };
        let span_bottom = span.bottom();
        span.shrink_size(n);
        if span.len() > 0 {
            self.0[col1 as usize].0.push(span);
        }
        if slot2 {
            assert_eq!(n, 1);
            self.0[col2 as usize].1 = Some(span_bottom);
        } else {
            let span2 = match self.0[col2 as usize].0.pop() {
                None => Span(span.bottom() - 1, n),
                Some(mut span2) => {
                    span2.extend_size(n);
                    span2
                }
            };
            self.0[col2 as usize].0.push(span2);
        }
        step
    }

    fn is_success(&self) -> bool {
        self.0.iter().all(|c| c.is_dead() || c.is_empty())
    }

    fn validate(&self) {
        let mut cards = [0u8; 9];
        for col in self.0.iter() {
            for span in col.0.as_ref() {
                assert!(span.len() > 0);
                for v in span.bottom().0..=span.top().0 {
                    assert!(v < 9);
                    cards[v as usize] += 1;
                }
            }
            if let Some(Card(v)) = col.1 {
                cards[v as usize] += 1;
            }
        }
        assert_eq!(cards, [4u8; 9], "{:?} does not pass validation", self);
    }

    /// How close (approx) this state is to a solution.
    fn score(&self) -> u8 {
        let mut max_span_len = 0;
        let mut free_cell_len = 0;
        let mut score: u8 = self
            .0
            .iter()
            .map(|c| {
                if !c.is_dead() {
                    if let Some(s) = c.0.last() {
                        if c.0.len() != 1 {
                            max_span_len = max_span_len.max(s.len());
                        }
                    }
                    if c.1.is_none() {
                        free_cell_len += 1;
                    }
                }
                let free_cell_score = if c.1.is_some() { 0 } else { 1 };
                // let free_stack_score = if c.is_empty() { 5 } else { 0 };
                // let dead_score = if c.is_dead() { 0 } else { 5 };
                let tidy_score = 6 - c.0.len();
                tidy_score + free_cell_score //+ free_stack_score + dead_score
            })
            .sum();
        if max_span_len > free_cell_len {
            // Penalty for not able to move.
            let penalty = (max_span_len - free_cell_len) * 10;
            score -= score.min(penalty);
        }
        score
    }

    fn find_column_index(&self, col: &Column, exclude: Option<u8>) -> u8 {
        for i in 0..6 {
            if &self.0[i] == col && Some(i as u8) != exclude {
                return i as u8;
            }
        }
        panic!("cannot find column {:?} in {:?}", col, self);
    }
}

/// Brute force searching all states.
#[derive(Default)]
struct Searcher {
    // Assigned states.
    cache: HashMap<State, usize>,
    states: Vec<State>,
    state_step_count: Vec<u16>,

    // State K comes from State V.0 with step V.1.
    edges: HashMap<usize, (usize, TransferStep)>,

    // Visited states.
    visited: HashSet<usize>,

    // For progress rendering.
    best_score: u8,
    best_state_id: usize,
}

impl Searcher {
    fn to_id(&mut self, state: &State, step_count: u16) -> usize {
        if let Some(id) = self.cache.get(state) {
            return *id;
        }
        let id = self.states.len();
        self.states.push(*state);
        self.state_step_count.push(step_count);
        self.cache.insert(*state, id);
        if (id + 1000_001) % 1000_000 == 0 {
            eprint!(
                "State count: {}M. Best score: {} {:?}          \r",
                1 + (id / 1000_000),
                self.best_score,
                &self.states[self.best_state_id]
            );
        }
        id
    }

    /// Return Some(explain) if a solution is found.
    fn search(&mut self, initial_state: State) -> Option<String> {
        // key: (score, -step_count, step_score)
        let mut to_visit = {
            let mut state = initial_state;
            state.normalize();
            let score = state.score();
            let id = self.to_id(&state, 0);
            let mut heap = BinaryHeap::new();
            heap.push(((score, 0i16, 0u8), id));
            heap
        };
        let mut result = None;
        while let Some(((score1, _neg_step_count, _step_score1), id1)) = to_visit.pop() {
            if !self.visited.insert(id1) {
                continue;
            }
            let step_count1 = self.state_step_count[id1];
            let state1 = &self.states[id1];
            dprintln!("Considering {:?} Score {}", &state1, score1);
            if score1 > self.best_score {
                self.best_score = score1;
                self.best_state_id = id1;
            }
            if state1.is_success() {
                eprintln!("\nFound solution!");
                result = Some(self.explain_solution(initial_state, id1));
                break;
            }
            for (mut next_state, step) in state1.clone().next_states() {
                next_state.normalize();
                let step_count2 = step_count1 + 1;
                let id2 = self.to_id(&next_state, step_count2);
                if !self.visited.contains(&id2) {
                    dprintln!(" Next: {:?}", &next_state);
                    let score2 = next_state.score();
                    to_visit.push(((score2, -(step_count2 as i16), step.step_score), id2));
                    self.edges.insert(id2, (id1, step));
                } else {
                    // Use less steps?
                    let existing_step_count2 = self.state_step_count[id2];
                    if existing_step_count2 > step_count2 {
                        dprintln!(" Visited Next: {:?}", &next_state);
                        dprintln!(
                            "  Optimize step count {} -> {}",
                            existing_step_count2,
                            step_count2,
                        );
                        self.state_step_count[id2] = step_count2;
                        self.edges.insert(id2, (id1, step));
                    }
                }
            }
        }
        eprintln!("State count: {}", self.states.len());
        result
    }

    /// Explain how to get the solution state.
    fn explain_solution(&self, initial_state: State, end_state_id: usize) -> String {
        let mut steps = Vec::new();
        let mut msg = String::new();
        let mut id = end_state_id;
        while let Some((prev_id, step)) = self.edges.get(&id) {
            let state = &self.states[id];
            dprintln!("Score {} {:?}", state.score(), state);
            steps.push(step);
            id = *prev_id;
        }
        steps.reverse();
        // Replay steps to reconstruct the column numbers.
        let mut state = initial_state;
        for (i, step) in steps.into_iter().enumerate() {
            msg += &format!("Step {:>3}. {}\n", i + 1, step.explain(&state));
            step.apply(&mut state);
            if is_verbose() {
                msg += &format!("          Board: {} (Score: {})\n", &state, state.score());
            }
        }
        msg
    }
}

pub(crate) fn process(s: &str) -> Option<String> {
    let state = State::parse(s);
    let mut searcher = Searcher::default();
    searcher.search(state)
}

pub(crate) fn main() {
    use std::io::Read;
    let s = {
        let mut s = String::new();
        let mut input = std::io::stdin();
        input.read_to_string(&mut s).unwrap();
        s
    };
    if let Some(msg) = process(&s) {
        println!("{}", msg);
    }
}
