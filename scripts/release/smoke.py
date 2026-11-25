#!/usr/bin/env python3
"""Exercise a release executable, optionally through Wine or QEMU."""

import os
import queue
import re
import subprocess
import sys
import tempfile
import threading
import time


def smoke(command, version):
    env = {key: value for key, value in os.environ.items() if not key.startswith("ZEROFISH_")}
    lines = queue.Queue()
    transcript = []
    with tempfile.TemporaryDirectory() as cwd, subprocess.Popen(
        command, cwd=cwd, env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT, text=True, bufsize=1,
    ) as engine:
        def read_lines():
            for line in engine.stdout:
                lines.put(line.strip())
            lines.put(None)

        def send(line):
            engine.stdin.write(line + "\n")
            engine.stdin.flush()

        def expect(pattern):
            deadline = time.monotonic() + 90
            while time.monotonic() < deadline:
                try:
                    line = lines.get(timeout=deadline - time.monotonic())
                except queue.Empty:
                    break
                if line is None:
                    raise RuntimeError(f"Engine exited before {pattern!r}")
                transcript.append(line)
                if re.fullmatch(pattern, line):
                    return
            raise RuntimeError(f"Timed out waiting for {pattern!r}")

        threading.Thread(target=read_lines, daemon=True).start()
        try:
            send("uci")
            expect(re.escape(f"id name zerofish {version}"))
            expect("uciok")
            send("isready")
            expect("readyok")
            send("position startpos")
            send("eval")
            expect(r"info string Evaluation NNUE -?\d+")
            send("go depth 3")
            expect(r"bestmove (?:([a-h])2\1[34]|b1[ac]3|g1[fh]3)(?: ponder [a-h][1-8][a-h][1-8][qrbn]?)?")
            send("quit")
            if engine.wait(timeout=15) != 0:
                raise RuntimeError(f"Engine exited with status {engine.returncode}")
        finally:
            if engine.poll() is None:
                engine.kill()
            print("\n".join(transcript))


if __name__ == "__main__":
    if len(sys.argv) < 3:
        sys.exit(f"Usage: {sys.argv[0]} VERSION COMMAND [ARGUMENT ...]")
    smoke(sys.argv[2:], sys.argv[1])
