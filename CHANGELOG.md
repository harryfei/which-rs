# CHANGELOG

## 5.0.0

- Remove several unused error messages
- Windows executables can now be found even if they don't have a '.exe' extension.
- Add new error message, `Error::CannotGetCurrentDirAndPathListEmpty`

## 4.4.2

- Remove dependency on `dirs` crate due to MPL licensing in its tree. Use `home` crate instead. (@Xaeroxe)

## 4.4.1

- Add tilde expansion for home directory (@Xaeroxe)
- Swap out libc for rustix, forbid unsafe (@notgull)
