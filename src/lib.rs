//! which
//!
//! A Rust equivalent of Unix command "which".
//! # Exmaple:
//!
//! To find wihch rustc exectable binary is using.
//!
//! ``` norun
//! use which::which;
//!
//! let result = which::which("rustc").unwrap();
//! assert_eq!(result, PathBuf::from("/usr/bin/rustc"));
//!
//! ```

extern crate libc;
#[cfg(test)]
extern crate tempdir;

use std::path::{Path,PathBuf};
use std::{env, fs};
#[cfg(unix)]
use std::ffi::CString;
use std::ffi::OsStr;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

fn is_exist(bin_path: &PathBuf) -> bool {

    match fs::metadata(bin_path).map(|metadata|{
        metadata.is_file()
    }) {
        Ok(true) => true,
        _ => false
    }
}

/// Return `true` if `path` is executable.
#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    CString::new(path.as_os_str().as_bytes())
        .and_then(|c| {
            Ok(unsafe { libc::access(c.as_ptr(), libc::X_OK) == 0 })
        })
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(_path: &Path) -> bool { true }


/// Find a exectable binary's path by name.
///
/// # Example
///
/// ``` norun
/// use which::which;
/// use std::path::PathBuf;
///
/// let result = which::which("rustc").unwrap();
/// assert_eq!(result, PathBuf::from("/usr/bin/rustc"));
///
/// ```
pub fn which<T: AsRef<OsStr>>(binary_name: T)
             -> Result<PathBuf, &'static str> {
    which_in(binary_name, env::var_os("PATH"))
}

/// Find `binary_name` in the path list `paths`.
pub fn which_in<T, U>(binary_name: T, paths: Option<U>)
             -> Result<PathBuf, &'static str>
                where T: AsRef<OsStr>,
                U: AsRef<OsStr> {
    let path_buf = paths.and_then(
        |paths| -> Option<PathBuf> {
            for path in env::split_paths(&paths) {
                let bin_path = path.join(binary_name.as_ref());
                if is_exist(&bin_path) && is_executable(&bin_path) {
                    return Some(bin_path);
                }
            }
            return None;

        });

    match path_buf {
        Some(path) => Ok(path),
        None => Err("Can not find binary path")
    }

}

#[cfg(test)]
mod test {
    use super::*;

    use std::env;
    use std::ffi::OsString;
    use std::fs;
    use std::io;
    use std::path::{Path,PathBuf};
    use tempdir::TempDir;

    struct TestFixture {
        /// Temp directory.
        pub tempdir: TempDir,
        /// $PATH
        pub paths: OsString,
        /// Binaries created in $PATH
        pub bins: Vec<PathBuf>,
    }

    const SUBDIRS: &'static [&'static str] = &["a", "b", "c"];
    const BIN_NAME: &'static str = "bin";

    #[cfg(unix)]
    fn mk_bin(dir: &Path, path: &str) -> io::Result<PathBuf> {
        use libc;
        use std::os::unix::fs::OpenOptionsExt;
        let bin = dir.join(path);
        fs::OpenOptions::new()
            .write(true)
            .create(true)
            .mode(0o666 | (libc::S_IXUSR as u32))
            .open(&bin)
            .and_then(|_f| bin.canonicalize())
    }

    fn touch(dir: &Path, path: &str) -> io::Result<PathBuf> {
        let b = dir.join(path);
        fs::File::create(&b)
            .and_then(|_f| b.canonicalize())
    }

    #[cfg(not(unix))]
    fn mk_bin(dir: &Path, path: &str) -> io::Result<PathBuf> {
        touch(dir, path)
    }

    impl TestFixture {
        pub fn new() -> TestFixture {
            let tempdir = TempDir::new("which_tests").unwrap();
            let mut builder = fs::DirBuilder::new();
            builder.recursive(true);
            let mut paths = vec!();
            let mut bins = vec!();
            for d in SUBDIRS.iter() {
                let p = tempdir.path().join(d);
                builder.create(&p).unwrap();
                bins.push(mk_bin(&p, &BIN_NAME).unwrap());
                paths.push(p);
            }
            TestFixture {
                tempdir: tempdir,
                paths: env::join_paths(paths).unwrap(),
                bins: bins,
            }
        }

        #[allow(dead_code)]
        pub fn touch(&self, path: &str) -> io::Result<PathBuf> {
            touch(self.tempdir.path(), &path)
        }

        pub fn mk_bin(&self, path: &str) -> io::Result<PathBuf> {
            mk_bin(self.tempdir.path(), &path)
        }
    }

    fn _which(f: &TestFixture, path: &str) -> Result<PathBuf, &'static str> {
        which_in(path, Some(f.paths.clone()))
    }

    #[test]
    fn it_works() {
        use std::process::Command;
        let result = which("rustc");
        assert!(result.is_ok());

        let which_result = Command::new("which")
            .arg("rustc")
            .output();

        assert_eq!(String::from(result.unwrap().to_str().unwrap()),
                   String::from_utf8(which_result.unwrap().stdout).unwrap().trim());
    }

    #[test]
    fn test_which() {
        let f = TestFixture::new();
        assert_eq!(_which(&f, &BIN_NAME).unwrap(), f.bins[0])
    }

    #[test]
    fn test_which_not_found() {
        let f = TestFixture::new();
        assert!(_which(&f, "a").is_err());
    }

    #[test]
    fn test_which_second() {
        let f = TestFixture::new();
        let b = f.mk_bin("b/another").unwrap();
        assert_eq!(_which(&f, "another").unwrap(), b);
    }

    #[test]
    #[cfg(unix)]
    fn test_which_non_executable() {
        // Shouldn't return non-executable files.
        let f = TestFixture::new();
        f.touch("b/another").unwrap();
        assert!(_which(&f, "another").is_err());
    }
}
