#!/usr/bin/env python3
"""Drives a UCI engine: sends each command, waits for bestmove after `go` and readyok after `isready`."""
import subprocess
import sys

COMMANDS = [
    "uci",
    "isready",
    "eval",
    "go depth 10",
    "position fen r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    "eval",
    "go depth 9",
    "position startpos moves e2e4 e7e5 g1f3 b8c6 f1b5",
    "go depth 9",
    "setoption name Evaluation value Classical",
    "position startpos",
    "eval",
    "go depth 9",
    "go perft 4",
    "isready",
]


def main():
    engine = sys.argv[1:]
    process = subprocess.Popen(engine, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)

    def wait_for(prefix):
        for line in process.stdout:
            line = line.rstrip("\n")
            print(line, flush=True)
            if line.startswith(prefix):
                return
        raise SystemExit(f"engine exited before {prefix}")

    for command in COMMANDS:
        print(f">>> {command}", flush=True)
        process.stdin.write(command + "\n")
        process.stdin.flush()
        if command.startswith("go perft"):
            wait_for("Nodes searched")
        elif command.startswith("go"):
            wait_for("bestmove")
        elif command == "isready":
            wait_for("readyok")
        elif command == "uci":
            wait_for("uciok")
        elif command == "eval":
            wait_for("info string Evaluation")
    process.stdin.write("quit\n")
    process.stdin.flush()
    process.wait(timeout=10)
    print(f"exit code {process.returncode}")


if __name__ == "__main__":
    main()
