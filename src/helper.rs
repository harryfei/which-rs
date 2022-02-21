use std::path::{Path, PathBuf};

pub trait PathExt {
    fn has_separator(&self) -> bool;

    fn to_absolute<P>(self, cwd: P) -> PathBuf
    where
        P: AsRef<Path>;

    /// Check if given path has extension which in the given vector.
    #[cfg(windows)]
    fn has_executable_extension<S: AsRef<str>>(&self, pathext: &[S]) -> bool;
}

impl PathExt for PathBuf {
    fn has_separator(&self) -> bool {
        self.components().count() > 1
    }

    fn to_absolute<P>(self, cwd: P) -> PathBuf
    where
        P: AsRef<Path>,
    {
        if self.is_absolute() {
            self
        } else {
            let mut new_path = PathBuf::from(cwd.as_ref());
            new_path.push(self);
            new_path
        }
    }

    /// Check if given path has extension which in the given vector.
    #[cfg(windows)]
    fn has_executable_extension<S: AsRef<str>>(&self, pathext: &[S]) -> bool {
        let ext = self.extension().and_then(|e| e.to_str());
        match ext {
            Some(ext) => pathext
                .iter()
                .any(|e| ext.eq_ignore_ascii_case(&e.as_ref()[1..])),
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_has_separator() {
        assert!(PathBuf::from("/foo").has_separator());
        assert!(PathBuf::from("foo/bar").has_separator());

        assert!(!PathBuf::from("foo").has_separator());
    }

    #[test]
    fn test_to_absolute() {
        assert_eq!(
            PathBuf::from("/foo").to_absolute("./hello"),
            PathBuf::from("/foo")
        );

        assert_eq!(
            PathBuf::from("foo/bar").to_absolute("./hello"),
            PathBuf::from("./hello/foo/bar")
        );
    }

    #[test]
    #[cfg(windows)]
    fn test_extension_in_extension_vector() {
        // Case insensitive
        assert!(PathBuf::from("foo.exe").has_executable_extension(&[".COM", ".EXE", ".CMD"]));

        assert!(PathBuf::from("foo.CMD").has_executable_extension(&[".COM", ".EXE", ".CMD"]));
    }

    #[test]
    #[cfg(windows)]
    fn test_extension_not_in_extension_vector() {
        assert!(!PathBuf::from("foo.bar").has_executable_extension(&[".COM", ".EXE", ".CMD"]));
    }
}
