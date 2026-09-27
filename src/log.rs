//! A simple debug log: what happens (taps, what the voice is doing) goes to
//! `sebastians-game.log` in the folder you ran the game from. It starts fresh each launch.
//! Start with `--no-log` to turn it off.

use std::fs::File;
use std::io::Write;
use std::sync::{Mutex, OnceLock};

static LOG: OnceLock<Option<Mutex<File>>> = OnceLock::new();

fn file() -> &'static Option<Mutex<File>> {
    LOG.get_or_init(|| {
        let off = std::env::args().any(|a| a == "--no-log");
        if off {
            None
        } else {
            File::create("sebastians-game.log").ok().map(Mutex::new)
        }
    })
}

/// Write one line to the log (does nothing unless the game was started with `--log`).
pub fn line(text: &str) {
    if let Some(f) = file() {
        if let Ok(mut f) = f.lock() {
            let t = macroquad::time::get_time();
            let _ = writeln!(f, "{t:8.2}  {text}");
            let _ = f.flush();
        }
    }
}
