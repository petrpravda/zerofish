use zerofish::board::BoardState;
use zerofish::pgn;

#[test]
fn converts_5000_games_between_san_and_uci() {
    let sans = include_str!("fixtures/pgn.5000.games.txt");
    let ucis = include_str!("fixtures/uci.5000.games.txt");
    let start = BoardState::start_pos();
    let mut games = 0;
    for (san, uci) in sans.lines().zip(ucis.lines()) {
        let parsed = pgn::parse(san).unwrap_or_else(|e| panic!("{san}: {e}"));
        assert_eq!(pgn::parse(&pgn::write(&parsed).unwrap()).unwrap(), parsed, "{san}");
        assert_eq!(pgn::movetext_to_uci(san).unwrap().join(" "), uci);
        assert_eq!(pgn::uci_to_san(&start, uci).unwrap().join(" "), san);
        games += 1;
    }
    // the fixture holds 4980 games despite its name
    assert_eq!(games, sans.lines().count());
    assert!(games > 4900);
}
