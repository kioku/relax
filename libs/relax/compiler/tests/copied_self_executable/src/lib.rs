#![cfg(any(target_os = "linux", target_os = "macos"))]

use std::process::Command;

fn nested_case(name: &str, copied: bool, clear: bool, guard: &str) {
    if std::env::var_os(guard).is_some() {
        assert!(std::env::args().any(|arg| arg == name));
        return;
    }
    let original = std::env::current_exe().unwrap();
    let directory =
        std::env::temp_dir().join(format!("owned-test-reexec-{}-{copied}", std::process::id()));
    let executable = if copied {
        std::fs::create_dir(&directory).unwrap();
        let executable = directory.join("copied-test-program");
        std::fs::copy(&original, &executable).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        executable
    } else {
        original
    };
    let mut command = Command::new(&executable);
    if clear {
        command.env_clear();
    }
    let result = command
        .args(["--exact", name, "--nocapture"])
        .env(guard, "set")
        .status()
        .unwrap();
    if copied {
        std::fs::remove_dir_all(&directory).unwrap();
    }
    assert!(result.success(), "selected self-reexecuted test must pass");
}

#[test]
fn direct_test_self_reexecution() {
    nested_case(
        "direct_test_self_reexecution",
        false,
        false,
        "OWNED_DIRECT_REEXEC_CHILD",
    );
}

#[test]
fn copied_test_self_reexecution() {
    nested_case(
        "copied_test_self_reexecution",
        true,
        false,
        "OWNED_COPIED_REEXEC_CHILD",
    );
}

#[test]
fn copied_test_self_reexecution_with_cleared_environment() {
    nested_case(
        "copied_test_self_reexecution_with_cleared_environment",
        true,
        true,
        "OWNED_CLEARED_REEXEC_CHILD",
    );
}
