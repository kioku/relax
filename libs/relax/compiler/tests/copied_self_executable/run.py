#!/usr/bin/env python3
"""Compare Cargo-package self-reexecution with rustc, including a cleared env.

A package test has a registered live executable/run context. Standalone-file
harness setup is a separate interface and must not replace this regression's
Cargo-package reference.
"""
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile


def main():
    runner = str(Path(sys.argv[1]).resolve())
    fixture = Path(__file__).resolve().parent
    environment = dict(os.environ, CARGO_NET_OFFLINE="true")
    with tempfile.TemporaryDirectory(prefix="copied-self-executable-") as directory:
        workspace = Path(directory)
        shutil.copy2(fixture / "Cargo.toml", workspace / "Cargo.toml")
        shutil.copytree(fixture / "src", workspace / "src")
        environment["CARGO_TARGET_DIR"] = str(workspace / "target")
        environment["RELAX_CACHE_DIR"] = str(workspace / "cache")
        commands = [
            ["cargo", "test", "--offline", "--release", "--no-fail-fast", "--lib", "--", "--test-threads=1"],
            [runner, "test", "--quiet", "--no-live", "copied-self-exe-regression", "--release", "--lib", "--", "--test-threads=1"],
        ]
        for engine, command in zip(["rustc", "relax"], commands):
            result = subprocess.run(command, cwd=workspace, env=environment, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
            expected = re.search(r"test result: ok\. 3 passed; 0 failed; 0 ignored;", result.stdout)
            if result.returncode != 0 or not expected:
                sys.stdout.write(result.stdout)
                raise SystemExit(f"{engine}: three named self-reexecution cases did not pass")
            print(f"{engine}: 3 named cases passed", flush=True)
        print("copied self-executable: direct, copied and cleared-environment Cargo tests agree")


if __name__ == "__main__":
    main()
