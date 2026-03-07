fn main() {
    windows_bindgen::bindgen([
        "--out",
        concat!(env!("CARGO_MANIFEST_DIR"), "/../which/src/win_ffi.rs"),
        "--flat",
        "--sys",
        "--no-deps",
        "--filter",
        "Windows.Win32.Storage.FileSystem.GetBinaryTypeW",
        "--filter",
        "Windows.Win32.Foundation.GetLastError",
    ])
    .unwrap();
}
