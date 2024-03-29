# `<0)))><`   zerofish

zerofish is a free, open-source, minimalistic chess engine written in Rust implementing UCI.

## Features
* Alpha-beta pruning
* Aspiration search
* Transposition table
* Quiescence
* Iterative deepening
* Null-move heuristics
* Late move reductions
* Zobrist hashing

Not implemented yet:
* Killer heuristics
* History heuristics
* Multi-cut pruning
* Futility pruning
* Reverse futility pruning* 
* Static exchange evaluation
* Razoring
* Adaptive null-move reduction
* Ponder search
* ProbCut
* Move branch search time management
* NNUE
* Advance Score Evaluation

## Scoring
Scoring implemented
* Material balance: This refers to the relative value of the pieces on the board, with the objective of having more valuable pieces than your opponent.

Scoring not implemented
* Pawn structure: The arrangement of pawns on the board can have a significant impact on the game, as they can control key squares and create opportunities for other pieces.
* King safety: The safety of the king is an important factor in determining the overall strength of a position. A exposed or vulnerable king can be a liability and lead to defeat.
* Control of the center: The center of the board is a key strategic area, as it allows pieces to move freely and exert influence over a large portion of the board.
* Development: Developing your pieces (moving them from their starting positions) efficiently can give you a strategic advantage by getting your pieces into the game quickly.
* Mobility: The ability of your pieces to move freely and effectively is an important factor in determining the strength of your position.
* Space: Having more space on the board allows you to maneuver your pieces and create opportunities for attack.
* Weaknesses: Identifying and exploiting weaknesses in your opponent's position, such as pawn weaknesses or undefended pieces, can give you a significant advantage.
* Initiative: Having the initiative means that you are the one dictating the pace of the game and forcing your opponent to respond to your threats.

\* I'm thinking about implementing NNUE in the future, so the scoring above might get be tossed away.
The scoring is very basic currently, and it's limiting the engine's strength. There is still possibility for hybrid score evaluation, however.

## Description

It aims at 3000+ ELO strength which it doesn't have yet.
It is based upon my previous project "javafish" which was implemented in Java. The javafish chess engine works, but it doesn't have any advanced evaluation besides material evaluation. I estimate its strength roughly about 1970 ELO.
At the time, I'm porting the previous engine (legacyjava) into Rust language (folder called engine).

## Getting Started

### Executing program

Get source files
```
git clone https://github.com/petrpravda/zerofish.git
```

Go to zerofish directory and make executable with
```
cargo build --manifest-path=engine/Cargo.toml --release
```

Start the chess engine
```
engine/target/release/zerofish
```

Execute some UCI commands like

`position fen 2k4r/ppp2p2/2nbp3/6q1/3PB2P/P3P1r1/1P1BQ3/2KR3R b - - 0 22`, `d`, `setoption name MultiPV value 3`, `go depth 10`

## Author

Petr Pravda  

## Releases 
[link](https://github.com/petrpravda/zerofish/releases/)

## Version History

* 0.1.0
    * Initial Release

## License

This project is licensed under the GPL License

## Acknowledgments

I got inspiration from many GPL chess engines. Most notably from
* [weiawagaJ](https://github.com/Heiaha/WeiawagaJ)
* [velvet](https://github.com/mhonert/velvet-chess)


Inspiration, code snippets, etc.
* [Square mapping](https://www.chessprogramming.org/Square_Mapping_Considerations)
* [Obstruction Difference algorithm](https://www.chessprogramming.org/Obstruction_Difference)
* [Tricky Perft positions](http://www.talkchess.com/forum3/viewtopic.php?t=47318)
