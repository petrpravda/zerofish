//! Streaming PGN lexer and parser.

use std::collections::VecDeque;
use std::fmt;
use std::io::{self, BufRead};

use super::game::{Annotation, Line, PgnGame, Ply, RESULTS};
use super::san;
use crate::board::BoardState;

const MAX_VARIATION_DEPTH: usize = 256;

#[derive(Debug)]
pub enum PgnError {
    Io(io::Error),
    Invalid(String),
}

impl fmt::Display for PgnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PgnError::Io(error) => write!(f, "{error}"),
            PgnError::Invalid(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for PgnError {}

impl From<io::Error> for PgnError {
    fn from(error: io::Error) -> Self {
        PgnError::Io(error)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    LeftTag,
    RightTag,
    LeftVariation,
    RightVariation,
    Str,
    Symbol,
    Number,
    Comment,
    Nag,
    End,
}

#[derive(Clone, Debug)]
struct Token {
    kind: Kind,
    text: String,
    line: usize,
    column: usize,
}

impl Token {
    fn error(&self, message: impl fmt::Display) -> PgnError {
        PgnError::Invalid(format!("{message} at line {}, column {}", self.line, self.column))
    }
}

/// `Character.isWhitespace`: no-break spaces are not whitespace, the file/group/record/unit separators are.
fn is_whitespace(c: char) -> bool {
    c.is_whitespace() && !matches!(c, '\u{a0}' | '\u{2007}' | '\u{202f}') || ('\u{1c}'..='\u{1f}').contains(&c)
}

fn is_result(text: &str) -> bool {
    RESULTS.contains(&text)
}

/// Reads characters one input line at a time, so games are parsed without loading the whole archive.
struct Lexer<R> {
    reader: R,
    buffer: VecDeque<char>,
    current: Option<char>,
    line: usize,
    column: usize,
}

impl<R: BufRead> Lexer<R> {
    fn new(reader: R) -> Result<Self, PgnError> {
        let mut lexer = Lexer { reader, buffer: VecDeque::new(), current: None, line: 1, column: 0 };
        lexer.advance()?;
        if lexer.current == Some('\u{feff}') {
            lexer.advance()?;
            lexer.column = 1;
        }
        Ok(lexer)
    }

    fn advance(&mut self) -> Result<(), PgnError> {
        if self.current == Some('\n') {
            self.line += 1;
            self.column = 0;
        }
        self.column += 1;
        if self.buffer.is_empty() {
            let mut text = String::new();
            self.reader.read_line(&mut text)?;
            self.buffer.extend(text.chars());
        }
        self.current = self.buffer.pop_front();
        Ok(())
    }

    fn next(&mut self) -> Result<Token, PgnError> {
        while let Some(c) = self.current {
            if c == '%' && self.column == 1 {
                while self.current.is_some_and(|c| c != '\n') {
                    self.advance()?;
                }
            } else if is_whitespace(c) {
                self.advance()?;
            } else {
                break;
            }
        }
        let (line, column) = (self.line, self.column);
        let token = |kind, text: String| Token { kind, text, line, column };
        let Some(first) = self.current else { return Ok(token(Kind::End, String::new())) };
        self.advance()?;
        let simple = match first {
            '[' => Some(Kind::LeftTag),
            ']' => Some(Kind::RightTag),
            '(' => Some(Kind::LeftVariation),
            ')' => Some(Kind::RightVariation),
            _ => None,
        };
        if let Some(kind) = simple {
            return Ok(token(kind, first.to_string()));
        }
        let mut text = String::new();
        if first == '{' || first == ';' {
            while let Some(c) = self.current.filter(|&c| if first == '{' { c != '}' } else { c != '\n' && c != '\r' }) {
                text.push(c);
                self.advance()?;
            }
            if first == '{' {
                if self.current.is_none() {
                    return Err(token(Kind::Comment, text).error("Unclosed comment"));
                }
                self.advance()?;
            }
            return Ok(token(Kind::Comment, text));
        }
        if first == '"' {
            loop {
                let c = match self.current {
                    None | Some('\n') | Some('\r') => return Err(token(Kind::Str, text).error("Unclosed tag string")),
                    Some('"') => break,
                    Some('\\') => {
                        self.advance()?;
                        match self.current {
                            Some(c @ ('\\' | '"')) => c,
                            _ => return Err(token(Kind::Str, text).error("Invalid tag escape")),
                        }
                    }
                    Some(c) => c,
                };
                text.push(c);
                self.advance()?;
            }
            self.advance()?;
            return Ok(token(Kind::Str, text));
        }
        if first == '$' {
            while let Some(c) = self.current.filter(char::is_ascii_digit) {
                text.push(c);
                self.advance()?;
            }
            if text.is_empty() {
                return Err(token(Kind::Nag, text).error("Missing numeric annotation"));
            }
            return Ok(token(Kind::Nag, text));
        }
        text.push(first);
        if first == '!' || first == '?' {
            while let Some(c) = self.current.filter(|&c| c == '!' || c == '?') {
                text.push(c);
                self.advance()?;
            }
            let nag = match text.as_str() {
                "!" => "1",
                "?" => "2",
                "!!" => "3",
                "??" => "4",
                "!?" => "5",
                "?!" => "6",
                _ => return Err(token(Kind::Nag, String::new()).error(format!("Invalid annotation: {text}"))),
            };
            return Ok(token(Kind::Nag, nag.to_string()));
        }
        if first.is_ascii_digit() {
            while let Some(c) = self.current.filter(char::is_ascii_digit) {
                text.push(c);
                self.advance()?;
            }
            if self.current == Some('.') {
                while self.current == Some('.') {
                    text.push('.');
                    self.advance()?;
                }
                return Ok(token(Kind::Number, text));
            }
        }
        if first == '}' || first == '.' || first < ' ' {
            return Err(token(Kind::Symbol, String::new()).error(format!("Unexpected character: {first}")));
        }
        while let Some(c) = self.current.filter(|&c| !is_whitespace(c) && !"[](){};\"$!?".contains(c)) {
            text.push(c);
            self.advance()?;
        }
        Ok(token(Kind::Symbol, text))
    }
}

/// Iterator over the games of a PGN stream; stops after the first error.
pub struct PgnReader<R> {
    reader: Option<R>,
    lexer: Option<Lexer<R>>,
    lookahead: Option<Token>,
    failed: bool,
}

impl<R: BufRead> PgnReader<R> {
    pub fn new(reader: R) -> Self {
        PgnReader { reader: Some(reader), lexer: None, lookahead: None, failed: false }
    }

    fn lexer(&mut self) -> Result<&mut Lexer<R>, PgnError> {
        if self.lexer.is_none() {
            let reader = self.reader.take().expect("reader");
            self.lexer = Some(Lexer::new(reader)?);
        }
        Ok(self.lexer.as_mut().unwrap())
    }

    fn peek(&mut self) -> Result<&Token, PgnError> {
        if self.lookahead.is_none() {
            let token = self.lexer()?.next()?;
            self.lookahead = Some(token);
        }
        Ok(self.lookahead.as_ref().unwrap())
    }

    fn peek_kind(&mut self) -> Result<Kind, PgnError> {
        Ok(self.peek()?.kind)
    }

    fn take(&mut self) -> Result<Token, PgnError> {
        self.peek()?;
        Ok(self.lookahead.take().unwrap())
    }

    fn expect(&mut self, kind: Kind) -> Result<Token, PgnError> {
        let token = self.take()?;
        if token.kind != kind {
            return Err(token.error(format!("Expected {kind:?}, got {:?}", token.kind)));
        }
        Ok(token)
    }

    fn next_game(&mut self) -> Result<Option<PgnGame>, PgnError> {
        if self.peek_kind()? == Kind::End {
            return Ok(None);
        }
        let mut tags: Vec<(String, String)> = Vec::new();
        let mut comments = Vec::new();
        loop {
            match self.peek_kind()? {
                Kind::Comment => comments.push(self.take()?.text),
                Kind::LeftTag => {
                    self.take()?;
                    let name = self.expect(Kind::Symbol)?;
                    let value = self.expect(Kind::Str)?;
                    self.expect(Kind::RightTag)?;
                    if tags.iter().any(|(n, _)| *n == name.text) {
                        return Err(name.error(format!("Duplicate tag: {}", name.text)));
                    }
                    tags.push((name.text, value.text));
                }
                _ => break,
            }
        }
        let tag_result = tags.iter().find(|(n, _)| n == "Result").map_or("*".to_string(), |(_, v)| v.clone());
        let beginning = self.peek()?.clone();
        let setup = PgnGame::new(tags.clone(), Line::default(), &tag_result).map_err(|e| beginning.error(e))?;
        let board = setup.starting_board().map_err(|e| beginning.error(format!("Invalid starting FEN: {e}")))?;
        let line = self.read_line(board, comments, 0)?;
        let ending = self.peek()?.clone();
        let result = match ending.kind {
            Kind::Symbol if is_result(&ending.text) => {
                self.take()?;
                ending.text.clone()
            }
            Kind::End => tag_result,
            _ => return Err(ending.error("Missing game result")),
        };
        PgnGame::new(tags, line, &result).map(Some).map_err(|e| ending.error(e))
    }

    fn read_line(&mut self, mut board: BoardState, mut comments: Vec<String>, depth: usize) -> Result<Line, PgnError> {
        if depth > MAX_VARIATION_DEPTH {
            return Err(self.peek()?.error("Variation nesting exceeds 256"));
        }
        let mut moves: Vec<Ply> = Vec::new();
        let mut before: Option<BoardState> = None;
        loop {
            let kind = self.peek_kind()?;
            match kind {
                Kind::End | Kind::RightVariation | Kind::LeftTag => {
                    let token = self.peek()?;
                    if depth > 0 && kind != Kind::RightVariation {
                        return Err(token.error("Unclosed variation"));
                    }
                    if depth == 0 && kind == Kind::RightVariation {
                        return Err(token.error("Unexpected closing variation"));
                    }
                    return Ok(Line { comments, moves });
                }
                Kind::Number => {
                    self.take()?;
                }
                Kind::Comment => {
                    let comment = self.take()?.text;
                    match moves.last_mut() {
                        None => comments.push(comment),
                        Some(ply) => ply.annotations.push(Annotation::Comment(comment)),
                    }
                }
                Kind::Nag => {
                    let token = self.take()?;
                    let value: u8 = token.text.parse().map_err(|_| token.error(format!("NAG outside 0..255: {}", token.text)))?;
                    let ply = moves.last_mut().ok_or_else(|| token.error("Annotation must follow a move"))?;
                    ply.annotations.push(Annotation::Nag(value));
                }
                Kind::LeftVariation => {
                    let token = self.take()?;
                    let Some(start) = before.clone() else { return Err(token.error("Variation must follow a move")) };
                    let variation = self.read_line(start, Vec::new(), depth + 1)?;
                    self.expect(Kind::RightVariation)?;
                    if variation.moves.is_empty() {
                        return Err(token.error("Empty variation"));
                    }
                    moves.last_mut().unwrap().annotations.push(Annotation::Variation(variation));
                }
                Kind::Symbol => {
                    if is_result(&self.peek()?.text) {
                        if depth > 0 {
                            return Err(self.peek()?.error("Game result inside variation"));
                        }
                        return Ok(Line { comments, moves });
                    }
                    let token = self.take()?;
                    let mv = san::parse(&board, &token.text).map_err(|e| token.error(e))?;
                    moves.push(Ply { mv, annotations: Vec::new() });
                    let next = board.do_move(mv);
                    before = Some(std::mem::replace(&mut board, next));
                }
                _ => {
                    let token = self.take()?;
                    return Err(token.error(format!("Unexpected token: {}", token.text)));
                }
            }
        }
    }
}

impl<R: BufRead> Iterator for PgnReader<R> {
    type Item = Result<PgnGame, PgnError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }
        let game = self.next_game().transpose();
        self.failed = matches!(game, Some(Err(_)));
        game
    }
}

/// SAN tokens of the main line, skipping move numbers, comments, NAGs, variations and the result.
pub fn mainline(text: &str) -> Result<Vec<String>, PgnError> {
    let mut lexer = Lexer::new(text.as_bytes())?;
    let mut result = Vec::new();
    let mut depth = 0usize;
    loop {
        let token = lexer.next()?;
        match token.kind {
            Kind::End => break,
            Kind::LeftVariation => depth += 1,
            Kind::RightVariation => {
                depth = depth.checked_sub(1).ok_or_else(|| token.error("Unexpected closing variation"))?;
            }
            Kind::Symbol => {
                if depth == 0 && !is_result(&token.text) {
                    result.push(token.text);
                }
            }
            Kind::Comment | Kind::Number | Kind::Nag => {}
            _ => return Err(token.error("Expected SAN movetext")),
        }
    }
    if depth != 0 {
        return Err(PgnError::Invalid("Unclosed variation".to_string()));
    }
    Ok(result)
}
