//! PGN reading and writing, and SAN/UCI move list conversion.

mod game;
mod reader;
pub mod san;

use std::io::{self, BufRead, Write};

pub use game::{Annotation, Line, PgnGame, Ply, RESULTS};
pub use reader::{mainline, PgnError, PgnReader};

use crate::board::BoardState;
use crate::types::WHITE;

/// Lazily parses the games of a PGN stream, one at a time.
pub fn read<R: BufRead>(reader: R) -> PgnReader<R> {
    PgnReader::new(reader)
}

/// Parses text holding exactly one game.
pub fn parse(text: &str) -> Result<PgnGame, PgnError> {
    let mut games = read(text.as_bytes());
    let game = games.next().transpose()?;
    let more = games.next().transpose()?;
    match (game, more) {
        (Some(game), None) => Ok(game),
        _ => Err(PgnError::Invalid("Expected exactly one PGN game".to_string())),
    }
}

pub fn write(game: &PgnGame) -> Result<String, PgnError> {
    let mut bytes = Vec::new();
    write_to(game, &mut bytes)?;
    Ok(String::from_utf8(bytes).expect("PGN output is UTF-8"))
}

/// Writes canonical PGN: headers, movetext wrapped near 80 columns, result. Does not flush `writer`.
pub fn write_to<W: Write>(game: &PgnGame, writer: &mut W) -> Result<(), PgnError> {
    for (name, value) in game.tags() {
        writeln!(writer, "[{name} \"{}\"]", value.replace('\\', "\\\\").replace('"', "\\\""))?;
    }
    writeln!(writer)?;
    let start = game.starting_board().map_err(PgnError::Invalid)?;
    let mut tokens = Vec::new();
    movetext_tokens(game.line(), &start, &mut tokens).map_err(PgnError::Invalid)?;
    tokens.push(game.result().to_string());
    let mut column = 0;
    for token in tokens {
        let length = token.chars().count();
        if column > 0 {
            if column + length + 1 > 80 {
                writer.write_all(b"\n")?;
                column = 0;
            } else {
                writer.write_all(b" ")?;
                column += 1;
            }
        }
        writer.write_all(token.as_bytes())?;
        column = match token.rfind('\n') {
            Some(index) => token[index + 1..].chars().count(),
            None => column + length,
        };
    }
    writer.write_all(b"\n\n")?;
    Ok(())
}

fn comment_token(text: &str) -> String {
    if text.contains('}') {
        format!(";{text}\n")
    } else {
        format!("{{{text}}}")
    }
}

fn movetext_tokens(line: &Line, start: &BoardState, out: &mut Vec<String>) -> Result<(), String> {
    out.extend(line.comments.iter().map(|c| comment_token(c)));
    let mut board = start.clone();
    let mut number_black = true;
    for ply in &line.moves {
        let number = board.full_move_normalized / 2 + 1;
        if board.side_to_play() == WHITE {
            out.push(format!("{number}."));
        } else if number_black {
            out.push(format!("{number}..."));
        }
        out.push(san::format(&board, ply.mv)?);
        for annotation in &ply.annotations {
            match annotation {
                Annotation::Comment(text) => out.push(comment_token(text)),
                Annotation::Nag(value) => out.push(format!("${value}")),
                Annotation::Variation(variation) => {
                    let mut inner = Vec::new();
                    movetext_tokens(variation, &board, &mut inner)?;
                    out.push(format!("({})", inner.join(" ")));
                }
            }
        }
        number_black = !ply.annotations.is_empty();
        board = board.do_move(ply.mv);
    }
    Ok(())
}

/// Plays SAN moves from the start position, returning them in UCI notation.
pub fn san_to_uci<S: AsRef<str>>(sans: &[S]) -> Result<Vec<String>, String> {
    Ok(san::decode(&BoardState::start_pos(), sans)?.iter().map(|m| m.uci()).collect())
}

/// Main-line SAN movetext (numbers, comments, variations and results are skipped) in UCI notation.
pub fn movetext_to_uci(text: &str) -> Result<Vec<String>, String> {
    san_to_uci(&mainline(text).map_err(|e| e.to_string())?)
}

/// UCI moves played from `start` in SAN.
pub fn uci_to_san(start: &BoardState, ucis: &str) -> Result<Vec<String>, String> {
    san::encode(start, &ucis.split_whitespace().collect::<Vec<_>>())
}

impl From<PgnError> for io::Error {
    fn from(error: PgnError) -> Self {
        match error {
            PgnError::Io(error) => error,
            PgnError::Invalid(message) => io::Error::new(io::ErrorKind::InvalidData, message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trips(game: &PgnGame) {
        let written = write(game).unwrap();
        let reparsed = parse(&written).unwrap();
        assert_eq!(&reparsed, game, "{written}");
        assert_eq!(write(&reparsed).unwrap(), written);
    }

    fn ucis(game: &PgnGame) -> Vec<String> {
        game.moves().iter().map(|m| m.uci()).collect()
    }

    #[test]
    fn round_trips_annotations_nested_variations_and_escaped_tags() {
        let game = parse(
            "[Event \"A \\\"quoted\\\" event\"]\n[Site \"C:\\\\games\"]\n[Result \"1/2-1/2\"]\n\n\
             {Opening} 1.e4! {King pawn} (1.d4 d5 (1...Nf6 $5) 2.c4)\n1...e5 ;a reply\n2.Nf3 $14 Nc6 1/2-1/2\n",
        )
        .unwrap();
        assert_eq!(game.tag("Event"), Some("A \"quoted\" event"));
        assert_eq!(game.tag("Site"), Some("C:\\games"));
        assert_eq!(ucis(&game), ["e2e4", "e7e5", "g1f3", "b8c6"]);
        let first = &game.line().moves[0];
        assert_eq!(first.annotations[0], Annotation::Nag(1));
        let Annotation::Variation(branch) = &first.annotations[2] else { panic!("variation expected") };
        assert_eq!(branch.moves[0].mv.uci(), "d2d4");
        round_trips(&game);
    }

    #[test]
    fn restores_the_position_before_the_move_for_each_variation() {
        let game = parse("1. e4 (1. d4 d5) (1. c4 e5) e5 2. Nf3 (2. Bc4) Nc6 *").unwrap();
        round_trips(&game);
        assert_eq!(game.moves().last().unwrap().uci(), "b8c6");
    }

    #[test]
    fn honors_black_to_move_fen_and_normalizes_setup_headers() {
        let game = parse("[FEN \"7k/8/8/8/8/8/8/7K b - - 0 42\"]\n42... Kg8 *\n").unwrap();
        assert_eq!(game.tag("SetUp"), Some("1"));
        assert!(write(&game).unwrap().contains("42... Kg8 *"));
        round_trips(&game);
    }

    #[test]
    fn preserves_line_comments_that_cannot_be_written_with_braces() {
        round_trips(&parse("1. e4 ;text with } and {\ne5 *").unwrap());
    }

    #[test]
    fn streams_games_lazily() {
        let mut games = read("1. e4 * [Event \"unclosed".as_bytes());
        assert_eq!(ucis(&games.next().unwrap().unwrap()), ["e2e4"]);
        assert!(games.next().unwrap().is_err());
        assert!(games.next().is_none());
    }

    #[test]
    fn supports_multiple_games_and_filtering() {
        let wins: Vec<PgnGame> = read("1.e4 1-0\n1.d4 0-1\n*".as_bytes()).map(Result::unwrap).filter(|g| g.result() == "1-0").collect();
        assert_eq!(wins.len(), 1);
        assert_eq!(ucis(&wins[0]), ["e2e4"]);
        assert!(parse("* *").is_err());
        assert!(parse("").is_err());
    }

    #[test]
    fn accepts_unfinished_fragments_bom_and_escape_lines() {
        let game = parse("\u{feff}%ignore this line\n1.e4 e5").unwrap();
        assert_eq!(game.result(), "*");
        assert_eq!(game.moves().len(), 2);
        round_trips(&game);
    }

    #[test]
    fn reports_malformed_input_with_locations() {
        for text in [
            "1.e4 {unclosed",
            "1.e4 (1.d4 *",
            "1.e4 ) *",
            "1.e4 () *",
            "(1.d4) *",
            "1.e4 $256 *",
            "$1 *",
            "1.e4!!! *",
            "1.e4 e5 2.Bh6 *",
            "[Event \"a\"] [Event \"b\"] *",
            "[Event \"bad\\q\"] *",
            "[Result \"1-0\"] 0-1",
            "[SetUp \"1\"] *",
            "[Variant \"Chess960\"] *",
            "1.e4 [Event \"next\"] *",
        ] {
            let error = parse(text).unwrap_err().to_string();
            assert!(error.contains("line"), "{text}: {error}");
        }
    }

    #[test]
    fn reports_io_failures() {
        struct Broken;
        impl io::Read for Broken {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("fixture"))
            }
        }
        let mut games = read(io::BufReader::new(Broken));
        assert!(matches!(games.next(), Some(Err(PgnError::Io(_)))));
        assert!(games.next().is_none());
    }

    #[test]
    fn legacy_movetext_conversion_handles_annotations_and_all_results() {
        for result in RESULTS {
            let text = format!("1.e4! {{note}} (1.d4 d5) 1...e5 $5 2.Nf3 {result}");
            assert_eq!(mainline(&text).unwrap(), ["e4", "e5", "Nf3"]);
            assert_eq!(movetext_to_uci(&text).unwrap().join(" "), "e2e4 e7e5 g1f3");
        }
        assert!(movetext_to_uci("").unwrap().is_empty());
        assert!(uci_to_san(&BoardState::start_pos(), " \n").unwrap().is_empty());
    }

    #[test]
    fn bounds_variation_depth_and_rejects_invalid_fen() {
        let nested = format!("1.e4 {}{} *", "(1.d4 ".repeat(257), ")".repeat(257));
        assert!(parse(&nested).unwrap_err().to_string().contains("nesting"));
        assert!(parse("[FEN \"bad\"] *").is_err());
    }

    #[test]
    fn validates_constructed_games() {
        let line = Line { comments: vec!["a }\nb".to_string()], moves: Vec::new() };
        assert!(PgnGame::new(Vec::new(), line, "*").is_err());
        assert!(PgnGame::new(vec![("1bad".to_string(), "x".to_string())], Line::default(), "*").is_err());
        let game = PgnGame::new(Vec::new(), Line::default(), "*").unwrap();
        assert_eq!(game.tags()[2], ("Date".to_string(), "????.??.??".to_string()));
    }
}
