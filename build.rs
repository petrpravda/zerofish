use std::{env, path::PathBuf};
fn main() {
    println!("cargo:rerun-if-env-changed=ZEROFISH_NNUE_LIBRARY");
    println!("cargo:rerun-if-env-changed=ZEROFISH_STOCKFISH_NETWORK");
    println!("cargo:rerun-if-env-changed=ZEROFISH_STOCKFISH_NETWORK_SHA256");
    if env::var_os("CARGO_FEATURE_RUST_STOCKFISH_NNUE").is_some() {
        let path = env::var_os("ZEROFISH_STOCKFISH_NETWORK")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../stockfish/src/nn-c9e480e92425.nnue"));
        let path = path
            .canonicalize()
            .expect("Set ZEROFISH_STOCKFISH_NETWORK to nn-c9e480e92425.nnue");
        let output = std::process::Command::new("sha256sum")
            .arg(&path)
            .output()
            .expect("sha256sum required for pinned network validation");
        assert!(output.status.success());
        let expected = env::var("ZEROFISH_STOCKFISH_NETWORK_SHA256").unwrap_or_else(|_| {
            "c9e480e92425f887df6ab9b05c6180fbd762b07879df9d9845f1aa75440caa73".into()
        });
        assert!(
            expected.len() == 64 && expected.bytes().all(|b| b.is_ascii_hexdigit()),
            "Expected a 64-character SHA-256 digest"
        );
        let expected = expected.to_ascii_lowercase();
        assert_eq!(
            String::from_utf8(output.stdout)
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap(),
            expected,
            "Stockfish network checksum mismatch"
        );
        println!("cargo:rustc-env=ZEROFISH_STOCKFISH_NETWORK_SHA256={expected}");
        println!("cargo:rerun-if-changed={}", path.display());
        println!(
            "cargo:rustc-env=ZEROFISH_STOCKFISH_NETWORK={}",
            path.display()
        );
    }
    if env::var_os("CARGO_FEATURE_STOCKFISH_NNUE").is_none() {
        return;
    }
    let library = env::var_os("ZEROFISH_NNUE_LIBRARY").map(PathBuf::from)
        .expect("Build native NNUE with scripts/build_stockfish_nnue.sh and set ZEROFISH_NNUE_LIBRARY to its .so path");
    let library = library.canonicalize().expect("NNUE library does not exist");
    println!("cargo:rerun-if-changed={}", library.display());
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    std::fs::copy(&library, output.join("libzerofish_nnue.so")).unwrap();
    println!("cargo:rustc-link-search=native={}", output.display());
    println!("cargo:rustc-link-lib=dylib=zerofish_nnue");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", output.display());
}
