# CHANGELOG

## 6.0.1

- Remove dependency on `once_cell` for Windows users, replace with `std::sync::OnceLock`.

## 6.0.0

- MSRV is now 1.70
- Upgraded all dependencies to latest version

## 5.0.0

- Remove several unused error messages
- Windows executables can now be found even if they don't have a '.exe' extension.
- Add new error message, `Error::CannotGetCurrentDirAndPathListEmpty`

## 4.4.2

- Remove dependency on `dirs` crate due to MPL licensing in its tree. Use `home` crate instead. (@Xaeroxe)

## 4.4.1

- Add tilde expansion for home directory (@Xaeroxe)
- Swap out libc for rustix, forbid unsafe (@notgull)
