use std::env;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use error::*;
use helper::check_extension;

#[cfg(windows)]
lazy_static! {
    static ref EXE_EXTENSION_VEC: Vec<String> = {
        // Read PATHEXT env variable and split it into vector of String
        let path_exts = env::var_os("PATHEXT").unwrap_or(OsString::from(env::consts::EXE_EXTENSION));
        env::split_paths(&path_exts)
            .map(|e| e.to_str().map(|e| e.to_owned()))
            .filter_map(|e| e).collect::<Vec<_>>()
    };
}

pub trait Checker {
    fn is_valid(&self, path: &Path) -> bool;
}

pub struct Finder;

impl Finder {
    pub fn new() -> Finder {
        Finder
    }

    pub fn find<T, U, V>(
        &self,
        binary_name: T,
        paths: Option<U>,
        cwd: V,
        binary_checker: &Checker,
    ) -> Result<PathBuf>
    where
        T: AsRef<OsStr>,
        U: AsRef<OsStr>,
        V: AsRef<Path>,
    {
        let path = PathBuf::from(&binary_name);
        // Does it have a path separator?
        if path.components().count() > 1 {
            if path.is_absolute() {
                check_with_exe_extension(path, binary_checker)
                    .ok_or(ErrorKind::BadAbsolutePath.into())
            } else {
                // Try to make it absolute.
                let mut new_path = PathBuf::from(cwd.as_ref());
                new_path.push(path);
                check_with_exe_extension(new_path, binary_checker)
                    .ok_or(ErrorKind::BadRelativePath.into())
            }
        } else {
            // No separator, look it up in `paths`.
            paths
                .and_then(|paths| {
                    env::split_paths(&paths)
                        .map(|p| p.join(binary_name.as_ref()))
                        .map(|p| check_with_exe_extension(p, binary_checker))
                        .skip_while(|res| res.is_none())
                        .next()
                })
                .map(|res| res.unwrap())
                .ok_or(ErrorKind::CannotFindBinaryPath.into())
        }
    }
}

#[cfg(unix)]
/// Check if given path with platform specification is valid
pub fn check_with_exe_extension<T: AsRef<Path>>(path: T, binary_checker: &Checker) -> Option<PathBuf> {
    if binary_checker.is_valid(&path) {
        Some(path)
    } else {
        None
    }
}

#[cfg(windows)]
/// Check if given path with platform specification is valid
pub fn check_with_exe_extension<T: AsRef<Path>>(path: T, binary_checker: &Checker) -> Option<PathBuf> {
    // Check if path already have executable extension
    if check_extension(&path, &EXE_EXTENSION_VEC) {
        if binary_checker.is_valid(path.as_ref()) {
            Some(path.as_ref().to_path_buf())
        } else {
            None
        }
    } else {
        // Check paths with windows executable extensions
        EXE_EXTENSION_VEC.iter()
            .map(|e| {
                // Append the extension.
                let mut s = path.as_ref().to_path_buf().into_os_string();
                s.push(e);
                PathBuf::from(s)
            })
            .skip_while(|p| !(binary_checker.is_valid(p)))
            .next()
    }
}
