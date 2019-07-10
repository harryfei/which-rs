//! which
//!
//! A Rust equivalent of Unix command `which(1)`.
//! # Example:
//!
//! To find which rustc executable binary is using:
//!
//! ``` norun
//! use which::which;
//!
//! let result = which::which("rustc").unwrap();
//! assert_eq!(result, PathBuf::from("/usr/bin/rustc"));
//!
//! ```

extern crate failure;
extern crate libc;

use failure::ResultExt;
mod checker;
mod error;
mod finder;
#[cfg(windows)]
mod helper;

use std::env;
use std::path::{Path, PathBuf};

use std::ffi::OsStr;

use checker::CompositeChecker;
use checker::ExecutableChecker;
use checker::ExistedChecker;
pub use error::*;
use finder::Finder;

/// Find a exectable binary's path by name.
///
/// If given an absolute path, returns it if the file exists and is executable.
///
/// If given a relative path, returns an absolute path to the file if
/// it exists and is executable.
///
/// If given a string without path separators, looks for a file named
/// `binary_name` at each directory in `$PATH` and if it finds an executable
/// file there, returns it.
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
pub fn which<T: AsRef<OsStr>>(binary_name: T) -> Result<PathBuf> {
    let cwd = env::current_dir().context(ErrorKind::CannotGetCurrentDir)?;
    which_in(binary_name, env::var_os("PATH"), &cwd)
}

/// Find `binary_name` in the path list `paths`, using `cwd` to resolve relative paths.
pub fn which_in<T, U, V>(binary_name: T, paths: Option<U>, cwd: V) -> Result<PathBuf>
where
    T: AsRef<OsStr>,
    U: AsRef<OsStr>,
    V: AsRef<Path>,
{
    let binary_checker = CompositeChecker::new()
        .add_checker(Box::new(ExistedChecker::new()))
        .add_checker(Box::new(ExecutableChecker::new()));

    let finder = Finder::new();

    finder.find(binary_name, paths, cwd, &binary_checker)
}
