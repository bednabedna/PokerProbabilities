mod poker;
use argh::FromArgs;
use poker::cardset::{CardParseError, CardSet};
use rayon::prelude::*;
use std::{str::FromStr, time::Instant};

/// Max cards allowed in player hand
const MAX_HAND: u32 = 2;
/// Max cards allowed in table
const MAX_TABLE: u32 = 5;

fn simulate(hand: CardSet, table: CardSet, players: u32, games: u32) -> u32 {
    assert!(players >= 2 && players <= 8);
    let hc = hand.count_cards();
    assert!(hc <= MAX_HAND);
    assert!((hand & table).is_empty());
    let hand_draw_count = MAX_HAND - hc;
    let tc = table.count_cards();
    assert!(tc <= MAX_TABLE);
    let deck = !(hand | table);
    let table_draw_count = MAX_TABLE - tc;
    let opponents = players - 1;
    (0..games)
        .into_par_iter()
        .map(|_| {
            let mut deck = deck;
            let table = table | deck.draw(table_draw_count);
            let my_comb = (hand | deck.draw(hand_draw_count) | table).comb();
            for _ in 0..opponents {
                let player_comb = (deck.draw(MAX_HAND) | table).comb();
                if player_comb > my_comb {
                    return 0;
                }
            }
            1
        })
        .sum()
}

fn print_simulation(hand: CardSet, table: CardSet, players: u32, games: u32) {
    assert!(players >= 2 && players <= 8);
    let hc = hand.count_cards();
    assert!(hc <= MAX_HAND);
    let hand_draw_count = MAX_HAND - hc;
    let table_cards_count = table.count_cards();
    assert!(table_cards_count <= MAX_TABLE);
    assert!((hand & table).is_empty());
    let table_draw_count = MAX_TABLE - table_cards_count;
    let deck = !(hand | table);
    let mut results = Vec::with_capacity(players as usize);
    let mut rows = Vec::with_capacity(players as usize);

    for round in 0..games {
        results.clear();
        rows.clear();
        let mut deck = deck;
        let hand = hand | deck.draw(hand_draw_count);
        let table = table | deck.draw(table_draw_count);

        results.push((hand, (hand | table).comb()));
        for _ in 0..players - 1 {
            let player_cards = deck.draw(MAX_HAND);
            let player_comb = (player_cards | table).comb();
            results.push((player_cards, player_comb));
        }
        let winning_combination = results.iter().map(|v| v.1).max().unwrap();
        let you_won = winning_combination == results[0].1;

        for &(cards, comb) in results.iter() {
            let won = winning_combination == comb;
            rows.push((
                format!("    {:?}", cards),
                format!("{}", comb.name()),
                if won { "[W]" } else { "" },
            ));
        }
        println!("{} ({:?})\n", if you_won { "WON" } else { "LOST" }, table);
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
        if round < games - 1 {
            println!("\n--------------------------\n");
        }
    }
    println!("\n");
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
    WrongNumberOfPlayers(u32),
    InvalidHandTableComposition(CardSet),
}

fn execute() -> Result<(), SimulationError> {
    let args: SimulationArgs = argh::from_env();

    let hand = CardSet::from_str(&args.hand).map_err(|err| SimulationError::HandParseError(err))?;
    let table =
        CardSet::from_str(&args.table).map_err(|err| SimulationError::TableParseError(err))?;

    if args.players < 2 || args.players > 8 {
        Err(SimulationError::WrongNumberOfPlayers(args.players))
    } else if hand.count_cards() > MAX_HAND {
        Err(SimulationError::InvalidHand(hand))
    } else if table.count_cards() > MAX_TABLE {
        Err(SimulationError::InvalidTable(table))
    } else if !(table & hand).is_empty() {
        Err(SimulationError::InvalidHandTableComposition(table & hand))
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
                "({:?}) ({:?}) = {}",
                hand,
                table,
                (hand | table).comb().name()
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
                "Error invalid hand: hand has {} cards, maximum is {}",
                hand.count_cards(),
                MAX_HAND
            ),
            SimulationError::InvalidTable(table) => println!(
                "Error invalid table: table has {} cards, maximum is {}",
                table.count_cards(),
                MAX_TABLE
            ),
            SimulationError::InvalidHandTableComposition(composition) => println!(
                "Error: table and hand are sharing the following cards: {:?}",
                composition
            ),
            SimulationError::WrongNumberOfPlayers(players) => {
                println!("Error: required 2-8 players, found {}", players)
            }
        }
    }
}
