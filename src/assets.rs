//! Finds the `assets` folder, whether the game is started with `cargo run` (from the project
//! folder) or by double-clicking the .exe in target/release.

use std::path::PathBuf;

pub fn dir() -> PathBuf {
    let mut candidates = vec![PathBuf::from("assets")];
    if let Some(exe_dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())) {
        candidates.push(exe_dir.join("assets"));
        candidates.push(exe_dir.join("../../assets")); // target/release -> project folder
    }
    candidates
        .into_iter()
        .find(|p| p.is_dir())
        .unwrap_or_else(|| PathBuf::from("assets"))
}
