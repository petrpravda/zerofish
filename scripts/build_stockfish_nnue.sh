#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
build="$root/target/stockfish-nnue"
weights="${1:-$root/../stockfish/src/nn-c9e480e92425.nnue}"
mkdir -p "$build"
echo 'c9e480e92425f887df6ab9b05c6180fbd762b07879df9d9845f1aa75440caa73  '"$weights" | sha256sum -c -
ln -sfn "$(realpath "$weights")" "$build/nn-c9e480e92425.nnue"
flags=(-std=c++17 -O3 -DNDEBUG -fPIC -fvisibility=hidden -ffunction-sections -fdata-sections -fno-semantic-interposition -march=native -DIS_64BIT -DUSE_POPCNT -DUSE_SSE2 -DUSE_SSSE3 -DUSE_SSE41)
if grep -qw avx2 /proc/cpuinfo; then flags+=(-DUSE_AVX2); fi
if grep -qw bmi2 /proc/cpuinfo; then flags+=(-DUSE_PEXT); fi
if grep -Eq '(^| )(avx512_vnni|avx512vnni)( |$)' /proc/cpuinfo; then flags+=(-DUSE_AVX512 -DUSE_VNNI); fi
if grep -Eq '(^| )(avx512_vbmi2|avx512vbmi2)( |$)' /proc/cpuinfo; then flags+=(-DUSE_AVX512ICL); fi
sources=(attacks.cpp bitboard.cpp position.cpp misc.cpp memory.cpp tt.cpp evaluate.cpp uci.cpp nnue/network.cpp nnue/nnue_accumulator.cpp nnue/nnue_misc.cpp nnue/features/k32q2.cpp nnue/features/full_threats.cpp nnue/features/pp_3wide.cpp)
args=()
for source in "${sources[@]}"; do args+=("$root/native/stockfish/$source"); done
cd "$build"
"${CXX:-g++}" "${flags[@]}" -I"$root/native/stockfish" -shared "$root/native/bridge.cpp" "${args[@]}" -Wl,--gc-sections,-z,defs -pthread -o libzerofish_nnue.so
printf '\nBuild Rust with:\nZEROFISH_NNUE_LIBRARY=%q cargo build --release --offline --features stockfish-nnue\n' "$build/libzerofish_nnue.so"
