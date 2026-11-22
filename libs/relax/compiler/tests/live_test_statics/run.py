#!/usr/bin/env python3
"""Check issue #260: a body edit resets statics unless a test left a thread alive.
Run with `python3 run.py /path/to/relax`. The proc macro is local; no registry is needed.
"""
import os
from pathlib import Path
import shutil
import signal
import subprocess as sp
import sys
import tempfile
import time


HERE = Path(__file__).resolve().parent


def wait_for_verdict(p, log, group, ready):
    deadline = time.monotonic() + 45
    while time.monotonic() < deadline:
        output = log.read_text(errors="replace")
        verdict = next((line for line in output.splitlines()
                        if line.startswith(f"testverdict group={group} ")), None)
        if verdict is not None and ready.is_file():
            return verdict, output
        if p.poll() is not None:
            break
        time.sleep(0.05)
    raise RuntimeError(f"no verdict for group {group}:\n{log.read_text(errors='replace')}")


def run(jit, park_worker):
    with tempfile.TemporaryDirectory(prefix="live-test-statics-") as tmp:
        root = Path(tmp)
        pkg = root / "pkg"
        pkg.mkdir()
        shutil.copy(HERE / "Cargo.toml", pkg)
        for name in ("src", "macros"):
            shutil.copytree(HERE / name, pkg / name)
        ready = root / "ready"
        registry = root / "registry"
        registry.mkdir()
        env = dict(os.environ, RELAX_CACHE_DIR=str(root / "cache"),
                   MAKEPAD_REMOTE_REGISTRY=str(registry), RELAX_LIVE_READY=str(ready))
        log = root / "live.log"
        cmd = [jit, "test", "--offline", "--live", "live-test-statics", "--lib"]
        if park_worker:
            cmd += ["--features", "park-worker"]
        with log.open("w") as output:
            p = sp.Popen(cmd, cwd=pkg, env=env, stdin=sp.DEVNULL,
                         stdout=output, stderr=sp.STDOUT, start_new_session=True)
        try:
            first, output = wait_for_verdict(p, log, 0, ready)
            if "ran=2 passed=2 failed=0" not in first:
                raise RuntimeError(output)
            lib = pkg / "src" / "lib.rs"
            edit = lib.with_suffix(".save")
            edit.write_text(lib.read_text().replace("swap(1, Relaxed)", "swap(2, Relaxed)"))
            edit.replace(lib)
            verdict, output = wait_for_verdict(p, log, 1, ready)
            restart = next((line for line in output.splitlines()
                            if line.startswith("relax live: restarting (")), "")
            if park_worker:
                if "ran=2 passed=2 failed=0" not in verdict or "selected=all skipped=0" not in verdict or "parked-test-worker" not in restart:
                    raise RuntimeError(output)
                print(restart)
            else:
                if "ran=1 passed=2 failed=0" not in verdict or "selected=affected skipped=1 statics_reset=1" not in verdict or restart:
                    raise RuntimeError(output)
            print(verdict)
        finally:
            try:
                os.killpg(p.pid, signal.SIGTERM)
                p.wait(timeout=10)
            except ProcessLookupError:
                p.wait()
            except sp.TimeoutExpired:
                os.killpg(p.pid, signal.SIGKILL)
                p.wait()


if __name__ == "__main__":
    jit = str(Path(sys.argv[1]).resolve())
    run(jit, park_worker=False)
    run(jit, park_worker=True)
    print("live_test_statics: ok")
