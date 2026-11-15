#[test]
fn self_execution_keeps_native_empty_filter_exit_status() {
    for clear in [false, true] {
        let mut command = std::process::Command::new(std::env::current_exe().unwrap());
        command.args(["--exact", "__owned_no_matching_test__", "--nocapture"]);
        if clear {
            command.env_clear();
        }
        let output = command.output().unwrap();
        assert!(output.status.success(), "empty-filter child failed (clear={clear}): {}", String::from_utf8_lossy(&output.stderr));
        assert!(String::from_utf8_lossy(&output.stdout).contains("0 passed; 0 failed;"));
    }
}

#[cfg(unix)]
#[test]
fn shell_reexecution_keeps_mode_private() {
    for clear in [false, true] {
        for value in [None, Some("owned-value")] {
            let mut command = std::process::Command::new("/bin/sh");
            command.args(["-c", "exec \"$1\" --exact shell_child_environment --ignored --nocapture", "owned-shell"]);
            command.arg(std::env::current_exe().unwrap());
            if clear { command.env_clear(); }
            match value {
                None => { command.env_remove("RELAX_SELF_EXEC_CHILD"); }
                Some(value) => { command.env("RELAX_SELF_EXEC_CHILD", value); }
            }
            command.env("OWNED_EXPECT_MARKER", value.unwrap_or("absent"));
            let output = command.output().unwrap();
            assert!(output.status.success(), "shell child failed: {}", String::from_utf8_lossy(&output.stderr));
            assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed;"));
        }
    }
}

#[cfg(unix)]
#[test]
#[ignore]
fn shell_child_environment() {
    let expected = std::env::var("OWNED_EXPECT_MARKER").unwrap();
    let marker = std::env::var("RELAX_SELF_EXEC_CHILD").ok();
    if expected == "absent" { assert!(marker.is_none()); }
    else { assert_eq!(marker.as_deref(), Some(expected.as_str())); }
}
