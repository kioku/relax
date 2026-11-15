#![cfg(unix)]
use std::io;
use std::os::unix::process::CommandExt;
use std::process::Command;

unsafe extern "C" {
    fn fork() -> i32;
    fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;
    fn _exit(status: i32) -> !;
}

#[test]
fn nested_fork_in_pre_exec_without_registered_handlers() {
    let mut simple = Command::new("/bin/sh");
    simple.args(["-c", ":"]);
    unsafe { simple.pre_exec(|| Ok(())); }
    assert!(simple.status().unwrap().success());

    let mut command = Command::new("/bin/sh");
    command.args(["-c", ":"]);
    unsafe {
        command.pre_exec(|| {
            let pid = fork();
            if pid < 0 {
                return Err(io::Error::last_os_error());
            }
            if pid == 0 {
                _exit(0);
            }
            let mut status = 0;
            if waitpid(pid, &mut status, 0) != pid {
                return Err(io::Error::last_os_error());
            }
            if status != 0 {
                return Err(io::Error::from_raw_os_error(10));
            }
            Ok(())
        });
    }
    assert!(command.status().unwrap().success());
}
