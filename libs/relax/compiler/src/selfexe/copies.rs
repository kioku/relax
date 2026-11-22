//! A byte copy cannot find the live run entries beside the original executable.
//! Keep only their location in file metadata, never the build environment. The
//! file's write fingerprint invalidates the hint when its contents are replaced.
use std::path::{Path, PathBuf};

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "ios"
))]
mod imp {
    use super::*;
    use std::ffi::{c_char, c_int, c_void, OsString};
    use std::fs::File;
    use std::io;
    use std::mem::ManuallyDrop;
    use std::os::fd::{AsRawFd, FromRawFd};
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    use std::os::unix::fs::MetadataExt;

    #[cfg(any(target_os = "linux", target_os = "android"))]
    const ATTRIBUTE: &[u8] = b"user.makepad.relax.live-executable\0";
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    const ATTRIBUTE: &[u8] = b"com.makepad.relax.live-executable\0";
    const MAGIC: &[u8] = b"relax-live-copy\0";

    extern "C" {
        #[cfg(any(target_os = "linux", target_os = "android"))]
        fn fgetxattr(fd: c_int, name: *const c_char, value: *mut c_void, size: usize) -> isize;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        fn fgetxattr(
            fd: c_int,
            name: *const c_char,
            value: *mut c_void,
            size: usize,
            position: u32,
            options: c_int,
        ) -> isize;
        #[cfg(any(target_os = "linux", target_os = "android"))]
        fn fsetxattr(
            fd: c_int,
            name: *const c_char,
            value: *const c_void,
            size: usize,
            flags: c_int,
        ) -> c_int;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        fn fsetxattr(
            fd: c_int,
            name: *const c_char,
            value: *const c_void,
            size: usize,
            position: u32,
            options: c_int,
        ) -> c_int;
        #[cfg(any(target_os = "linux", target_os = "android"))]
        fn fremovexattr(fd: c_int, name: *const c_char) -> c_int;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        fn fremovexattr(fd: c_int, name: *const c_char, options: c_int) -> c_int;
    }

    fn get(fd: c_int) -> Option<Vec<u8>> {
        let name = ATTRIBUTE.as_ptr().cast();
        #[cfg(any(target_os = "linux", target_os = "android"))]
        let read = |p, n| unsafe { fgetxattr(fd, name, p, n) };
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        let read = |p, n| unsafe { fgetxattr(fd, name, p, n, 0, 0) };
        let size = read(std::ptr::null_mut(), 0);
        if size < 0 {
            return None;
        }
        let mut data = vec![0; size as usize];
        let size = read(data.as_mut_ptr().cast(), data.len());
        if size < 0 {
            return None;
        }
        data.truncate(size as usize);
        Some(data)
    }

    fn fingerprint(file: &File) -> Option<Vec<u8>> {
        let metadata = file.metadata().ok()?;
        let written = metadata
            .modified()
            .ok()?
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?;
        let mut bytes = MAGIC.to_vec();
        bytes.extend_from_slice(&metadata.len().to_le_bytes());
        bytes.extend_from_slice(&written.as_nanos().to_le_bytes());
        Some(bytes)
    }

    fn origin_file(file: &File) -> Option<PathBuf> {
        let data = get(file.as_raw_fd())?;
        let prefix = fingerprint(file)?;
        let path = data.strip_prefix(prefix.as_slice())?;
        let path = PathBuf::from(OsString::from_vec(path.to_vec()));
        path.is_absolute().then_some(path)
    }

    pub(super) fn origin(path: &Path) -> Option<PathBuf> {
        origin_file(&File::open(path).ok()?)
    }

    fn put(file: &File, origin: &Path) -> io::Result<()> {
        let mut data = fingerprint(file)
            .ok_or_else(|| io::Error::other("executable copy has no write fingerprint"))?;
        data.extend_from_slice(origin.as_os_str().as_bytes());
        #[cfg(any(target_os = "linux", target_os = "android"))]
        let result = unsafe {
            fsetxattr(
                file.as_raw_fd(),
                ATTRIBUTE.as_ptr().cast(),
                data.as_ptr().cast(),
                data.len(),
                0,
            )
        };
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        let result = unsafe {
            fsetxattr(
                file.as_raw_fd(),
                ATTRIBUTE.as_ptr().cast(),
                data.as_ptr().cast(),
                data.len(),
                0,
                0,
            )
        };
        if result == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn copy_origin_tracks_file_bytes_without_changing_them() {
            let dir =
                std::env::temp_dir().join(format!("relax-copy-origin-{}", std::process::id()));
            std::fs::create_dir(&dir).unwrap();
            let path = dir.join("copy");
            let file = File::options()
                .read(true)
                .write(true)
                .create_new(true)
                .open(&path)
                .unwrap();
            file.set_len(8).unwrap();
            let bytes = std::fs::read(&path).unwrap();
            put(&file, &dir).unwrap();
            assert_eq!(origin(&path), Some(dir.clone()));
            assert_eq!(
                std::fs::read(&path).unwrap(),
                bytes,
                "origin metadata is not executable content"
            );
            let renamed = dir.join("renamed");
            std::fs::rename(&path, &renamed).unwrap();
            assert_eq!(
                origin(&renamed),
                Some(dir.clone()),
                "renaming preserves file identity"
            );
            file.set_len(9).unwrap();
            assert_eq!(
                origin(&renamed),
                None,
                "replacement contents cannot inherit an earlier program"
            );
            file.set_len(8).unwrap();
            put(&file, &dir).unwrap();
            let unrelated = File::options()
                .read(true)
                .write(true)
                .create_new(true)
                .open(dir.join("other"))
                .unwrap();
            note(unrelated.as_raw_fd(), file.as_raw_fd());
            assert_eq!(
                origin(&renamed),
                None,
                "an unrelated source must clear an earlier executable origin"
            );
            drop(unrelated);
            drop(file);
            std::fs::remove_dir_all(dir).unwrap();
        }
    }

    pub(super) fn note(from: c_int, to: c_int) {
        // These descriptors belong to Relax std's open files, not this borrowed view.
        let from = ManuallyDrop::new(unsafe { File::from_raw_fd(from) });
        let to = ManuallyDrop::new(unsafe { File::from_raw_fd(to) });
        // Use exactly the protected executable identity exposed by Relax std's
        // current_exe(), including a run entered through its launch process.
        let original = std::env::var_os("RELAX_SELF_EXEC").and_then(|line| {
            let bytes = line.as_encoded_bytes().split(|&b| b == 0x1f).next()?;
            let path = PathBuf::from(OsString::from_vec(bytes.to_vec()));
            let a = from.metadata().ok()?;
            let b = std::fs::metadata(&path).ok()?;
            (a.dev() == b.dev() && a.ino() == b.ino())
                .then(|| path.parent().map(Path::to_path_buf))
                .flatten()
        });
        let trace = std::env::var("RELAX_TRACE")
            .is_ok_and(|value| value.split(',').any(|part| part == "selfexe"));
        let inherited = if original.is_none() {
            origin_file(&from)
        } else {
            None
        };
        if trace {
            eprintln!(
                "selfexe: copy source_is_program={} source_is_copy={}",
                original.is_some(),
                inherited.is_some()
            );
        }
        if let Some(origin) = original.or(inherited) {
            // Copying bytes remains an ordinary fs::copy even where this filesystem
            // cannot store executable-origin metadata; it does not become an I/O error.
            let result = put(&to, &origin);
            if trace {
                eprintln!(
                    "selfexe: copied executable origin recorded={}",
                    result.is_ok()
                );
            }
        } else {
            // A destination may previously have held a copied executable. Never let
            // replacement bytes inherit its earlier live program's identity.
            #[cfg(any(target_os = "linux", target_os = "android"))]
            unsafe {
                fremovexattr(to.as_raw_fd(), ATTRIBUTE.as_ptr().cast());
            }
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            unsafe {
                fremovexattr(to.as_raw_fd(), ATTRIBUTE.as_ptr().cast(), 0);
            }
        }
    }
}

pub(super) fn origin(path: &Path) -> Option<PathBuf> {
    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios"
    ))]
    {
        imp::origin(path)
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios"
    )))]
    {
        let _ = path;
        None
    }
}

pub(crate) extern "C" fn note(from: i32, to: i32) {
    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios"
    ))]
    imp::note(from, to);
    #[cfg(not(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios"
    )))]
    let _ = (from, to);
}
