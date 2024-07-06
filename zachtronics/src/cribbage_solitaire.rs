use std::collections::HashMap;
use std::hash::Hash;
use std::io::Read;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

static DEBUG: AtomicBool = AtomicBool::new(false);
fn is_debug() -> bool {
    DEBUG.load(Ordering::Acquire)
}

macro_rules! dprintln {
    ($($t:tt)*) => {
        if is_debug() {
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

const COLUMNS: usize = 4;
const ROWS: usize = 13;

/// Problem to solve.
#[derive(Debug)]
struct Problem {
    cards: [[u8; COLUMNS]; ROWS],
}

// Count of unused cards in each column.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
struct CardLens([u8; COLUMNS]);

// Transfer from one CardLens state to another by "clicking" at cards
// to move them to the in-game card stack and then "clicking"
// "next state".
struct CardStackSearch<'a> {
    problem: &'a Problem,
    // Stack for DFS.
    stack: Vec<CardStateSearchFrame>,
}

#[derive(Clone, Copy, Debug, Default)]
struct CardPicked {
    // Picked card value (by the parent frame).
    card: u8,
    // The following fields are derived from the card stack.
    //
    // Column the card come from.
    // Can also be caluclated by comparing card_lens with the parent frame.
    column: u8,
    // Sum of the card stack.
    // Can also be calculated by following card and parent cards.
    sum: u8,
    // Score of the card stack.
    // Can also be calculated by following card and parent cards.
    score: u8,
}

// Small Vec optimization for Vec<CardPicked>
#[derive(Clone, Copy, Debug, Default)]
struct CardStackPicked {
    // A card stack contains at most 13 cards (1*4+2*4+3*2>31)
    cards: [u8; 13],
    columns: [u8; 13],
    score: u8,
    len: u8,
}

#[derive(Clone, Copy, Debug)]
struct CardStateSearchFrame {
    // Remaining cards.
    card_lens: CardLens,
    // Columns that are already considered.
    considered_column_len: u8,

    // Card picked by the parent search frame and derived score, etc.
    // Note this is pointless for stack[0].
    picked: CardPicked,
}

type Score = u8;

impl<'a> Iterator for CardStackSearch<'a> {
    // Cards picked in the stack, and remaining cards.
    type Item = (CardStackPicked, CardLens);

    // Return a full stack of card picks, and their score.
    fn next(&mut self) -> Option<Self::Item> {
        let frame = or!(self.stack.last_mut(), return None);

        for column in (frame.considered_column_len)..(COLUMNS as u8) {
            let card = or!(self.problem.card_at(frame.card_lens, column), continue);
            let card_sum = card.min(10);
            let next_sum = frame.picked.sum + card_sum;
            if next_sum > 31 {
                continue;
            }

            // Pick the card and continue search by calling next() recursively.
            frame.considered_column_len = column + 1;
            let mut next_card_lens = frame.card_lens;
            let score = frame.picked.score;
            if DEBUG.load(Ordering::Acquire) {
                let stack_str = self.card_stack_str();
                eprintln!("  Considering pushing {card} to [{stack_str}]");
            }
            next_card_lens.0[column as usize] -= 1;
            let frame = CardStateSearchFrame {
                card_lens: next_card_lens,
                considered_column_len: 0,
                picked: CardPicked {
                    card,
                    column,
                    sum: next_sum,
                    score: score, // Not correct. Will be fixed later.
                },
            };
            // Fixup score.
            self.stack.push(frame);
            let inc_score = self.calculate_incremental_score();
            self.stack.last_mut().unwrap().picked.score += inc_score;
            return self.next();
        }

        // No cards can be picked. End this stack.
        if frame.considered_column_len == 0 {
            frame.considered_column_len = COLUMNS as u8;
            let card_lens = frame.card_lens;
            let picked = CardStackPicked::from_search_stack(&self.stack[1..]);
            Some((picked, card_lens))
        } else {
            self.stack.pop();
            self.next()
        }
    }
}

impl CardStackPicked {
    fn from_search_stack(v: &[CardStateSearchFrame]) -> Self {
        let mut cards = [0u8; 13];
        let mut columns = [0u8; 13];
        for (i, f) in v.iter().enumerate() {
            cards[i] = f.picked.card;
            columns[i] = f.picked.column;
        }
        let len = v.len();
        let score = v.last().map(|p| p.picked.score).unwrap_or_default();
        Self {
            cards,
            columns,
            len: len as u8,
            score,
        }
    }

    fn card_stack_str(&self) -> String {
        card_stack_str(self.cards.into_iter().take(self.len as usize))
    }

    fn column_str(&self) -> String {
        self.columns
            .into_iter()
            .take(self.len as usize)
            .map(|v| (v + 1).to_string())
            .collect::<Vec<String>>()
            .join(" ")
    }
}

impl<'a> CardStackSearch<'a> {
    fn from_problem_card_lens(problem: &'a Problem, card_lens: CardLens) -> Self {
        Self {
            problem,
            stack: vec![CardStateSearchFrame {
                card_lens,
                considered_column_len: 0,
                picked: Default::default(),
            }],
        }
    }

    fn card_stack_str(&self) -> String {
        card_stack_str(self.stack.iter().skip(1).map(|f| f.picked.card))
    }

    // Calcualte the new score caused by the last stack frame.
    fn calculate_incremental_score(&self) -> u8 {
        let stack = &self.stack[1..];
        calculate_incremental_score_for_stack(stack)
    }
}

fn card_stack_str(stack: impl IntoIterator<Item = u8>) -> String {
    stack
        .into_iter()
        .map(card_int_to_str)
        .collect::<Vec<_>>()
        .join(" ")
}

fn calculate_incremental_score_for_stack(stack: &[CardStateSearchFrame]) -> u8 {
    // stack[0] does not contain card information.
    let len = stack.len();
    let picked = or!(stack.last().map(|f| f.picked), return 0);
    // Calculate score.
    let mut score = 0u8;
    // +2: First card is a Jack.
    if stack.len() == 1 && picked.card == 11 {
        dprintln!("  +2 (first Jack)");
        score += 2;
    }
    // +2: Exactly 15 or 31.
    if picked.sum == 15 || picked.sum == 31 {
        dprintln!("  +2 (exactly 15 or 31)");
        score += 2;
    }
    // +2, +6, +12: Set of same card. Overlaps are double counted.
    let same_card_count = (1..len)
        .rev()
        .take_while(|&i| stack[i as usize].picked.card == stack[(i as usize) - 1].picked.card)
        .count();
    if same_card_count > 0 {
        let same_card_score = match same_card_count {
            1 => 2,
            2 => 6,
            _ => 12,
        };
        score += same_card_score;
        dprintln!("  +{same_card_score} (same card #{same_card_count})");
    }
    // +3, +4, ..., +7: Run of 3 to 7 cards in any order.
    // Overlaps are double counted.
    for run_len in (3..=(7.min(len))).rev() {
        let end = len;
        let start = end - run_len;
        if is_run_of_many_cards(
            stack[start as usize..end as usize]
                .iter()
                .map(|f| f.picked.card),
        ) {
            let inc_score = run_len as u8;
            // Example in game: 5 4 2 A 3 5 4 2 A 3 is considered 5-run multiple times
            // and gets +5 x 6 in the game.
            // Example in game: 6 4 5 A 2 3 6 4, "6 4 5" and "6 4 5 A 2 3" are double
            // counted as +3 and +6.
            // Example in game: 4 6 5 4 3 9, "6 5 4" and "6 5 4 3" are double
            // counted as +3 and + 4.
            dprintln!("  +{inc_score} (run of #{run_len} cards)");
            score += inc_score;
            break;
        }
    }
    score
}

#[derive(Clone, Debug)]
struct Solver<'a> {
    problem: &'a Problem,

    // state -> (best score, card stack, previous state).
    cache: HashMap<CardLens, (Score, CardStackPicked, CardLens)>,
    cache_hit_count: usize,

    // Config
    //
    // Skip search if stack score + N is lower then the best.
    skip_if_stack_score_lower_than_best: u8,
    // Skip after considering N choices.
    skip_after_choices: usize,
    // Quality setting.
    quality: u8,
}

impl Problem {
    fn parse(s: &str) -> Option<Self> {
        let mut cards = [[0u8; COLUMNS]; ROWS];
        let mut card_counts = [0; 14];
        for (i, line) in s.lines().take(ROWS).enumerate() {
            let mut line_cards = [0u8; COLUMNS];
            let mut pending_1 = false;
            let mut j = 0;
            for ch in line.chars() {
                let v = match (pending_1, ch) {
                    (false, '1') => {
                        pending_1 = true;
                        continue;
                    }
                    (true, _) => {
                        pending_1 = false;
                        let s: String = format!("1{}", ch);
                        str_to_card_int(s.trim())
                    }
                    (false, ' ') => {
                        continue;
                    }
                    (false, _) => {
                        let s: String = ch.to_string();
                        str_to_card_int(&s)
                    }
                }?;
                card_counts[v as usize] += 1;
                if card_counts[v as usize] > 4 {
                    // Duplicated cards.
                    dprintln!("Duplicated card: {s}");
                    return None;
                }
                line_cards[j] = v;
                j += 1;
                if j >= 4 {
                    break;
                }
            }
            cards[i] = line_cards;
        }
        if card_counts.iter().fold(0, |a, v| a + v) != 52 {
            dprintln!("Missing cards: {card_counts:?}");
            return None;
        }
        // cards.reverse();
        Some(Self { cards })
    }

    fn initial_state(&self) -> Solver {
        let mut solver = Solver {
            problem: self,
            cache: HashMap::with_capacity(30559),
            cache_hit_count: 0,
            skip_if_stack_score_lower_than_best: 0,
            skip_after_choices: 0,
            quality: 0,
        };
        solver.set_quality(if cfg!(debug_assertions) { 3 } else { 10 });
        solver
    }

    // Card at the given column.
    fn card_at(&self, card_lens: CardLens, column: u8) -> Option<u8> {
        let i = card_lens.0[column as usize];
        if i == 0 {
            None
        } else {
            Some(self.cards[(i - 1) as usize][column as usize])
        }
    }
}

impl CardLens {
    fn initial_search_state() -> Self {
        let mut state = Self::default();
        state.0 = [ROWS as u8; COLUMNS];
        state
    }

    // End state: no more cards.
    fn is_end_state(&self) -> bool {
        self.0.iter().all(|&v| v == 0)
    }
}

#[test]
fn test_stack_scores() {
    fn t(cards: &str) -> String {
        let mut out = Vec::new();
        let cards: Vec<u8> = cards
            .split_whitespace()
            .map(|s| str_to_card_int(s).unwrap())
            .collect();
        let mut stack = Vec::new();
        let mut sum = 0;
        for card in cards {
            sum += card.min(10);
            stack.push(CardStateSearchFrame {
                card_lens: Default::default(),
                considered_column_len: 0,
                picked: CardPicked {
                    card,
                    column: 0,
                    sum,
                    score: 0,
                },
            });
            let inc_score = calculate_incremental_score_for_stack(&stack);
            if inc_score > 0 {
                out.push(format!("+{}", inc_score))
            } else {
                out.push("-".to_string())
            }
        }
        out.join(" ")
    }

    assert_eq!(t("5 4 2 A 3 5 4 2 A 3"), "- - - - +7 +5 +5 +5 +5 +5");
    assert_eq!(t("K K K A"), "- +2 +6 +2");
    assert_eq!(t("6 6 3"), "- +2 +2");
    assert_eq!(t("J 2 2 1"), "+2 - +2 +2");
    assert_eq!(t("A A A A 9 2"), "- +2 +6 +12 - +2");
    assert_eq!(t("2 3 4 5 6 7"), "- - +3 +4 +5 +6");
    assert_eq!(t("A 2 3 4 5 6 7"), "- - +3 +4 +7 +6 +7");

    // "6 4 5" and "6 4 5 A 2 3" are double counted as 3-run (+3) and 6-run (+6).
    assert_eq!(t("6 4 5 A 2 3 6 4"), "- - +5 - - +6 +6 +8");

    // "6 5 4" and "6 5 4 3" are double counted as 3-run and 4-run.
    assert_eq!(t("4 6 5 4 3 9"), "- - +5 +3 +4 +2");
}

fn is_run_of_many_cards(cards: impl IntoIterator<Item = u8>) -> bool {
    let mut bits = 0u32;
    for c in cards {
        let b = 1u32 << c;
        if (bits & b) != 0 {
            return false;
        }
        bits |= b;
    }
    let min_bit = 1u32 << bits.trailing_zeros();
    let max_bit = 1u32 << (u32::BITS - 1 - bits.leading_zeros());
    max_bit * 2 - min_bit == bits
}

#[test]
fn test_is_run_of_many_cards() {
    assert!(is_run_of_many_cards([7, 8]));
    assert!(is_run_of_many_cards([4, 3]));
    assert!(is_run_of_many_cards([5, 6, 7]));
    assert!(is_run_of_many_cards([7, 6, 5]));
    assert!(is_run_of_many_cards([5, 7, 6]));
    assert!(is_run_of_many_cards([4, 9, 6, 7, 8, 5]));
    assert!(!is_run_of_many_cards([5, 5, 6]));
    assert!(!is_run_of_many_cards([5, 8, 6]));
}

impl<'a> Solver<'a> {
    fn set_quality(&mut self, quality: u8) {
        let quality = quality.min(10);
        let (n, c) = match quality {
            0 => (0, 0),
            1 => (1, 3),
            2 => (1, 5),
            3 => (1, 10),
            4 => (2, 10),
            5 => (2, 20),
            6 => (3, 30),
            7 => (4, 40),
            8 => (6, 100),
            9 => (10, 1000),
            _ => (128, usize::MAX),
        };
        self.skip_if_stack_score_lower_than_best = n;
        self.skip_after_choices = c;
        self.quality = quality;
    }

    fn solve(&mut self) {
        let mut state = CardLens::initial_search_state();
        let total_score = self.best_score(&state);
        println!(
            "{}Score: {total_score}",
            if self.quality == 10 { "Best " } else { "" }
        );
        // Trace back to figure out each step.
        let mut step_count = 0;
        loop {
            let (_score, stack, next_state) = or!(self.cache.get(&state), break);
            if stack.len == 0 {
                break;
            }
            let next_score = self.cache.get(&next_state).map(|v| v.0).unwrap_or_default();
            step_count += 1;
            let stack_str = stack.card_stack_str();
            let column_str = stack.column_str();
            println!(
                "{:>2}.{:>4} Take    [{}]\n        Columns [{}]",
                step_count,
                total_score - next_score,
                stack_str,
                column_str,
            );
            state = *next_state;
        }
        // Stats
        eprintln!(
            "Searched states: {}. Cache hit: {}.",
            self.cache.len(),
            self.cache_hit_count,
        );
        if self.quality < 10 {
            eprintln!(
                "Search Quality ($Q): {}. May miss better solutions.",
                self.quality
            );
        }
    }

    fn best_score(&mut self, card_lens: &CardLens) -> Score {
        if card_lens.is_end_state() {
            return 0;
        }
        if let Some(v) = self.cache.get(card_lens) {
            self.cache_hit_count += 1;
            return v.0;
        }
        dprintln!("Calculating best score for {card_lens:?}");
        let mut best = (0, CardStackPicked::default(), CardLens::default());
        let stack_search = CardStackSearch::from_problem_card_lens(self.problem, *card_lens);
        let mut choices: Vec<_> = stack_search.collect();
        if !choices.is_empty() {
            let best_stack_score = if self.quality < 10 {
                choices.sort_unstable_by_key(|c| 255u8 - c.0.score);
                choices[0].0.score
            } else {
                0
            };
            for (i, (stack, next_card_lens)) in choices.into_iter().enumerate() {
                debug_assert_ne!(card_lens, &next_card_lens);
                // Consider skips.
                if best.0 > 0 {
                    if i > self.skip_after_choices {
                        break;
                    }
                    if stack.score + self.skip_if_stack_score_lower_than_best < best_stack_score {
                        break;
                    }
                }
                let next_score = self.best_score(&next_card_lens);
                let score = next_score + stack.score;
                if score > best.0 || best.0 == 0 {
                    best = (score, stack, next_card_lens);
                    dprintln!("  Update best to {score} ({stack:?} {next_card_lens:?})");
                }
            }
        }
        self.cache.insert(*card_lens, best);
        best.0
    }
}

fn str_to_card_int(s: &str) -> Option<u8> {
    let v = match s {
        "1" | "A" | "a" => 1,
        "2" => 2,
        "3" => 3,
        "4" => 4,
        "5" => 5,
        "6" => 6,
        "7" => 7,
        "8" => 8,
        "9" => 9,
        "10" => 10,
        "J" | "j" => 11,
        "Q" | "q" => 12,
        "K" | "k" => 13,
        _ => return None,
    };
    Some(v)
}

fn card_int_to_str(v: u8) -> &'static str {
    [
        "_", "A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K",
    ][v as usize]
}

pub(crate) fn main() {
    DEBUG.store(std::env::var_os("D").is_some(), Ordering::Release);
    let s = {
        let mut s = String::new();
        let mut input = std::io::stdin();
        input.read_to_string(&mut s).unwrap();
        s
    };
    let p = Problem::parse(&s).unwrap();
    let mut s = p.initial_state();
    if let Some(quality) = std::env::var("Q").ok().and_then(|v| v.parse::<u8>().ok()) {
        s.set_quality(quality);
    }
    s.solve();
}
