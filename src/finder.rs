use std::env;
use std::ffi::OsStr;
#[cfg(windows)]
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use error::*;
#[cfg(windows)]
use helper::has_executable_extension;

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
                self.check_with_exe_extension(path, binary_checker)
                    .ok_or(ErrorKind::BadAbsolutePath.into())
            } else {
                // Try to make it absolute.
                let mut new_path = PathBuf::from(cwd.as_ref());
                new_path.push(path);
                self.check_with_exe_extension(new_path, binary_checker)
                    .ok_or(ErrorKind::BadRelativePath.into())
            }
        } else {
            // No separator, look it up in `paths`.
            paths
                .and_then(|paths| {
                    env::split_paths(&paths)
                        .map(|p| p.join(binary_name.as_ref()))
                        .map(|p| self.check_with_exe_extension(p, binary_checker))
                        .skip_while(|res| res.is_none())
                        .next()
                })
                .map(|res| res.unwrap())
                .ok_or(ErrorKind::CannotFindBinaryPath.into())
        }
    }

    #[cfg(unix)]
    /// Check if given path with platform specification is valid
    pub fn check_with_exe_extension<T: AsRef<Path>>(&self, path: T, binary_checker: &Checker) -> Option<PathBuf> {
        if binary_checker.is_valid(path.as_ref()) {
            Some(path.as_ref().to_path_buf())
        } else {
            None
        }
    }

    #[cfg(windows)]
    /// Check if given path with platform specification is valid
    pub fn check_with_exe_extension<T: AsRef<Path>>(&self, path: T, binary_checker: &Checker) -> Option<PathBuf> {
        // Read PATHEXT env variable and split it into vector of String
        let path_exts = env::var_os("PATHEXT").unwrap_or(OsString::from(env::consts::EXE_EXTENSION));
        let exe_extension_vec = env::split_paths(&path_exts)
            .filter_map(|e| e.to_str().map(|e| e.to_owned()))
            .collect::<Vec<_>>();

        // Check if path already have executable extension
        if has_executable_extension(&path, &exe_extension_vec) {
            if binary_checker.is_valid(path.as_ref()) {
                Some(path.as_ref().to_path_buf())
            } else {
                None
            }
        } else {
            // Check paths appended with windows executable extensions
            // e.g. path `c:/windows/bin` will expend to:
            // c:/windows/bin.COM
            // c:/windows/bin.EXE
            // c:/windows/bin.CMD
            // ...
            exe_extension_vec.iter()
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
}
