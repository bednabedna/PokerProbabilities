use crate::{CardSet, CombType, Combination};
use std::str::FromStr;

fn check_parse(input: &str, card_index: usize) {
    assert_eq!(CardSet::from_str(input), Ok(CardSet(1 << card_index)))
}

#[test]
fn none_all() {
    assert_eq!(CardSet::none().not(), CardSet::all());
}

#[test]
fn parse() {
    check_parse("2♦", 0);
    check_parse("3♦", 1);
    check_parse("4♦", 2);
    check_parse("5♦", 3);
    check_parse("6♦", 4);
    check_parse("7♦", 5);
    check_parse("8♦", 6);
    check_parse("9♦", 7);
    check_parse("10♦", 8);
    check_parse("J♦", 9);
    check_parse("Q♦", 10);
    check_parse("K♦", 11);
    check_parse("A♦", 12);

    check_parse("2♥", 13);
    check_parse("3♥", 14);
    check_parse("4♥", 15);
    check_parse("5♥", 16);
    check_parse("6♥", 17);
    check_parse("7♥", 18);
    check_parse("8♥", 19);
    check_parse("9♥", 20);
    check_parse("10♥", 21);
    check_parse("J♥", 22);
    check_parse("Q♥", 23);
    check_parse("K♥", 24);
    check_parse("A♥", 25);

    check_parse("2♠", 26);
    check_parse("3♠", 27);
    check_parse("4♠", 28);
    check_parse("5♠", 29);
    check_parse("6♠", 30);
    check_parse("7♠", 31);
    check_parse("8♠", 32);
    check_parse("9♠", 33);
    check_parse("10♠", 34);
    check_parse("J♠", 35);
    check_parse("Q♠", 36);
    check_parse("K♠", 37);
    check_parse("A♠", 38);

    check_parse("2♣", 39);
    check_parse("3♣", 40);
    check_parse("4♣", 41);
    check_parse("5♣", 42);
    check_parse("6♣", 43);
    check_parse("7♣", 44);
    check_parse("8♣", 45);
    check_parse("9♣", 46);
    check_parse("10♣", 47);
    check_parse("J♣", 48);
    check_parse("Q♣", 49);
    check_parse("K♣", 50);
    check_parse("A♣", 51);
}

#[test]
fn parse_multiple() {
    assert_eq!(
        CardSet::from_str("8♠2♣Q♣"),
        Ok(CardSet::from_str("8♠")
            .unwrap()
            .add(&CardSet::from_str("2♣").unwrap())
            .add(&CardSet::from_str("Q♣").unwrap()))
    );
    assert_eq!(CardSet::from_str("8♠2♣Q♣").unwrap().count_cards(), 3);
}
#[test]
fn size_of_comb() {
    assert_eq!(std::mem::size_of::<Combination>(), 8);
}

fn comb(input: &[&str]) -> Combination {
    let cards = input
        .iter()
        .map(|s| CardSet::from_str(s).unwrap())
        .fold(CardSet::none(), |a, b| a.add(&b));
    Combination::new(cards)
}

/*
StraightFlush
Poker
Straight */
#[test]
fn straight_flush() {
    assert_eq!(
        comb(&["2♥", "3♥", "4♥", "5♥", "6♥"]).comb_type,
        CombType::StraightFlush
    );
    assert_eq!(
        comb(&["10♠", "J♠", "Q♠", "K♠", "A♠"]).comb_type,
        CombType::StraightFlush
    );
    assert!(comb(&["10♠", "J♠", "Q♠", "K♠", "A♠"]) > comb(&["2♠", "3♥", "4♠", "5♥", "6♠"]));
    assert!(comb(&["10♠", "J♠", "Q♠", "K♠", "A♠"]) > comb(&["9♠", "10♠", "J♠", "Q♠", "K♠"]));
    assert!(comb(&["2♥", "3♥", "4♥", "5♥", "6♥"]) > comb(&["10♠", "J♥", "Q♠", "K♠", "A♠"]));
    assert!(comb(&["2♥", "3♥", "4♥", "5♥", "6♥"]) > comb(&["A♠", "A♥", "A♦", "A♣", "K♠"]));
}
/*
StraightFlush
Poker
FullHouse*/
#[test]
fn poker() {
    assert_eq!(
        comb(&["10♠", "10♥", "10♦", "10♣", "3♠"]).comb_type,
        CombType::Poker
    );
    assert!(comb(&["10♠", "10♥", "10♦", "10♣", "4♠"]) > comb(&["10♠", "10♥", "10♦", "10♣", "3♠"]));
    assert!(comb(&["10♠", "10♥", "10♦", "10♣", "2♠"]) > comb(&["9♠", "9♥", "9♦", "9♣", "A♠"]));
    assert!(comb(&["2♠", "2♥", "2♦", "2♣", "3♠"]) > comb(&["A♠", "A♥", "A♦", "Q♣", "Q♠"]));
}
/*
Poker
FullHouse
Flush*/
#[test]
fn full_house() {
    assert_eq!(
        comb(&["10♠", "10♥", "10♦", "2♣", "2♠"]).comb_type,
        CombType::FullHouse
    );
    assert_eq!(
        comb(&["2♠", "2♥", "2♦", "10♣", "10♠"]).comb_type,
        CombType::FullHouse
    );
    assert!(comb(&["A♠", "A♥", "A♦", "Q♣", "Q♠"]) > comb(&["A♠", "A♥", "A♦", "J♣", "J♠"]));
    assert!(comb(&["10♠", "10♥", "10♦", "2♣", "2♠"]) > comb(&["9♠", "9♥", "9♦", "A♣", "A♠"]));
    assert!(comb(&["2♠", "2♥", "2♦", "3♣", "3♠"]) > comb(&["Q♥", "K♥", "A♥", "K♦", "A♦"]));
    assert!(comb(&["2♠", "2♥", "2♦", "3♣", "3♠"]) > comb(&["A♥", "K♥", "A♦", "K♦", "Q♦"]));
}
/*
FullHouse
Flush
Straight
*/
#[test]
fn flush() {
    assert_eq!(
        comb(&["2♥", "3♥", "5♥", "2♦", "3♦"]).comb_type,
        CombType::Flush
    );
    assert_eq!(
        comb(&["2♠", "3♠", "5♠", "2♣", "3♣"]).comb_type,
        CombType::Flush
    );
    assert!(comb(&["2♥", "3♥", "5♥", "2♦", "3♦"]) > comb(&["2♥", "3♥", "4♥", "2♦", "3♦"]));
    assert!(comb(&["2♥", "3♥", "4♥", "2♦", "3♦"]) > comb(&["10♠", "J♥", "Q♠", "K♠", "A♠"]));
    assert!(comb(&["2♥", "3♥", "4♥", "2♦", "3♦"]) > comb(&["A♥", "K♥", "Q♥", "A♦", "J♦"]));
    assert!(comb(&["2♥", "3♥", "4♥", "2♦", "8♦"]) > comb(&["A♥", "K♥", "Q♥", "9♦", "J♦"]));
    assert!(comb(&["A♥", "2♥", "Q♥", "A♦", "2♦"]) > comb(&["A♥", "K♥", "Q♥", "A♦", "J♦"]));
    assert!(comb(&["5♥", "3♥", "2♥", "5♦", "3♦"]) > comb(&["A♥", "3♥", "2♥", "A♦", "7♦"]));
}
/*
Flush
Straight
Tris */
#[test]
fn straight() {
    assert_eq!(
        comb(&["10♠", "J♥", "Q♠", "K♠", "A♠"]).comb_type,
        CombType::Straight
    );
    assert!(comb(&["10♠", "J♥", "Q♠", "K♠", "A♠"]) > comb(&["9♠", "10♠", "J♠", "Q♥", "K♠"]));
    assert!(comb(&["2♠", "3♥", "4♠", "5♠", "6♠"]) > comb(&["A♠", "A♥", "A♣", "Q♥", "K♠"]));
}
/*
Straight
Tris
TwoPairs */
#[test]
fn tris() {
    assert_eq!(
        comb(&["5♠", "5♥", "5♣", "Q♥", "K♠"]).comb_type,
        CombType::Tris
    );
    assert!(comb(&["5♠", "5♥", "5♣", "Q♥", "K♠"]) > comb(&["5♠", "5♥", "5♣", "Q♥", "J♠"]));
    assert!(comb(&["5♠", "5♥", "5♣", "Q♥", "K♠"]) > comb(&["5♠", "5♥", "5♣", "10♥", "J♠"]));
    assert!(comb(&["7♠", "7♥", "7♣", "2♥", "2♠"]) > comb(&["6♠", "6♥", "6♣", "A♥", "A♠"]));
    assert!(comb(&["2♠", "2♥", "2♣", "3♥", "4♠"]) > comb(&["A♠", "A♥", "K♣", "K♥", "Q♠"]));
}
/*
Tris
TwoPairs
Pair */
#[test]
fn two_pairs() {
    assert_eq!(
        comb(&["10♠", "10♥", "Q♠", "Q♦", "A♠"]).comb_type,
        CombType::TwoPairs
    );
    assert!(comb(&["10♠", "10♥", "Q♠", "Q♦", "A♠"]) > comb(&["10♠", "10♥", "Q♠", "Q♦", "J♠"]));
    assert!(comb(&["10♠", "10♥", "K♠", "K♦", "2♠"]) > comb(&["10♠", "10♥", "Q♠", "Q♦", "A♠"]));
    assert!(comb(&["10♠", "10♥", "Q♠", "Q♦", "A♠"]) > comb(&["10♠", "10♥", "Q♠", "Q♦", "2♠"]));
    assert!(comb(&["2♠", "2♥", "Q♠", "Q♦", "3♠"]) > comb(&["10♠", "10♥", "J♠", "J♦", "A♠"]));
    assert!(comb(&["2♠", "2♥", "3♠", "3♦", "4♠"]) > comb(&["A♠", "A♥", "Q♠", "K♦", "J♠"]));
}
/*
TwoPairs
Pair
HighCard */
#[test]
fn pair() {
    assert_eq!(
        comb(&["2♠", "2♥", "3♠", "4♦", "5♠"]).comb_type,
        CombType::Pair
    );
    assert!(comb(&["10♠", "10♥", "2♠", "3♦", "5♠"]) > comb(&["10♠", "10♥", "2♠", "3♦", "4♠"]));
    assert!(comb(&["10♠", "10♥", "2♠", "3♦", "4♠"]) > comb(&["9♠", "9♥", "A♠", "K♦", "Q♠"]));
    assert!(comb(&["10♠", "10♥", "Q♠", "Q♦", "A♠"]) > comb(&["10♠", "10♥", "Q♠", "Q♦", "2♠"]));
    assert!(comb(&["2♠", "2♥", "3♠", "4♦", "5♠"]) > comb(&["A♠", "9♥", "Q♠", "K♦", "J♠"]));
}
/*
TwoPairs
Pair
HighCard */
#[test]
fn high_card() {
    assert_eq!(
        comb(&["A♠", "K♥", "2♠", "3♠", "4♠"]).comb_type,
        CombType::HighCard
    );
    assert!(comb(&["A♠", "J♥", "Q♠", "K♠", "9♠"]) > comb(&["A♠", "J♥", "Q♠", "K♠", "8♠"]));
    assert!(comb(&["A♠", "10♥", "7♠", "6♠", "5♠"]) > comb(&["A♠", "9♥", "7♠", "6♠", "5♠"]));
    assert!(comb(&["A♠", "2♥", "3♠", "4♠", "5♠"]) > comb(&["K♠", "Q♥", "J♠", "10♠", "8♠"]));
}

#[test]
fn draw() {
    assert_eq!(CardSet::all().draw(52), CardSet::all());
    assert_eq!(CardSet::all().draw(0), CardSet::none());
    let mut deck = CardSet::all();
    let h1 = deck.draw(7);
    assert_eq!(deck.count_cards(), 52 - 7);
    assert_eq!(h1.count_cards(), 7);
    let h1 = deck.draw(5);
    assert_eq!(deck.count_cards(), 52 - 7 - 5);
    assert_eq!(h1.count_cards(), 5);
}
