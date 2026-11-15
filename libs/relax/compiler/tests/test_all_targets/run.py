"""Cargo test target unions, including disabled-but-explicit targets and errors."""
from pathlib import Path
import os
import re
import subprocess
import sys
import tempfile

runner = str(Path(sys.argv[1]).resolve())
with tempfile.TemporaryDirectory(prefix="relax-test-targets-") as directory:
    root = Path(directory)
    (root / "src/bin").mkdir(parents=True)
    (root / "tests").mkdir()
    (root / "examples").mkdir()
    (root / "Cargo.toml").write_text('''[package]
name = "target-union"
version = "0.0.0"
edition = "2024"

[[bin]]
name = "helper"
path = "src/bin/helper.rs"
test = false

[[example]]
name = "enabled"
path = "examples/enabled.rs"
test = true
''')
    (root / "src/lib.rs").write_text('#[test] fn library_case() { assert_eq!(2 + 2, 4); }\n')
    (root / "src/main.rs").write_text('fn main() {}\n#[test] fn binary_case() { assert_eq!(3 + 4, 7); }\n')
    (root / "src/bin/helper.rs").write_text('fn main() {}\n#[test] fn explicit_helper_case() { assert_eq!(4 + 5, 9); }\n')
    (root / "tests/integration.rs").write_text('#[test] fn integration_case() { assert_eq!(5 + 6, 11); }\n')
    (root / "examples/example.rs").write_text('fn main() {}\n#[test] fn example_case() { assert_eq!(6 + 7, 13); }\n')

    (root / "examples/enabled.rs").write_text('fn main() {}\n#[test] fn enabled_example_case() { assert_eq!(7 + 8, 15); }\n')

    def run(command, environment):
        return subprocess.run(command, cwd=root, env=environment, capture_output=True, text=True, timeout=180)

    def verdicts(result):
        return sorted(re.findall(r"^test (\S+) \.\.\. ok$", result.stdout, re.M))

    cases = [
        (["--tests"], ["library_case", "binary_case", "integration_case", "enabled_example_case"]),
        (["--lib", "--tests"], ["library_case", "binary_case", "integration_case", "enabled_example_case"]),
        (["--bin", "helper", "--tests"], ["library_case", "binary_case", "explicit_helper_case", "integration_case", "enabled_example_case"]),
        (["--bins", "--tests"], ["library_case", "binary_case", "explicit_helper_case", "integration_case", "enabled_example_case"]),
        (["--test", "integration", "--tests"], ["library_case", "binary_case", "integration_case", "enabled_example_case"]),
        (["--example", "example", "--tests"], ["library_case", "binary_case", "integration_case", "enabled_example_case", "example_case"]),
        (["--examples"], ["example_case", "enabled_example_case"]),
        (["--test", "missing", "--tests"], ["library_case", "binary_case", "integration_case", "enabled_example_case"]),
        (["--bin", "missing", "--bins"], ["binary_case", "explicit_helper_case"]),
    ]
    for index, (flags, expected) in enumerate(cases):
        environment = dict(os.environ, CARGO_NET_OFFLINE="true", RELAX_OFFLINE="1", CARGO_TARGET_DIR=str(root / "native-target"), RELAX_CACHE_DIR=str(root / f"cache-{index}"))
        native = run(["cargo", "test", "--offline", "--release", *flags, "--", "--test-threads=1"], environment)
        jit = run([runner, "test", "--quiet", "--no-live", "--release", "target-union", *flags, "--", "--test-threads=1"], environment)
        assert native.returncode == 0, native.stderr
        assert jit.returncode == 0, jit.stderr
        assert verdicts(native) == sorted(expected), (flags, verdicts(native), expected)
        assert verdicts(jit) == verdicts(native), (flags, verdicts(jit), verdicts(native))
    for flags in [["--bin", "missing"], ["--test", "missing"], ["--example", "missing"], ["--bin", "missing", "--tests"], ["--example", "missing", "--tests"]]:
        option = flags[0]
        environment = dict(os.environ, CARGO_NET_OFFLINE="true", RELAX_OFFLINE="1", CARGO_TARGET_DIR=str(root / "native-target"), RELAX_CACHE_DIR=str(root / (option[2:] + "-missing-cache")))
        native = run(["cargo", "test", "--offline", "--release", *flags], environment)
        jit = run([runner, "test", "--quiet", "--no-live", "--release", "target-union", *flags], environment)
        assert native.returncode != 0 and jit.returncode != 0, option
        assert not verdicts(native) and not verdicts(jit), option
print("test target unions: nine positive selections and five missing-target errors agree")
