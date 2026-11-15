#[test]
fn inline_binary_is_available_to_integration_test() {
    assert_eq!(inline_target_regression::value(), 42);
    let executable = option_env!("CARGO_BIN_EXE_fixture")
        .expect("inline bin declarations expose Cargo's integration-test executable");
    let status = std::process::Command::new(executable).status().unwrap();
    assert_eq!(status.code(), Some(23));
}
