use crate::checker::CompositeChecker;
use crate::helper::has_executable_extension;
use crate::sys::Sys;
use crate::sys::SysReadDirEntry;
use crate::{error::*, NonFatalErrorHandler};
use either::Either;
#[cfg(feature = "regex")]
use regex::Regex;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::iter;
use std::path::{Component, Path, PathBuf};

pub trait Checker {
    fn is_valid<F: NonFatalErrorHandler>(
        &self,
        path: &Path,
        nonfatal_error_handler: &mut F,
    ) -> bool;
}

trait PathExt {
    fn has_separator(&self) -> bool;

    fn to_absolute<P>(self, cwd: P) -> PathBuf
    where
        P: AsRef<Path>;
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
            new_path.extend(
                self.components()
                    .skip_while(|c| matches!(c, Component::CurDir)),
            );
            new_path
        }
    }
}

pub struct Finder<TSys: Sys> {
    sys: TSys,
}

impl<TSys: Sys + 'static> Finder<TSys> {
    pub fn new(sys: TSys) -> Self {
        Finder { sys }
    }

    pub fn find<'a, T, U, V, F: NonFatalErrorHandler + 'a>(
        &self,
        binary_name: T,
        paths: Option<U>,
        cwd: Option<V>,
        binary_checker: CompositeChecker<TSys>,
        mut nonfatal_error_handler: F,
    ) -> Result<impl Iterator<Item = PathBuf> + 'a>
    where
        T: AsRef<OsStr>,
        U: AsRef<OsStr>,
        V: AsRef<Path> + 'a,
    {
        let path = PathBuf::from(&binary_name);

        #[cfg(feature = "tracing")]
        tracing::debug!(
            "query binary_name = {:?}, paths = {:?}, cwd = {:?}",
            binary_name.as_ref().to_string_lossy(),
            paths.as_ref().map(|p| p.as_ref().to_string_lossy()),
            cwd.as_ref().map(|p| p.as_ref().display())
        );

        let binary_path_candidates = match cwd {
            Some(cwd) if path.has_separator() => {
                #[cfg(feature = "tracing")]
                tracing::trace!(
                    "{} has a path seperator, so only CWD will be searched.",
                    path.display()
                );
                // Search binary in cwd if the path have a path separator.
                Either::Left(Self::cwd_search_candidates(&self.sys, path, cwd))
            }
            _ => {
                #[cfg(feature = "tracing")]
                tracing::trace!("{} has no path seperators, so only paths in PATH environment variable will be searched.", path.display());
                // Search binary in PATHs(defined in environment variable).
                let paths = paths.ok_or(Error::CannotGetCurrentDirAndPathListEmpty)?;
                let paths = self.sys.env_split_paths(paths.as_ref());
                if paths.is_empty() {
                    return Err(Error::CannotGetCurrentDirAndPathListEmpty);
                }

                Either::Right(Self::path_search_candidates(
                    &self.sys,
                    path,
                    paths.into_iter(),
                ))
            }
        };
        let sys = self.sys.clone();
        let ret = binary_path_candidates.filter_map(move |p| {
            binary_checker
                .is_valid(&p, &mut nonfatal_error_handler)
                .then(|| correct_casing(&sys, p, &mut nonfatal_error_handler))
        });
        #[cfg(feature = "tracing")]
        let ret = ret.inspect(|p| {
            tracing::debug!("found path {}", p.display());
        });
        Ok(ret)
    }

    #[cfg(feature = "regex")]
    pub fn find_re<T, F: NonFatalErrorHandler>(
        &self,
        binary_regex: impl std::borrow::Borrow<Regex>,
        paths: Option<T>,
        binary_checker: CompositeChecker<TSys>,
        mut nonfatal_error_handler: F,
    ) -> Result<impl Iterator<Item = PathBuf>>
    where
        T: AsRef<OsStr>,
    {
        let p = paths.ok_or(Error::CannotGetCurrentDirAndPathListEmpty)?;
        let paths = self.sys.env_split_paths(p.as_ref());

        let sys = self.sys.clone();
        let matching_re = paths
            .into_iter()
            .flat_map(move |p| sys.read_dir(&p))
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(move |p| {
                if let Some(unicode_file_name) = p.file_name().unwrap().to_str() {
                    binary_regex.borrow().is_match(unicode_file_name)
                } else {
                    false
                }
            })
            .filter(move |p| binary_checker.is_valid(p, &mut nonfatal_error_handler));

        Ok(matching_re)
    }

    fn cwd_search_candidates<C>(
        sys: &TSys,
        binary_name: PathBuf,
        cwd: C,
    ) -> impl Iterator<Item = PathBuf>
    where
        C: AsRef<Path>,
    {
        let path = binary_name.to_absolute(cwd);

        Self::append_extension(sys, iter::once(path))
    }

    fn path_search_candidates<P>(
        sys: &TSys,
        binary_name: PathBuf,
        paths: P,
    ) -> impl Iterator<Item = PathBuf>
    where
        P: Iterator<Item = PathBuf>,
    {
        let new_paths = paths.map({
            let sys = sys.clone();
            move |p| tilde_expansion(&sys, &p).join(binary_name.clone())
        });

        Self::append_extension(sys, new_paths)
    }

    fn append_extension<P>(sys: &TSys, paths: P) -> impl Iterator<Item = PathBuf>
    where
        P: Iterator<Item = PathBuf>,
    {
        struct PathsIter<P>
        where
            P: Iterator<Item = PathBuf>,
        {
            paths: P,
            current_path_with_index: Option<(PathBuf, usize)>,
            path_extensions: Cow<'static, [String]>,
        }

        impl<P> Iterator for PathsIter<P>
        where
            P: Iterator<Item = PathBuf>,
        {
            type Item = PathBuf;

            fn next(&mut self) -> Option<Self::Item> {
                if self.path_extensions.is_empty() {
                    self.paths.next()
                } else if let Some((p, index)) = self.current_path_with_index.take() {
                    let next_index = index + 1;
                    if next_index < self.path_extensions.len() {
                        self.current_path_with_index = Some((p.clone(), next_index));
                    }
                    // Append the extension.
                    let mut p = p.into_os_string();
                    p.push(&self.path_extensions[index]);
                    let ret = PathBuf::from(p);
                    #[cfg(feature = "tracing")]
                    tracing::trace!("possible extension: {}", ret.display());
                    Some(ret)
                } else {
                    let p = self.paths.next()?;
                    if has_executable_extension(&p, &self.path_extensions) {
                        #[cfg(feature = "tracing")]
                        tracing::trace!(
                            "{} already has an executable extension, not modifying it further",
                            p.display()
                        );
                    } else {
                        #[cfg(feature = "tracing")]
                        tracing::trace!(
                            "{} has no extension, using PATHEXT environment variable to infer one",
                            p.display()
                        );
                        // Appended paths with windows executable extensions.
                        // e.g. path `c:/windows/bin[.ext]` will expand to:
                        // [c:/windows/bin.ext]
                        // c:/windows/bin[.ext].COM
                        // c:/windows/bin[.ext].EXE
                        // c:/windows/bin[.ext].CMD
                        // ...
                        self.current_path_with_index = Some((p.clone(), 0));
                    }
                    Some(p)
                }
            }
        }

        let path_extensions = if sys.is_windows() {
            sys.env_windows_path_ext()
        } else {
            Cow::Borrowed(Default::default())
        };

        PathsIter {
            paths,
            current_path_with_index: None,
            path_extensions,
        }
    }
}

fn tilde_expansion<'a, TSys: Sys>(sys: &TSys, p: &'a Path) -> Cow<'a, Path> {
    let mut component_iter = p.components();
    if let Some(Component::Normal(o)) = component_iter.next() {
        if o == "~" {
            let new_path = sys.home_dir();
            if let Some(mut new_path) = new_path {
                new_path.extend(component_iter);
                #[cfg(feature = "tracing")]
                tracing::trace!(
                    "found tilde, substituting in user's home directory to get {}",
                    new_path.display()
                );
                return Cow::Owned(new_path);
            } else {
                #[cfg(feature = "tracing")]
                tracing::trace!("found tilde in path, but user's home directory couldn't be found");
            }
        }
    }
    Cow::Borrowed(p)
}

fn correct_casing<TSys: Sys, F: NonFatalErrorHandler>(
    sys: &TSys,
    mut p: PathBuf,
    nonfatal_error_handler: &mut F,
) -> PathBuf {
    if sys.is_windows() {
        if let (Some(parent), Some(file_name)) = (p.parent(), p.file_name()) {
            if let Ok(iter) = sys.read_dir(parent) {
                for e in iter {
                    match e {
                        Ok(e) => {
                            if e.file_name().eq_ignore_ascii_case(file_name) {
                                p.pop();
                                p.push(e.file_name());
                                break;
                            }
                        }
                        Err(e) => {
                            nonfatal_error_handler.handle(NonFatalError::Io(e));
                        }
                    }
                }
            }
        }
    }
    p
}
