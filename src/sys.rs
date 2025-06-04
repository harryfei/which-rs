use std::borrow::Cow;
use std::env::VarError;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::path::PathBuf;

pub trait SysReadDirEntry {
    /// Gets the file name of the directory entry, not the full path.
    fn file_name(&self) -> OsString;
    /// Gets the full path of the directory entry.
    fn path(&self) -> PathBuf;
}

pub trait SysMetadata {
    /// Gets if the path is a symlink.
    fn is_symlink(&self) -> bool;
    /// Gets if the path is a file.
    fn is_file(&self) -> bool;
}

/// Represents the system that `which` interacts with to get information
/// about the environment and file system.
///
/// ### How to use in Wasm without WASI
///
/// WebAssembly without WASI does not have a filesystem, but using this crate is possible in `wasm32-unknown-unknown` targets by disabling default features:
///
/// ```toml
/// which = { version = "...", default-features = false }
/// ```
///
// Then providing your own implementation of the `which::sys::Sys` trait:
///
/// ```rs
/// use which::WhichConfig;
///
/// struct WasmSys;
///
/// impl which::sys::Sys for WasmSys {
///     // it is up to you to implement this trait based on the
///     // environment you are running WebAssembly in
/// }
///
/// let paths = WhichConfig::new_with_sys(WasmSys)
///     .all_results()
///     .unwrap()
///     .collect::<Vec<_>>();
/// ```
pub trait Sys: Clone {
    type ReadDirEntry: SysReadDirEntry;
    type Metadata: SysMetadata;

    /// Check if the current platform is Windows.
    ///
    /// This can be set to true in wasm32-unknown-unknown targets that
    /// are running on Windows systems.
    fn is_windows(&self) -> bool;
    /// Gets the current working directory.
    fn current_dir(&self) -> io::Result<PathBuf>;
    /// Gets the home directory of the current user.
    fn home_dir(&self) -> Option<PathBuf>;
    /// Splits a platform-specific PATH variable into a list of paths.
    fn env_split_paths(&self, paths: &OsStr) -> Vec<PathBuf>;
    /// Gets the value of an environment variable.
    fn env_var_os(&self, name: &OsStr) -> Option<OsString>;
    fn env_var(&self, key: &OsStr) -> Result<String, VarError> {
        match self.env_var_os(key) {
            Some(val) => val.into_string().map_err(VarError::NotUnicode),
            None => Err(VarError::NotPresent),
        }
    }
    /// Gets and parses the PATHEXT environment variable on Windows.
    ///
    /// Override this to enable caching the parsed PATHEXT.
    ///
    /// Note: This will only be called when `is_windows()` returns `true`
    /// and isn't conditionally compiled with `#[cfg(windows)]` so that it
    /// can work in Wasm.
    fn env_windows_path_ext(&self) -> Cow<'static, [String]> {
        Cow::Owned(
            self.env_var(OsStr::new("PATHEXT"))
                .map(|pathext| parse_path_ext(&pathext))
                // PATHEXT not being set or not being a proper Unicode string is exceedingly
                // improbable and would probably break Windows badly. Still, don't crash:
                .unwrap_or_default(),
        )
    }
    /// Gets the metadata of the provided path, following symlinks.
    fn metadata(&self, path: &Path) -> io::Result<Self::Metadata>;
    /// Gets the metadata of the provided path, not following symlinks.
    fn symlink_metadata(&self, path: &Path) -> io::Result<Self::Metadata>;
    /// Reads the directory entries of the provided path.
    fn read_dir(
        &self,
        path: &Path,
    ) -> io::Result<Box<dyn Iterator<Item = io::Result<Self::ReadDirEntry>>>>;
    /// Checks if the provided path is a valid executable.
    fn is_valid_executable(&self, path: &Path) -> io::Result<bool>;
}

impl SysReadDirEntry for std::fs::DirEntry {
    fn file_name(&self) -> OsString {
        self.file_name()
    }

    fn path(&self) -> PathBuf {
        self.path()
    }
}

impl SysMetadata for std::fs::Metadata {
    fn is_symlink(&self) -> bool {
        self.file_type().is_symlink()
    }

    fn is_file(&self) -> bool {
        self.file_type().is_file()
    }
}

#[cfg(feature = "real-sys")]
#[derive(Default, Clone)]
pub struct RealSys;

#[cfg(feature = "real-sys")]
impl RealSys {
    #[inline]
    pub(crate) fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        #[allow(clippy::disallowed_methods)] // ok, sys implementation
        std::fs::canonicalize(path)
    }
}

#[cfg(feature = "real-sys")]
impl Sys for RealSys {
    type ReadDirEntry = std::fs::DirEntry;
    type Metadata = std::fs::Metadata;

    #[inline]
    fn is_windows(&self) -> bool {
        // Again, do not change the code to directly use `#[cfg(windows)]`
        // because we want to allow people to implement this code in Wasm
        // and then tell at runtime if running on a Windows system.
        cfg!(windows)
    }

    #[inline]
    fn current_dir(&self) -> io::Result<PathBuf> {
        #[allow(clippy::disallowed_methods)] // ok, sys implementation
        std::env::current_dir()
    }

    #[inline]
    fn home_dir(&self) -> Option<PathBuf> {
        // Home dir shim, use env_home crate when possible. Otherwise, return None
        #[cfg(any(windows, unix, target_os = "redox"))]
        {
            env_home::env_home_dir()
        }
        #[cfg(not(any(windows, unix, target_os = "redox")))]
        {
            None
        }
    }

    #[inline]
    fn env_split_paths(&self, paths: &OsStr) -> Vec<PathBuf> {
        #[allow(clippy::disallowed_methods)] // ok, sys implementation
        std::env::split_paths(paths).collect()
    }

    fn env_windows_path_ext(&self) -> Cow<'static, [String]> {
        use std::sync::OnceLock;

        // Sample %PATHEXT%: .COM;.EXE;.BAT;.CMD;.VBS;.VBE;.JS;.JSE;.WSF;.WSH;.MSC
        // PATH_EXTENSIONS is then [".COM", ".EXE", ".BAT", …].
        // (In one use of PATH_EXTENSIONS we skip the dot, but in the other we need it;
        // hence its retention.)
        static PATH_EXTENSIONS: OnceLock<Vec<String>> = OnceLock::new();
        let path_extensions = PATH_EXTENSIONS.get_or_init(|| {
            self.env_var(OsStr::new("PATHEXT"))
                .map(|s| parse_path_ext(&s))
                // PATHEXT not being set or not being a proper Unicode string is exceedingly
                // improbable and would probably break Windows badly. Still, don't crash:
                .unwrap_or_default()
        });
        Cow::Borrowed(path_extensions)
    }

    #[inline]
    fn env_var_os(&self, name: &OsStr) -> Option<OsString> {
        #[allow(clippy::disallowed_methods)] // ok, sys implementation
        std::env::var_os(name)
    }

    #[inline]
    fn read_dir(
        &self,
        path: &Path,
    ) -> io::Result<Box<dyn Iterator<Item = io::Result<Self::ReadDirEntry>>>> {
        #[allow(clippy::disallowed_methods)] // ok, sys implementation
        let iter = std::fs::read_dir(path)?;
        Ok(Box::new(iter))
    }

    #[inline]
    fn metadata(&self, path: &Path) -> io::Result<Self::Metadata> {
        #[allow(clippy::disallowed_methods)] // ok, sys implementation
        std::fs::metadata(path)
    }

    #[inline]
    fn symlink_metadata(&self, path: &Path) -> io::Result<Self::Metadata> {
        #[allow(clippy::disallowed_methods)] // ok, sys implementation
        std::fs::symlink_metadata(path)
    }

    #[cfg(any(unix, target_os = "wasi", target_os = "redox"))]
    fn is_valid_executable(&self, path: &Path) -> io::Result<bool> {
        use rustix::fs as rfs;
        rfs::access(path, rfs::Access::EXEC_OK)
            .map(|_| true)
            .map_err(|e| io::Error::from_raw_os_error(e.raw_os_error()))
    }

    #[cfg(windows)]
    fn is_valid_executable(&self, path: &Path) -> io::Result<bool> {
        winsafe::GetBinaryType(&path.display().to_string())
            .map(|_| true)
            .map_err(|e| io::Error::from_raw_os_error(e.raw() as i32))
    }
}

fn parse_path_ext(pathext: &str) -> Vec<String> {
    pathext
        .split(';')
        .filter_map(|s| {
            if s.as_bytes().first() == Some(&b'.') {
                Some(s.to_owned())
            } else {
                // Invalid segment; just ignore it.
                None
            }
        })
        .collect()
}
