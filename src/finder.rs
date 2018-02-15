use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use helper::ensure_exe_extension;
use error::*;

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
        let path = ensure_exe_extension(binary_name.as_ref());

        // Does it have a path separator?
        if path.components().count() > 1 {
            if path.is_absolute() {
                if binary_checker.is_valid(&path) {
                    // Already fine.
                    Ok(path)
                } else {
                    // Absolute path but it's not usable.
                    Err(ErrorKind::BadAbsolutePath.into())
                }
            } else {
                // Try to make it absolute.
                let mut new_path = PathBuf::from(cwd.as_ref());
                new_path.push(path);
                let new_path = ensure_exe_extension(new_path);
                if binary_checker.is_valid(&new_path) {
                    Ok(new_path)
                } else {
                    // File doesn't exist or isn't executable.
                    Err(ErrorKind::BadRelativePath.into())
                }
            }
        } else {
            // No separator, look it up in `paths`.
            paths
                .and_then(|paths| {
                    env::split_paths(paths.as_ref())
                        .map(|p| ensure_exe_extension(p.join(binary_name.as_ref())))
                        .skip_while(|p| !(binary_checker.is_valid(p)))
                        .next()
                })
                .ok_or_else(|| ErrorKind::CannotFindBinaryPath.into())
        }
    }
}
