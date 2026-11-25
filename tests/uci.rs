use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

struct Engine {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Engine {
    fn start() -> Engine {
        let mut child = Command::new(env!("CARGO_BIN_EXE_zerofish")).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().expect("start engine");
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        Engine { child, stdin, stdout }
    }

    fn send(&mut self, command: &str) {
        writeln!(self.stdin, "{command}").unwrap();
        self.stdin.flush().unwrap();
    }

    /// Lines up to and including the first one starting with `prefix`.
    fn read_until(&mut self, prefix: &str) -> Vec<String> {
        let mut lines = Vec::new();
        loop {
            let mut line = String::new();
            assert!(self.stdout.read_line(&mut line).unwrap() > 0, "engine closed output before {prefix}: {lines:?}");
            let line = line.trim_end().to_string();
            let done = line.starts_with(prefix);
            lines.push(line);
            if done {
                return lines;
            }
        }
    }

    fn quit(mut self) {
        self.send("quit");
        assert!(self.child.wait().unwrap().success());
    }
}

#[test]
fn uci_handshake_and_search() {
    let mut engine = Engine::start();
    assert_eq!(engine.read_until("zerofish"), [zerofish::uci::BANNER]);
    engine.send("uci");
    let lines = engine.read_until("uciok");
    assert!(lines.contains(&concat!("id name zerofish ", env!("CARGO_PKG_VERSION")).to_string()));
    assert!(lines.iter().any(|l| l.starts_with("option name Threads")));
    engine.send("isready");
    engine.read_until("readyok");

    engine.send("ucinewgame");
    engine.send("position startpos moves e2e4 e7e5");
    engine.send("go depth 6");
    let lines = engine.read_until("bestmove");
    assert!(lines.iter().any(|l| l.starts_with("info currmove") && l.contains(" depth 6 ")));
    let best = lines.last().unwrap().split_whitespace().nth(1).unwrap().to_string();
    assert_eq!(best.len(), 4);

    // mate in one with the classical evaluation
    engine.send("setoption name Evaluation value Classical");
    engine.send("position fen 6k1/5ppp/8/8/8/8/5PPP/3R2K1 w - - 0 1");
    engine.send("go depth 5");
    let lines = engine.read_until("bestmove");
    assert_eq!(lines.last().unwrap(), "bestmove d1d8");
    assert!(lines.iter().any(|l| l.contains("score mate 1")));
    engine.quit();
}

#[test]
fn time_controls_threads_and_stop() {
    let mut engine = Engine::start();
    engine.send("setoption name Threads value 3");
    engine.send("setoption name Hash value 8");
    engine.send("position startpos");
    engine.send("go movetime 200");
    let lines = engine.read_until("bestmove");
    assert!(lines.iter().any(|l| l.contains("threads=3")));

    engine.send("go wtime 1000 btime 1000 winc 0 binc 0");
    engine.read_until("bestmove");

    engine.send("go infinite");
    engine.send("isready");
    engine.read_until("readyok");
    engine.send("stop");
    let bestmove = engine.read_until("bestmove").pop().unwrap();
    assert_ne!(bestmove, "bestmove 0000");
    engine.quit();
}

#[test]
fn utility_commands_and_errors() {
    let mut engine = Engine::start();
    engine.send("position startpos movespgn e4 e5 Nf3");
    assert_eq!(engine.read_until("e2e4"), [zerofish::uci::BANNER, "e2e4 e7e5 g1f3"]);
    engine.send("go perft 3");
    let lines = engine.read_until("Nodes searched");
    assert_eq!(lines.last().unwrap(), "Nodes searched: 23193");

    engine.send("position startpos moves e2e5");
    engine.read_until("info string Command failed");
    engine.send("setoption name Threads value 0");
    engine.read_until("info string Command failed");
    engine.send("setoption name Nonsense value 1");
    engine.read_until("info string Unknown option Nonsense");
    engine.send("position fen 8/8/8/8/8/8/8/8 w - - 0 1");
    engine.read_until("info string Command failed");

    engine.send("position fen 7k/6Q1/6K1/8/8/8/8/8 b - - 0 1");
    engine.send("go depth 3");
    assert_eq!(engine.read_until("bestmove").pop().unwrap(), "bestmove 0000");
    engine.send("eval");
    engine.read_until("info string Evaluation NNUE");
    engine.send("d");
    let lines = engine.read_until("Legal uci moves");
    assert!(lines.iter().any(|l| l == "Fen: 7k/6Q1/6K1/8/8/8/8/8 b - - 0 1"));
    engine.quit();
}

#[test]
fn display_reports_checking_squares() {
    let mut engine = Engine::start();
    let cases = [
        ("startpos", ""),
        ("fen 4k3/8/8/8/8/8/4R3/K7 b - - 0 1", "e2"),
        ("fen k7/8/8/8/8/8/4r3/4K3 w - - 0 1", "e2"),
        ("fen k7/8/8/8/1b6/8/8/4K3 w - - 0 1", "b4"),
        ("fen k7/8/8/8/4q3/8/8/4K3 w - - 0 1", "e4"),
        ("fen k7/8/8/8/8/5n2/8/4K3 w - - 0 1", "f3"),
        ("fen k7/8/8/8/8/8/3p4/4K3 w - - 0 1", "d2"),
        ("fen 4k3/3P4/8/8/8/8/8/K7 b - - 0 1", "d7"),
        ("fen k3r3/8/8/8/1b6/8/8/4K3 w - - 0 1", "b4 e8"),
        ("fen k3r3/8/8/8/8/8/4P3/4K3 w - - 0 1", ""),
        ("startpos moves f2f3 e7e5 g2g4 d8h4", "h4"),
    ];
    for (position, checkers) in cases {
        engine.send(&format!("position {position}"));
        engine.send("d");
        let lines = engine.read_until("Checkers:");
        assert_eq!(lines.last().unwrap().trim_end(), format!("Checkers: {checkers}").trim_end(), "{position}");
        engine.read_until("Legal uci moves");
    }
    engine.send("position startpos movespgn e4 exd5");
    engine.read_until("info string Command failed: Illegal SAN: exd5");
    engine.quit();
}

#[test]
fn end_of_input_quits() {
    let mut engine = Engine::start();
    engine.send("go infinite");
    drop(engine.stdin);
    let mut output = String::new();
    std::io::Read::read_to_string(&mut engine.stdout, &mut output).unwrap();
    assert!(output.contains("bestmove "));
    assert!(engine.child.wait().unwrap().success());
}

#[test]
fn chess960_protocol_castling_and_search() {
    let mut engine = Engine::start();
    engine.send("uci");
    assert!(engine.read_until("uciok").iter().any(|line| line == "option name UCI_Chess960 type check default false"));
    engine.send("setoption name UCI_Chess960 value true");
    engine.send("ucinewgame");
    engine.send("d");
    assert!(engine.read_until("Legal uci moves").iter().any(|line| line.contains(" w HAha - 0 1")));
    engine.send("position startpos movespgn Nf3 Nf6 g3 g6 Bg2 Bg7 O-O");
    assert_eq!(engine.read_until("g1f3").last().unwrap(), "g1f3 g8f6 g2g3 g7g6 f1g2 f8g7 e1h1");
    engine.send("position fen 4k3/8/8/8/8/8/8/1R1K2R1 w KQ - 0 1 moves d1g1");
    engine.send("d");
    assert!(engine.read_until("Legal uci moves").iter().any(|line| line == "Fen: 4k3/8/8/8/8/8/8/1R3RK1 b - - 1 1"));

    for (threads, evaluation, fen) in [
        (1, "NNUE", "4k3/8/8/8/8/8/8/1R1K2R1 w GB - 0 1"),
        (2, "NNUE", "1r1k2r1/8/8/8/8/8/8/7K b gb - 0 1"),
        (1, "Classical", "4k3/8/8/8/8/8/8/2RK3R w HC - 0 1"),
    ] {
        engine.send(&format!("setoption name Threads value {threads}"));
        engine.send(&format!("setoption name Evaluation value {evaluation}"));
        engine.send(&format!("position fen {fen}"));
        engine.send("go depth 4");
        let lines = engine.read_until("bestmove");
        assert!(!lines.iter().any(|line| line.contains("failed")), "{lines:?}");
        let root = zerofish::board::BoardState::from_fen(fen).unwrap();
        for line in &lines {
            if let Some((_, pv)) = line.split_once(" pv ") {
                let mut state = root.clone();
                for mv in pv.split_whitespace() {
                    state = state.do_move_uci(mv).unwrap_or_else(|| panic!("Illegal PV move {mv}: {line}"));
                }
            }
        }
        let best = lines.last().unwrap().split_whitespace().nth(1).unwrap();
        assert!(root.find_legal_move(best).is_some(), "{lines:?}");
    }

    engine.send("setoption name UCI_Chess960 value false");
    engine.send("position fen r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1 moves e1g1");
    engine.send("d");
    assert!(engine.read_until("Legal uci moves").iter().any(|line| line == "Fen: r3k2r/8/8/8/8/8/8/R4RK1 b kq - 1 1"));
    engine.quit();
}
