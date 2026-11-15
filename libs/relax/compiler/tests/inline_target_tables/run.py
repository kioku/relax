#!/usr/bin/env python3
"""Compare Cargo inline binary declarations and integration-test execution with rustc.
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
    with tempfile.TemporaryDirectory(prefix="inline-target-tables-") as directory:
        workspace = Path(directory)
        shutil.copy2(fixture / "Cargo.toml", workspace / "Cargo.toml")
        shutil.copytree(fixture / "src", workspace / "src")
        shutil.copytree(fixture / "tests", workspace / "tests")
        shutil.copytree(fixture / "fixture", workspace / "fixture")
        environment["CARGO_TARGET_DIR"] = str(workspace / "target")
        environment["RELAX_CACHE_DIR"] = str(workspace / "cache")
        commands = [
            ["cargo", "test", "--offline", "--release", "--no-fail-fast", "--test", "checks", "--", "--test-threads=1"],
            [runner, "test", "--quiet", "--no-live", "inline-target-regression", "--release", "--test", "checks", "--", "--test-threads=1"],
        ]
        for engine, command in zip(["rustc", "relax"], commands):
            result = subprocess.run(command, cwd=workspace, env=environment, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
            expected = re.search(r"test result: ok\. 1 passed; 0 failed; 0 ignored;", result.stdout)
            if result.returncode != 0 or not expected:
                sys.stdout.write(result.stdout)
                raise SystemExit(f"{engine}: inline binary integration case did not pass")
            print(f"{engine}: 1 named case passed", flush=True)
        print("inline target binary environment and execution agree")


if __name__ == "__main__":
    main()
