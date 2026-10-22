use std::{env, path::PathBuf};
fn main() {
    println!("cargo:rerun-if-env-changed=ZEROFISH_NNUE_LIBRARY");
    if env::var_os("CARGO_FEATURE_STOCKFISH_NNUE").is_none() { return; }
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
