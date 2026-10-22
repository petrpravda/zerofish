//! Independently replay a completed match through the Rust rules implementation.
use std::{collections::BTreeMap, fs::File, io::BufReader};
use zerofish::{pgn, position::{BoardPosition, is_threefold_repetition}, types::WHITE};
fn main() {
    let path = std::env::args().nth(1).expect("usage: validate_match games.pgn");
    let mut games = 0;
    let mut plies = 0;
    let mut wdl = [0; 3];
    let mut endings = BTreeMap::new();
    let mut pairs: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for parsed in pgn::read(BufReader::new(File::open(path).unwrap())) {
        let game = parsed.expect("PGN must parse with legal SAN moves");
        let mut position = BoardPosition::from_fen(game.starting_fen()).unwrap();
        for mv in game.moves() {
            assert!(position.state().generate_legal_moves().iter().any(|legal| legal == mv));
            position.do_move(mv);
            plies += 1;
        }
        let state = position.state();
        let termination = game.tag("Termination").unwrap();
        let draw = "1/2-1/2";
        match termination {
            "checkmate" => {
                assert!(state.generate_legal_moves().is_empty() && state.is_king_attacked());
                assert_eq!(game.result(), if state.side_to_play() == WHITE { "0-1" } else { "1-0" });
            }
            "stalemate" => { assert!(state.generate_legal_moves().is_empty() && !state.is_king_attacked()); assert_eq!(game.result(), draw); }
            "threefold repetition" => { assert!(is_threefold_repetition(state, position.keys())); assert_eq!(game.result(), draw); }
            "fifty-move rule" => { assert!(state.half_move_clock >= 100); assert_eq!(game.result(), draw); }
            "insufficient material" => { assert!(state.has_insufficient_material()); assert_eq!(game.result(), draw); }
            "time forfeit" => { /* Clock evidence is checked separately from protocol timestamps. */ }
            other => panic!("Invalid or incomplete game: {other}"),
        }
        *endings.entry(termination.to_string()).or_insert(0) += 1;
        let candidate_white = games % 2 == 0;
        let score = match game.result() {
            "1/2-1/2" => 1,
            "1-0" => if candidate_white { 0 } else { 2 },
            "0-1" => if candidate_white { 2 } else { 0 },
            other => panic!("Unfinished result: {other}"),
        };
        wdl[score] += 1;
        pairs.entry(game.tag("Round").unwrap().to_string()).or_default().push((game.starting_fen().to_string(), game.tag("White").unwrap().to_string()));
        games += 1;
    }
    assert_eq!(games, 100);
    assert_eq!(pairs.len(), 50);
    let mut unique = std::collections::BTreeSet::new();
    for entries in pairs.values() {
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].0, entries[1].0);
        assert_ne!(entries[0].1, entries[1].1);
        assert!(unique.insert(entries[0].0.split_whitespace().take(4).collect::<Vec<_>>().join(" ")));
    }
    println!("Validated {games} games, 50 unique color-reversed position pairs, {plies} legal plies.");
    println!("Zerofish W/D/L: {}/{}/{}", wdl[0], wdl[1], wdl[2]);
    println!("Terminations: {endings:?}");
}
