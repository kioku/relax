// The binary's script runs in the test's build only: its input is in the test's environment, the
// forced `[env]` HOME applies, and the test's PATH stands over the unforced one.
fn main() {
    println!("cargo:rerun-if-env-changed=LC_ALL");
    assert_eq!(std::env::var("LC_ALL").as_deref(), Ok("C"), "the binary's build script ran again without the test's input");
    assert!(pc_gen::home_configured(), "the build-dependency's script lost the forced [env] HOME");
    let home = std::env::var_os("HOME").is_some_and(|h| std::path::Path::new(&h).ends_with("configured-home"));
    let path = std::env::var("PATH").unwrap_or_default() != "no-tools-for-this-fixture";
    println!("cargo:rustc-env=PREPARED_BUILD=home {home} path {path}");
}
