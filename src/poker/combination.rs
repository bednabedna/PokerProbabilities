use super::cardset::CardSet;
use std::cmp::Ordering;

#[derive(Eq, PartialEq, Clone, Copy)]
pub struct Combination {
    comb_type: CombType,
    comb_value: u32,
}

#[derive(Eq, PartialEq, Clone, Debug, Copy)]
pub enum CombType {
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
    pub fn comb_type(&self) -> CombType {
        self.comb_type
    }
    fn new(cards: CardSet) -> Self {
        debug_assert!(cards.count_cards() <= 5);
        let cards = cards.as_u64();
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
                comb_value: if numbers == 0b1000000001111 {
                    // lowest straight should have the minimum value
                    0b1111
                } else {
                    numbers
                },
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
                        comb_value: if numbers == 0b1000000001111 {
                            // lowest straight should have the minimum value
                            0b1111
                        } else {
                            numbers
                        },
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

impl CardSet {
    pub fn comb(&self) -> Combination {
        Combination::new(*self)
    }
}
