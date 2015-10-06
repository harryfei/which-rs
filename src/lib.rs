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

use std::path::PathBuf;
use std::{env, fs};

fn is_exist(bin_path: &PathBuf) -> bool {

    match fs::metadata(bin_path).map(|metadata|{
        metadata.is_file()
    }) {
        Ok(true) => true,
        _ => false
    }
}


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
pub fn which(binary_name: &'static str)
             -> Result<PathBuf, &'static str> {

    let path_buf = env::var_os("PATH").and_then(
        |paths| -> Option<PathBuf> {
            for path in env::split_paths(&paths) {
                let bin_path = path.join(binary_name);
                if is_exist(&bin_path) {
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
fn dont_works() {
    let result = which("cargo-no-exist");
    assert_eq!(result, Err("Can not find binary path"))
}
