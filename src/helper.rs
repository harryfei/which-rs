use std::env;
use std::path::PathBuf;
use std::path::Path;

/// Like `Path::with_extension`, but don't replace an existing extension.
pub fn ensure_exe_extension<T: AsRef<Path>>(path: T) -> PathBuf {
    if env::consts::EXE_EXTENSION.is_empty() {
        // Nothing to do.
        path.as_ref().to_path_buf()
    } else {
        match path.as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case(env::consts::EXE_EXTENSION))
        {
            // Already has the right extension.
            Some(true) => path.as_ref().to_path_buf(),
            _ => {
                // Append the extension.
                let mut s = path.as_ref().to_path_buf().into_os_string();
                s.push(".");
                s.push(env::consts::EXE_EXTENSION);
                PathBuf::from(s)
            }
        }
    }
}

#[test]
fn test_exe_extension() {
    let expected = PathBuf::from("foo").with_extension(env::consts::EXE_EXTENSION);
    assert_eq!(expected, ensure_exe_extension(PathBuf::from("foo")));
    let p = expected.clone();
    assert_eq!(expected, ensure_exe_extension(p));
}

#[test]
#[cfg(windows)]
fn test_exe_extension_existing_extension() {
    assert_eq!(
        PathBuf::from("foo.bar.exe"),
        ensure_exe_extension("foo.bar")
    );
}

#[test]
#[cfg(windows)]
fn test_exe_extension_existing_extension_uppercase() {
    assert_eq!(PathBuf::from("foo.EXE"), ensure_exe_extension("foo.EXE"));
}
