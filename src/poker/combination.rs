use super::cardset::CardSet;
#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Copy)]
pub struct Combination(u32);

#[derive(Eq, PartialEq, Clone, Debug, Copy)]
pub enum CombinationType {
    RoyalFlush = (1 << 31) + 12,
    StraightFlush = 1 << 31,
    Poker = 1 << 30,
    FullHouse = 1 << 29,
    Flush = 1 << 28,
    Straight = 1 << 27,
    Tris = 1 << 26,
    TwoPairs = 1 << 25,
    Pair = 1 << 24,
    HighCard = 1 << 23,
}

impl Combination {
    /// Position of most significant bit in bits[0..13].
    /// If bits is 0, 0 also is returned
    fn msb(bits: u32) -> u32 {
        assert!(bits < 1 << 13);
        (0..13)
            .rev()
            .find(|&n| bits.wrapping_shr(n) == 1)
            .unwrap_or(0)
    }
    /// Keeps only up to n msb in bits[0..13]
    fn keep_n_bits(bits: u32, n: usize) -> u32 {
        let shift_amount = (0..13)
            .rev()
            .filter(|b| bits & (1 << b) != 0)
            .take(n)
            .last()
            .unwrap_or(13);
        bits.wrapping_shr(shift_amount) << shift_amount
    }
    pub fn as_u32(&self) -> u32 {
        self.0
    }
    #[cfg(test)]
    pub fn category(&self) -> CombinationType {
        let v = self.as_u32();
        assert!(v <= CombinationType::RoyalFlush as u32);
        if v == CombinationType::RoyalFlush as u32 {
            CombinationType::RoyalFlush
        } else if v >= CombinationType::StraightFlush as u32 {
            CombinationType::StraightFlush
        } else if v >= CombinationType::Poker as u32 {
            CombinationType::Poker
        } else if v >= CombinationType::FullHouse as u32 {
            CombinationType::FullHouse
        } else if v >= CombinationType::Flush as u32 {
            CombinationType::Flush
        } else if v >= CombinationType::Straight as u32 {
            CombinationType::Straight
        } else if v >= CombinationType::Tris as u32 {
            CombinationType::Tris
        } else if v >= CombinationType::TwoPairs as u32 {
            CombinationType::TwoPairs
        } else if v >= CombinationType::Pair as u32 {
            CombinationType::Pair
        } else {
            CombinationType::HighCard
        }
    }
    pub fn name(&self) -> &str {
        let v = self.as_u32();
        assert!(v <= CombinationType::RoyalFlush as u32);
        if v == CombinationType::RoyalFlush as u32 {
            "RoyalFlush"
        } else if v >= CombinationType::StraightFlush as u32 {
            "StraightFlush"
        } else if v >= CombinationType::Poker as u32 {
            "Poker"
        } else if v >= CombinationType::FullHouse as u32 {
            "FullHouse"
        } else if v >= CombinationType::Flush as u32 {
            "Flush"
        } else if v >= CombinationType::Straight as u32 {
            "Straight"
        } else if v >= CombinationType::Tris as u32 {
            "Tris"
        } else if v >= CombinationType::TwoPairs as u32 {
            "TwoPairs"
        } else if v >= CombinationType::Pair as u32 {
            "Pair"
        } else {
            "HighCard"
        }
    }
    fn straight_bits(numbers: u32) -> u32 {
        // add ace (numbers >> 12) to check for the minimal straight (A-5)
        let numbers_ace = (numbers << 1) | (numbers >> 12);
        numbers & numbers_ace & (numbers_ace << 1) & (numbers_ace << 2) & (numbers_ace << 3)
    }
    fn new(cards: CardSet) -> Self {
        debug_assert!(cards.count_cards() <= 8);
        let cards = cards.as_u64();
        let suit_mask = 0b1111111111111;
        let n1 = (cards & suit_mask) as u32;
        let n2 = (cards.wrapping_shr(1 * 13) & suit_mask) as u32;
        let n3 = (cards.wrapping_shr(2 * 13) & suit_mask) as u32;
        let n4 = (cards.wrapping_shr(3 * 13) & suit_mask) as u32;
        let (flush_count, flush_value) = (n1.count_ones(), n1)
            .max((n2.count_ones(), n2))
            .max((n3.count_ones(), n3))
            .max((n4.count_ones(), n4));
        let is_flush = flush_count >= 5;
        if is_flush {
            let straight_value = Combination::straight_bits(flush_value);
            let is_royal_flush = straight_value != 0;
            if is_royal_flush {
                // highest straight wins
                return Combination(
                    CombinationType::StraightFlush as u32 | Combination::msb(straight_value),
                );
            }
        }
        let numbers = n1 | n2 | n3 | n4;
        let poker_bits = n1 & n2 & n3 & n4;
        let is_poker = poker_bits != 0;
        if is_poker {
            let highest_poker = Combination::msb(poker_bits);
            // highest poker or highest card not in poker
            Combination(
                CombinationType::Poker as u32
                    | ((highest_poker + 1) << 13)
                    | Combination::msb(numbers & !(1 << highest_poker)),
            )
        } else {
            let tris_bits = (n1 & n2 & n3) | (n1 & n2 & n4) | (n1 & n3 & n4) | (n2 & n3 & n4);
            let highest_tris = Combination::msb(tris_bits);
            let highest_tris_bit = tris_bits & (1 << highest_tris);
            // all pairs that aren't also part of the highest tris_value
            let pair_bits = ((n1 & n2) | (n1 & n3) | (n1 & n4) | (n2 & n3) | (n2 & n4) | (n3 & n4))
                & !highest_tris_bit;
            let is_tris = highest_tris_bit != 0;
            let is_pair = pair_bits != 0;
            let is_full_house = is_tris && is_pair;
            if is_full_house {
                // highest tries or highest pair
                Combination(
                    CombinationType::FullHouse as u32
                        | ((highest_tris + 1) << 13)
                        | Combination::msb(pair_bits),
                )
            } else if is_flush {
                // 5 highest cards
                Combination(
                    CombinationType::Flush as u32 | Combination::keep_n_bits(flush_value, 5),
                )
            } else {
                let straight_bits = Combination::straight_bits(numbers);
                let is_straight = straight_bits != 0;
                if is_straight {
                    // highest straight
                    Combination(CombinationType::Straight as u32 | Combination::msb(straight_bits))
                } else if is_tris {
                    Combination(
                        CombinationType::Tris as u32| // uses bit 26
                         ((highest_tris + 1) << 13) // uses bits 18 to 13
                            | Combination::keep_n_bits(numbers & !highest_tris_bit, 2), // highest tris or 2 highest cards
                    )
                } else if pair_bits != 0 {
                    if pair_bits.count_ones() >= 2 {
                        let first_2_pairs = Combination::keep_n_bits(pair_bits, 2);
                        Combination(
                            CombinationType::TwoPairs as u32  // uses bit 25
                            | (first_2_pairs << 6) // uses bits 19 to 6
                            | Combination::msb(numbers & !first_2_pairs), // highest pair or second highest pair or highest card
                        )
                    } else {
                        Combination(
                            CombinationType::Pair as u32 // uses bit 24
                            | ((Combination::msb(pair_bits) + 1) << 13) // uses bits 18 to 13
                            | Combination::keep_n_bits(numbers & !pair_bits, 3), // pair or highest 3 cards
                        )
                    }
                } else {
                    Combination(
                        CombinationType::HighCard as u32 | Combination::keep_n_bits(numbers, 5), // highest 5 cards
                    )
                }
            }
        }
    }
}

impl CardSet {
    pub fn comb(&self) -> Combination {
        Combination::new(*self)
    }
}

#[test]
fn msb_test() {
    assert_eq!(Combination::msb(0b10100), 4);
    assert_eq!(Combination::msb(0b1111111111111), 12);
    assert_eq!(Combination::msb(0b1), 0);
    assert_eq!(Combination::msb(0b0), 0);
}

#[test]
fn keep_n_bits_test() {
    assert_eq!(Combination::keep_n_bits(0b10100, 0), 0b0);
    assert_eq!(Combination::keep_n_bits(0b10100, 1), 0b10000);
    assert_eq!(Combination::keep_n_bits(0b10100, 2), 0b10100);
    assert_eq!(Combination::keep_n_bits(0b10110, 2), 0b10100);
    assert_eq!(Combination::keep_n_bits(0b10110, 3), 0b10110);
    assert_eq!(Combination::keep_n_bits(0b10100, 3), 0b10100);
    assert_eq!(Combination::keep_n_bits(0b10100, 4), 0b10100);
    assert_eq!(Combination::keep_n_bits(0b00000, 4), 0b00000);
}
