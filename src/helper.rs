use std::path::Path;

/// Check if given path has extension which in the given vector.
pub fn has_executable_extension<T: AsRef<Path>, S: AsRef<str>>(path: T, exts_vec: &Vec<S>) -> bool {
    match path.as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| exts_vec.iter().any(|ext| e.eq_ignore_ascii_case(&ext.as_ref()[1..])))
    {
        Some(true) => true,
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_extension_in_extension_vector() {
        // Case insensitive
        assert!(
            has_executable_extension(
                PathBuf::from("foo.exe"),
                &vec![".COM", ".EXE", ".CMD"]
            )
        );

        assert!(
            has_executable_extension(
                PathBuf::from("foo.CMD"),
                &vec![".COM", ".EXE", ".CMD"]
            )
        );
    }

    #[test]
    fn test_extension_not_in_extension_vector() {
        assert!(
            !has_executable_extension(
                PathBuf::from("foo.bar"),
                &vec![".COM", ".EXE", ".CMD"]
            )
        );
    }
}
