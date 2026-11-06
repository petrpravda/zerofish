use std::{
    io::Write,
    process::{Command, Stdio},
};

fn run(args: &[&str]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_zerofish"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child
        .stdin
        .take()
        .unwrap()
        .write_all(b"uci\nisready\neval\nquit\n");
    child.wait_with_output().unwrap()
}
#[test]
fn explicit_backends_keep_nnue_and_classical_available() {
    for backend in ["zerofish-nnue", "stockfish-nnue", "classical"] {
        let output = run(&["--evaluation-backend", backend]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("readyok"));
        assert!(text.contains(if backend == "classical" {
            "Evaluation Classical"
        } else {
            "Evaluation NNUE"
        }));
    }
}
#[test]
fn incompatible_missing_and_ambiguous_configuration_fails() {
    for args in [
        vec!["--evaluation-backend", "unknown"],
        vec![
            "--evaluation-backend",
            "zerofish-nnue",
            "--network",
            "/nonexistent/zerofish.nnue",
        ],
        vec![
            "--evaluation-backend",
            "classical",
            "--network",
            "weights.nnue",
        ],
        vec![
            "--evaluation-backend",
            "stockfish-nnue",
            "--network",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/resources/weights-original.nnue"
            ),
        ],
    ] {
        let output = run(&args);
        assert!(!output.status.success());
        assert!(!output.stderr.is_empty());
    }
}
