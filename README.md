# PokerProbabilities
CLI application used to estimate Poker Texas Holdem winning probabilities simulating games with the cards provided.


Cards are given as a string, for example "4CAQ" means 2 cards: 4 of ♥ and ace of ♦.<br/>
The values are '1' or 'A', '2' to '10', 'J' or '11', 'Q' or '12', 'K' or '13'.<br/>
Suits are 'C' or '♥', 'Q' or '♦', 'P' or '♠' and 'F' or '♣'.<br/>
All values and suits can be also lowercase.


```
Options:
  -h, --hand        cards in hand, maximum 2, defaults to no cards
  -t, --table       cards on the table, maximum 3, defaults to no cards
  -p, --players     number of players in game, defaults to 4
  -g, --games       number of rounds to simulate, defaults to 1 million
  --time            display execution time
  --help            display usage information
```
