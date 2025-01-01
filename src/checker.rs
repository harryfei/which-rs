use crate::finder::Checker;
use crate::sys::Sys;
use crate::sys::SysMetadata;
use crate::{NonFatalError, NonFatalErrorHandler};
use std::path::Path;

pub struct ExecutableChecker<TSys: Sys> {
    sys: TSys,
}

impl<TSys: Sys> ExecutableChecker<TSys> {
    pub fn new(sys: TSys) -> Self {
        Self { sys }
    }
}

impl<TSys: Sys> Checker for ExecutableChecker<TSys> {
    fn is_valid<F: NonFatalErrorHandler>(
        &self,
        path: &Path,
        nonfatal_error_handler: &mut F,
    ) -> bool {
        if self.sys.is_windows() && path.extension().is_some() {
            true
        } else {
            let ret = self
                .sys
                .is_valid_executable(path)
                .map_err(|e| nonfatal_error_handler.handle(NonFatalError::Io(e)))
                .unwrap_or(false);
            #[cfg(feature = "tracing")]
            tracing::trace!("{} EXEC_OK = {ret}", path.display());
            ret
        }
    }
}

pub struct ExistedChecker<TSys: Sys> {
    sys: TSys,
}

impl<TSys: Sys> ExistedChecker<TSys> {
    pub fn new(sys: TSys) -> Self {
        Self { sys }
    }
}

impl<TSys: Sys> Checker for ExistedChecker<TSys> {
    fn is_valid<F: NonFatalErrorHandler>(
        &self,
        path: &Path,
        nonfatal_error_handler: &mut F,
    ) -> bool {
        if self.sys.is_windows() {
            let ret = self
                .sys
                .symlink_metadata(path)
                .map(|metadata| {
                    #[cfg(feature = "tracing")]
                    tracing::trace!(
                        "{} is_file() = {}, is_symlink() = {}",
                        path.display(),
                        metadata.is_file(),
                        metadata.is_symlink()
                    );
                    metadata.is_file() || metadata.is_symlink()
                })
                .map_err(|e| {
                    nonfatal_error_handler.handle(NonFatalError::Io(e));
                })
                .unwrap_or(false);
            #[cfg(feature = "tracing")]
            tracing::trace!(
                "{} has_extension = {}, ExistedChecker::is_valid() = {ret}",
                path.display(),
                path.extension().is_some()
            );
            ret
        } else {
            let ret = self.sys.metadata(path).map(|metadata| metadata.is_file());
            #[cfg(feature = "tracing")]
            tracing::trace!("{} is_file() = {ret:?}", path.display());
            match ret {
                Ok(ret) => ret,
                Err(e) => {
                    nonfatal_error_handler.handle(NonFatalError::Io(e));
                    false
                }
            }
        }
    }
}

pub struct CompositeChecker<TSys: Sys> {
    existed_checker: ExistedChecker<TSys>,
    executable_checker: ExecutableChecker<TSys>,
}

impl<TSys: Sys> CompositeChecker<TSys> {
    pub fn new(sys: TSys) -> Self {
        CompositeChecker {
            executable_checker: ExecutableChecker::new(sys.clone()),
            existed_checker: ExistedChecker::new(sys),
        }
    }
}

impl<TSys: Sys> Checker for CompositeChecker<TSys> {
    fn is_valid<F: NonFatalErrorHandler>(
        &self,
        path: &Path,
        nonfatal_error_handler: &mut F,
    ) -> bool {
        self.existed_checker.is_valid(path, nonfatal_error_handler)
            && self
                .executable_checker
                .is_valid(path, nonfatal_error_handler)
    }
}
