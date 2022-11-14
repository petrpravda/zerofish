# 0fish

0fish is a free, open-source, minimalistic engine written in Rust implementing UCI.

## Description

It aims at 3000+ ELO strenght which it doesn't have yet.
It is based upon my previous project "javafish" which was implemented in Java. The javafish chess engine works, but it doesn't have any advanced evaluation besides material evaluation. I estimate its strength roughly about 2400 ELO.
At the time, I'm porting the previous engine (legacyjava) into Rust language (folder called engine).

## Getting Started

### Dependencies

* it doesn't have any 

### Executing program

Get source files
```
git clone https://github.com/petrpravda/0fish.git
```

Go to 0fish directory and make executable with
```
cargo build --manifest-path=engine/Cargo.toml --release
```

Start the chess engine
```
engine/target/release/zerofish
```

## Authors

Petr Pravda  

## Version History

* 0.1
    * Initial Release

## License

This project is licensed under the GPL License

## Acknowledgments

I got inspiration from many GPL chess engines. Most notably from
* [velvet](https://github.com/mhonert/velvet-chess)


Inspiration, code snippets, etc.
* [Square mapping](https://www.chessprogramming.org/Square_Mapping_Considerations)
* [Obstruction Difference algorithm](https://www.chessprogramming.org/Obstruction_Difference)
