use argh::FromArgs;
use rand::distributions::{Distribution, Uniform};
use rayon::prelude::*;
use std::{cmp::Ordering, fmt::Debug, time::Instant};
use std::{
    iter::Peekable,
    str::{Chars, FromStr},
};

#[cfg(test)]
mod tests;

#[derive(PartialEq, Eq, Clone, Copy)]
struct CardSet(usize);

impl CardSet {
    #[allow(dead_code)]
    fn none() -> Self {
        Self(0)
    }
    #[allow(dead_code)]
    fn all() -> Self {
        Self(0b1111111111111111111111111111111111111111111111111111)
    }

    fn not(&self) -> CardSet {
        CardSet((!self.0) & 0b1111111111111111111111111111111111111111111111111111)
    }

    fn add(&self, other: &CardSet) -> CardSet {
        CardSet(self.0 | other.0)
    }

    fn and(&self, other: &CardSet) -> CardSet {
        CardSet(self.0 & other.0)
    }

    fn draw(&mut self, mut count: u32) -> CardSet {
        let mut cards = self.0;
        let mut rng = rand::thread_rng();
        let die = Uniform::from(0..52);
        while count > 0 && cards != 0 {
            let card_index = die.sample(&mut rng);
            let after = cards & !(1 << card_index);
            if after != cards {
                cards = after;
                count -= 1;
            }
        }
        let drawn = self.0 & !(cards);
        self.0 = cards;
        CardSet(drawn)
    }

    fn comb(&self) -> Combination {
        Combination::new(*self)
    }

    fn count_cards(&self) -> u32 {
        self.0.count_ones()
    }

    fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

#[derive(Eq, PartialEq, Clone, Debug, Copy)]
enum CombType {
    StraightFlush = 1 << 31,
    Poker = 1 << 30,
    FullHouse = 1 << 29,
    Flush = 1 << 28,
    Straight = 1 << 27,
    Tris = 1 << 26,
    TwoPairs = 1 << 17,
    Pair = 1 << 14,
    HighCard = 0,
}

#[derive(Eq, PartialEq, Clone, Copy)]
struct Combination {
    comb_type: CombType,
    comb_value: u32,
}

impl Ord for Combination {
    fn cmp(&self, other: &Self) -> Ordering {
        let v1 = self.comb_type as u32 | self.comb_value;
        let v2 = other.comb_type as u32 | other.comb_value;
        v1.cmp(&v2)
    }
}

impl PartialOrd for Combination {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Combination {
    fn bit_position(bits: u32) -> u32 {
        (0..12)
            .find(|&n| (bits.wrapping_shr(n) & 1) != 0)
            .unwrap_or(12)
    }
    fn new(cards: CardSet) -> Self {
        let cards = cards.0;
        let all_numbers = 0b1111111111111;
        let n1 = (cards & all_numbers) as u32;
        let n2 = (cards.wrapping_shr(13) & all_numbers) as u32;
        let n3 = (cards.wrapping_shr(2 * 13) & all_numbers) as u32;
        let n4 = (cards.wrapping_shr(3 * 13) & all_numbers) as u32;
        let numbers = n1 | n2 | n3 | n4;
        let is_straight = numbers == 0b11111 ||        // 2-6
                          numbers == 0b111110 ||       // 3-7
                          numbers == 0b1111100 ||      // 4-8
                          numbers == 0b11111000 ||     // 5-9
                          numbers == 0b111110000 ||    // 6-10
                          numbers == 0b1111100000 ||   // 7-J
                          numbers == 0b11111000000 ||  // 8-Q
                          numbers == 0b111110000000 || // 9-K
                          numbers == 0b1111100000000 ||// 10-A
                          numbers == 0b1000000001111; // A-5
        let flush_value = (cards & 0b11111111111111111111111111)
            .max(cards & !0b11111111111111111111111111)
            .count_ones();
        let is_flush = flush_value > 4;
        if is_straight && is_flush {
            Combination {
                comb_type: CombType::StraightFlush,
                comb_value: numbers,
            }
        } else {
            let poker_value = n1 & n2 & n3 & n4;
            let is_poker = poker_value > 0;
            if is_poker {
                Combination {
                    comb_type: CombType::Poker,
                    comb_value: (poker_value << 13) | numbers,
                }
            } else {
                let tris_value = (n1 & n2 & n3) | (n1 & n2 & n4) | (n1 & n3 & n4) | (n2 & n3 & n4);
                let pair_value =
                    ((n1 & n2) | (n1 & n3) | (n1 & n4) | (n2 & n3) | (n2 & n4) | (n3 & n4))
                        & !tris_value;
                let pairs_count = pair_value.count_ones();
                let is_tris = tris_value > 0;
                let is_pair = pair_value > 0;
                if is_tris && is_pair {
                    Combination {
                        comb_type: CombType::FullHouse,
                        comb_value: (tris_value << 13) | pair_value,
                    }
                } else if is_flush {
                    Combination {
                        comb_type: CombType::Flush,
                        comb_value: (pairs_count << 26) | (pair_value << 13) | numbers,
                    }
                } else if is_straight {
                    Combination {
                        comb_type: CombType::Straight,
                        comb_value: numbers,
                    }
                } else if is_tris {
                    Combination {
                        comb_type: CombType::Tris,
                        comb_value: ((Combination::bit_position(tris_value) + 1) << 13) | numbers,
                    }
                } else if pairs_count > 1 {
                    Combination {
                        comb_type: CombType::TwoPairs,
                        comb_value: (pair_value << 5)
                            | Combination::bit_position(numbers & !pair_value),
                    }
                } else if pair_value != 0 {
                    Combination {
                        comb_type: CombType::Pair,
                        comb_value: ((Combination::bit_position(pair_value) + 1) << 13) | numbers,
                    }
                } else {
                    Combination {
                        comb_type: CombType::HighCard,
                        comb_value: numbers,
                    }
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum CardParseError {
    InvalidDigit(char),
    InvalidSuit(char),
    RepeatedCard(CardSet),
    UnexpectedEndOfInput,
}

fn parse_one_card(chars: &mut Peekable<Chars>) -> Result<Option<CardSet>, CardParseError> {
    let maybe_num_char_1 = chars.next().map(|c| c.to_ascii_uppercase());
    if maybe_num_char_1.is_none() {
        return Ok(None);
    }
    let num_char_1 = maybe_num_char_1.unwrap();

    let number = if num_char_1 == '1' {
        let num_char_2 = chars
            .peek()
            .ok_or(CardParseError::UnexpectedEndOfInput)?
            .to_ascii_uppercase();
        Ok(if num_char_2 >= '0' && num_char_2 <= '3' {
            chars.next();
            (num_char_2 as isize - '0' as isize) as usize + 10
        } else {
            14
        })
    } else if num_char_1 >= '2' && num_char_1 <= '9' {
        Ok((num_char_1 as isize - '0' as isize) as usize)
    } else if num_char_1 == 'J' {
        Ok(11)
    } else if num_char_1 == 'Q' {
        Ok(12)
    } else if num_char_1 == 'K' {
        Ok(13)
    } else if num_char_1 == 'A' {
        Ok(14)
    } else {
        Err(CardParseError::InvalidDigit(num_char_1))
    }?;

    let suit_char = chars
        .next()
        .ok_or(CardParseError::UnexpectedEndOfInput)?
        .to_ascii_uppercase();

    let suit = match suit_char {
        '♦' | 'Q' => Ok(0 * 13),
        '♥' | 'C' => Ok(1 * 13),
        '♠' | 'P' => Ok(2 * 13),
        '♣' | 'F' => Ok(3 * 13),
        _ => Err(CardParseError::InvalidSuit(suit_char)),
    }?;

    Ok(Some(CardSet(1 << (number - 2 + suit))))
}

impl FromStr for CardSet {
    type Err = CardParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars().peekable();
        let mut cards = CardSet::none();
        while let Some(card) = parse_one_card(&mut chars)? {
            if cards.and(&card).is_empty() {
                cards = cards.add(&card);
            } else {
                return Err(CardParseError::RepeatedCard(card));
            }
        }
        Ok(cards)
    }
}

impl Debug for CardSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut is_first = true;
        for i in 0..52 {
            if (self.0.wrapping_shr(i) & 1) == 1 {
                if is_first {
                    is_first = false;
                } else {
                    write!(f, ",")?;
                }
                let number = i % 13;
                let suit = i / 13;
                if number == 8 {
                    write!(f, "10")?;
                } else {
                    write!(
                        f,
                        "{}",
                        match number {
                            12 => 'A',
                            11 => 'K',
                            10 => 'Q',
                            9 => 'J',
                            _ => (('0' as u8 + 2) + number as u8) as char,
                        },
                    )?;
                }
                write!(
                    f,
                    "{}",
                    match suit {
                        0 => '♦',
                        1 => '♥',
                        2 => '♠',
                        _ => '♣',
                    }
                )?;
            }
        }
        Ok(())
    }
}

fn simulate(hand: CardSet, table: CardSet, players: u32, games: u32) -> u32 {
    let players = players - 1;
    assert_eq!(hand.count_cards(), 2);
    let table_cards_count = table.count_cards();
    assert!(table_cards_count <= 3);
    let my_cards = hand.add(&table);
    assert!(my_cards.count_cards() <= 5);
    let deck = my_cards.not();
    let table_draw_count = 3 - table_cards_count;
    (0..games)
        .into_par_iter()
        .map(|_| {
            let mut deck = deck;
            let table = table.add(&deck.draw(table_draw_count));
            let my_comb = my_cards.add(&table).comb();
            for _ in 0..players {
                let player_comb = deck.draw(2).add(&table).comb();
                if player_comb > my_comb {
                    return 0;
                }
            }
            1
        })
        .sum()
}

fn print_simulation(hand: CardSet, table: CardSet, players: u32, games: u32) {
    assert!(players > 1);
    assert_eq!(hand.count_cards(), 2);
    let table_cards_count = table.count_cards();
    assert!(table_cards_count <= 3);
    let my_cards = hand.add(&table);
    assert!(my_cards.count_cards() <= 5);
    let deck = my_cards.not();
    let table_draw_count = 3 - table_cards_count;
    let mut results = Vec::with_capacity(players as usize);
    let mut rows = Vec::with_capacity(players as usize);

    for _ in 0..games {
        let mut deck = deck;
        let table = table.add(&deck.draw(table_draw_count));
        results.clear();
        results.push({
            let my_cards = my_cards.add(&table);
            (my_cards, my_cards.comb())
        });
        for _ in 0..players - 1 {
            let player_cards = deck.draw(2).add(&table);
            let player_comb = player_cards.comb();
            results.push((player_cards, player_comb));
        }
        let winning_combination = results.iter().map(|v| v.1).max().unwrap();
        let you_won = winning_combination == results[0].1;

        rows.clear();
        rows.push((
            format!("    ({:?}) ({:?})", hand, table),
            format!("{:?}", results[0].1.comb_type),
            if you_won { "[W]" } else { "" },
        ));
        for &(cards, comb) in results.iter().skip(1) {
            let won = winning_combination == comb;
            rows.push((
                format!("    {:?}", cards),
                format!("{:?}", comb.comb_type),
                if won { "[W]" } else { "" },
            ));
        }
        println!("ROUND {}\n", if you_won { "WON" } else { "LOST" });
        let padding_1 = rows.iter().map(|row| row.0.chars().count()).max().unwrap();
        let padding_2 = rows.iter().map(|row| row.1.chars().count()).max().unwrap();
        for row in &rows {
            println!(
                "{:<w1$}   {:<w2$}   {}",
                row.0,
                row.1,
                row.2,
                w1 = padding_1,
                w2 = padding_2
            );
        }

        println!();
    }
    println!();
}

#[derive(FromArgs)]
/// Estimate Poker Texas Holdem winning probabilities simulating games with the cards provided.
/// Cards are given as a string, for example "4CAQ" means 2 cards: 4 of ♥ and ace of ♦.
/// The values are '1' or 'A', '2' to '10', 'J' or '11', 'Q' or '12', 'K' or '13'.
/// Suits are 'C' or '♥', 'Q' or '♦', 'P' or '♠' and 'F' or '♣'.
/// All values and suits can be also lowercase.
struct SimulationArgs {
    /// cards in hand, maximum 2, defaults to no cards
    #[argh(option, default = "String::new()", short = 'h')]
    hand: String,

    /// cards on the table, maximum 3, defaults to no cards
    #[argh(option, default = "String::new()", short = 't')]
    table: String,

    /// number of players in game, defaults to 4
    #[argh(option, default = "4", short = 'p')]
    players: u32,

    /// number of rounds to simulate, defaults to 1 million
    #[argh(option, default = "1000000", short = 'g')]
    games: u32,

    /// print provided number of simulated rounds, optional
    #[argh(option, default = "0", short = 's')]
    show: u32,

    #[argh(switch)]
    /// display execution time
    time: bool,
}

enum SimulationError {
    HandParseError(CardParseError),
    TableParseError(CardParseError),
    InvalidHand(CardSet),
    InvalidTable(CardSet),
    NotEnoughPlayers,
    InvalidHandTableComposition(CardSet),
}

fn execute() -> Result<(), SimulationError> {
    let args: SimulationArgs = argh::from_env();

    let hand = CardSet::from_str(&args.hand).map_err(|err| SimulationError::HandParseError(err))?;
    let table =
        CardSet::from_str(&args.table).map_err(|err| SimulationError::TableParseError(err))?;

    if args.players < 2 {
        Err(SimulationError::NotEnoughPlayers)
    } else if hand.count_cards() > 2 {
        Err(SimulationError::InvalidHand(hand))
    } else if table.count_cards() > 3 {
        Err(SimulationError::InvalidTable(table))
    } else if !table.and(&hand).is_empty() {
        Err(SimulationError::InvalidHandTableComposition(
            table.and(&hand),
        ))
    } else {
        let maybe_timing = if args.time {
            Some(Instant::now())
        } else {
            None
        };

        if args.show > 0 {
            print_simulation(hand, table, args.players, args.show);
        }

        let wins = if hand.is_empty() {
            // without a hand, winning probabilities are equal
            args.games / args.players
        } else {
            simulate(hand, table, args.players, args.games)
        };

        let maybe_execution_time = maybe_timing.map(|time| time.elapsed());

        if hand.is_empty() {
            println!("No hand, equal winning probability among players.");
        } else {
            println!(
                "({:?}) ({:?}) {:?}",
                hand,
                table,
                hand.add(&table).comb().comb_type
            );
        }

        println!(
            "{}/{} = {:.2}%",
            wins,
            args.games,
            wins as f64 / args.games as f64 * 100.0
        );

        if let Some(execution_time) = maybe_execution_time {
            println!("\nsimulated in {:?}", execution_time);
        }
        Ok(())
    }
}

fn print_card_parse_error(error: CardParseError, msg: &str) {
    print!("Error parsing cards: ");
    match error {
        CardParseError::InvalidDigit(d) => println!("found invalid digit '{}' in {}", d, msg),
        CardParseError::InvalidSuit(s) => println!("found invalid suit '{}' in {}", s, msg),
        CardParseError::RepeatedCard(card) => println!("{} has more than one {:?}", msg, card),
        CardParseError::UnexpectedEndOfInput => println!("incomplete input for cards of {}", msg),
    }
}

fn main() {
    if let Err(error) = execute() {
        match error {
            SimulationError::HandParseError(e) => print_card_parse_error(e, "hand"),
            SimulationError::TableParseError(e) => print_card_parse_error(e, "table"),
            SimulationError::InvalidHand(hand) => println!(
                "Error invalid hand: hand has {} cards, maximum 2 allowed",
                hand.count_cards()
            ),
            SimulationError::InvalidTable(table) => println!(
                "Error invalid table: table has {} cards, maximum 3 allowed",
                table.count_cards()
            ),
            SimulationError::InvalidHandTableComposition(composition) => println!(
                "Error: table and hand are sharing the following cards: {:?}",
                composition
            ),
            SimulationError::NotEnoughPlayers => {
                println!("Error: not enough players, at least 2 required")
            }
        }
    }
}
