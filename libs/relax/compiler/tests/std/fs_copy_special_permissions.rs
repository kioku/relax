//! Linux: writing a copy of a file with special mode bits must not restore those bits.
#![cfg(target_os = "linux")]

use std::fs;
use std::os::unix::fs::PermissionsExt;

#[test]
fn copy_restores_only_ordinary_permissions() {
    let dir = std::env::temp_dir().join(format!("relax-copy-mode-{}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    let source = dir.join("source");
    let destination = dir.join("destination");
    fs::write(&source, b"new content").unwrap();
    fs::set_permissions(&source, fs::Permissions::from_mode(0o7540)).unwrap();
    assert_eq!(fs::metadata(&source).unwrap().permissions().mode() & 0o7777, 0o7540);
    fs::write(&destination, b"old content").unwrap();
    fs::set_permissions(&destination, fs::Permissions::from_mode(0o666)).unwrap();

    let result = fs::copy(&source, &destination);
    let mode = fs::metadata(&destination).unwrap().permissions().mode() & 0o7777;
    let contents = fs::read(&destination).unwrap();
    fs::remove_dir_all(&dir).unwrap();
    assert_eq!(result.unwrap(), 11);
    assert_eq!(contents, b"new content");
    assert_eq!(mode, 0o540);
}
