//! PGN game model: headers, a main line of plies with comments, NAGs and variations, and a result.

use crate::board::BoardState;
use crate::fen::START_POS;
use crate::moves::Move;

pub const RESULTS: [&str; 4] = ["1-0", "0-1", "1/2-1/2", "*"];
const ROSTER: [&str; 7] = ["Event", "Site", "Date", "Round", "White", "Black", "Result"];

#[derive(Clone, Debug, PartialEq)]
pub struct PgnGame {
    tags: Vec<(String, String)>,
    line: Line,
    result: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Line {
    pub comments: Vec<String>,
    pub moves: Vec<Ply>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ply {
    pub mv: Move,
    pub annotations: Vec<Annotation>,
}

/// A variation starts from the position before the move it annotates.
#[derive(Clone, Debug, PartialEq)]
pub enum Annotation {
    Comment(String),
    Nag(u8),
    Variation(Line),
}

fn put(tags: &mut Vec<(String, String)>, name: &str, value: &str) {
    match tags.iter_mut().find(|(n, _)| n == name) {
        Some(entry) => entry.1 = value.to_string(),
        None => tags.push((name.to_string(), value.to_string())),
    }
}

fn valid_tag_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic()) && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// A `}` comment can only be written as a `;` comment, which ends at the line break.
pub fn validate_comment(text: &str) -> Result<(), String> {
    if text.contains('}') && (text.contains('\n') || text.contains('\r')) {
        return Err("Comment cannot be represented in PGN".to_string());
    }
    Ok(())
}

fn validate_line(line: &Line) -> Result<(), String> {
    line.comments.iter().try_for_each(|c| validate_comment(c))?;
    for ply in &line.moves {
        for annotation in &ply.annotations {
            match annotation {
                Annotation::Comment(text) => validate_comment(text)?,
                Annotation::Nag(_) => {}
                Annotation::Variation(variation) if variation.moves.is_empty() => return Err("Empty variation".to_string()),
                Annotation::Variation(variation) => validate_line(variation)?,
            }
        }
    }
    Ok(())
}

impl PgnGame {
    /// Validates the headers and fills in the Seven Tag Roster, `Result` and `SetUp`.
    pub fn new(tags: Vec<(String, String)>, line: Line, result: &str) -> Result<PgnGame, String> {
        if !RESULTS.contains(&result) {
            return Err(format!("Invalid result: {result}"));
        }
        let get = |name: &str| tags.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str());
        if get("Result").is_some_and(|r| r != result) {
            return Err("Result tag disagrees with movetext".to_string());
        }
        for (index, (name, value)) in tags.iter().enumerate() {
            if !valid_tag_name(name) {
                return Err(format!("Invalid tag: {name}"));
            }
            if value.chars().any(|c| c < ' ' || c == '\u{7f}') {
                return Err(format!("Control character in tag: {name}"));
            }
            if tags[..index].iter().any(|(n, _)| n == name) {
                return Err(format!("Duplicate tag: {name}"));
            }
        }
        match (get("SetUp"), get("FEN")) {
            (Some("1"), None) => return Err("SetUp requires FEN".to_string()),
            (Some(s), _) if s != "0" && s != "1" => return Err("Invalid SetUp tag".to_string()),
            (Some("0"), Some(_)) => return Err("FEN conflicts with SetUp 0".to_string()),
            _ => {}
        }
        if let Some(variant) = get("Variant").filter(|v| !v.eq_ignore_ascii_case("Standard")) {
            return Err(format!("Unsupported variant: {variant}"));
        }
        validate_line(&line)?;
        let mut normalized = Vec::with_capacity(tags.len() + ROSTER.len());
        for name in ROSTER {
            put(&mut normalized, name, get(name).unwrap_or(if name == "Date" { "????.??.??" } else { "?" }));
        }
        put(&mut normalized, "Result", result);
        for (name, value) in &tags {
            put(&mut normalized, name, value);
        }
        if get("FEN").is_some() {
            put(&mut normalized, "SetUp", "1");
        }
        Ok(PgnGame { tags: normalized, line, result: result.to_string() })
    }

    /// Headers in output order.
    pub fn tags(&self) -> &[(String, String)] {
        &self.tags
    }

    pub fn tag(&self, name: &str) -> Option<&str> {
        self.tags.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str())
    }

    pub fn line(&self) -> &Line {
        &self.line
    }

    pub fn result(&self) -> &str {
        &self.result
    }

    pub fn starting_fen(&self) -> &str {
        self.tag("FEN").unwrap_or(START_POS)
    }

    pub fn starting_board(&self) -> Result<BoardState, String> {
        BoardState::from_fen(self.starting_fen())
    }

    pub fn moves(&self) -> Vec<Move> {
        self.line.moves.iter().map(|ply| ply.mv).collect()
    }
}
