use rand::distributions::{Distribution, Uniform};
use std::fmt::Debug;
use std::ops;
use std::{
    iter::Peekable,
    str::{Chars, FromStr},
};

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct CardSet(u64);

impl CardSet {
    #[allow(dead_code)]
    pub fn none() -> Self {
        Self(0)
    }
    #[allow(dead_code)]
    pub fn all() -> Self {
        Self(0b1111111111111111111111111111111111111111111111111111)
    }

    #[allow(dead_code)]
    pub fn one(index: u32) -> Self {
        assert!(index < 52);
        Self(1 << index)
    }

    pub fn draw(&mut self, mut count: u32) -> CardSet {
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

    pub fn count_cards(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl ops::Not for CardSet {
    type Output = CardSet;

    fn not(self) -> CardSet {
        CardSet((!self.0) & 0b1111111111111111111111111111111111111111111111111111)
    }
}

impl ops::BitOr<CardSet> for CardSet {
    type Output = CardSet;

    fn bitor(self, rhs: CardSet) -> CardSet {
        CardSet(self.0 | rhs.0)
    }
}

impl ops::BitOrAssign<CardSet> for CardSet {
    fn bitor_assign(&mut self, rhs: CardSet) {
        self.0 |= rhs.0;
    }
}

impl ops::BitAnd<CardSet> for CardSet {
    type Output = CardSet;

    fn bitand(self, rhs: CardSet) -> CardSet {
        CardSet(self.0 & rhs.0)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CardParseError {
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
            if (cards & card).is_empty() {
                cards |= card;
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
        for number in 0..13 {
            for suit in 0..4 {
                if (self.0.wrapping_shr(number + suit * 13) & 1) == 1 {
                    if is_first {
                        is_first = false;
                    } else {
                        write!(f, ",")?;
                    }
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
        }

        Ok(())
    }
}
