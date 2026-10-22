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
    assert_eq!(engine.read_until("zerofish"), ["zerofish 0.1 by Petr Pravda"]);
    engine.send("uci");
    let lines = engine.read_until("uciok");
    assert!(lines.contains(&"id name zerofish".to_string()));
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
    engine.send("position startpos moves e2e4 e7e5 g1f3");
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
