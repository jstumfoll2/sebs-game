//! A simple debug log: what happens (taps, what the voice is doing) goes to
//! `star-catchers.log` in the folder you ran the game from. The previous run's log is kept
//! as `star-catchers.prev.log` (handy after a crash). Start with `--no-log` to turn it off.
//!
//! It also watches for trouble:
//! - if the game crashes with an error message, the message is written to the log;
//! - a "watchdog" thread notices if the game stops drawing frames for a few seconds (a freeze)
//!   and logs which step it was stuck in.

use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

static LOG: OnceLock<Option<Mutex<File>>> = OnceLock::new();
static START: OnceLock<Instant> = OnceLock::new();
/// When the main loop last finished a frame (milliseconds since start).
static LAST_FRAME_MS: AtomicU64 = AtomicU64::new(0);
/// What the main loop is doing right now ("update", "draw", ...).
static STEP: Mutex<&'static str> = Mutex::new("starting");

/// How long without a frame counts as a freeze.
const STALL_MS: u64 = 3000;

fn now_ms() -> u64 {
    START.get_or_init(Instant::now).elapsed().as_millis() as u64
}

fn file() -> &'static Option<Mutex<File>> {
    LOG.get_or_init(|| {
        if std::env::args().any(|a| a == "--no-log") {
            return None;
        }
        let _ = std::fs::rename("star-catchers.log", "star-catchers.prev.log");
        File::create("star-catchers.log").ok().map(Mutex::new)
    })
}

/// Write one line to the log. Safe to call from any thread.
pub fn line(text: &str) {
    if let Some(f) = file() {
        if let Ok(mut f) = f.lock() {
            let t = now_ms() as f64 / 1000.0;
            let _ = writeln!(f, "{t:8.2}  {text}");
            let _ = f.flush();
        }
    }
}

/// Call once at startup: records crashes and starts the freeze watchdog.
pub fn start() {
    START.get_or_init(Instant::now);
    frame_done();
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        line(&format!("CRASH: {info}"));
        default_hook(info);
    }));
    std::thread::spawn(|| {
        let mut stalled = false;
        loop {
            std::thread::sleep(Duration::from_millis(500));
            let since = now_ms().saturating_sub(LAST_FRAME_MS.load(Ordering::Relaxed));
            let step = STEP.lock().map(|s| *s).unwrap_or("?");
            if since > STALL_MS && !stalled {
                stalled = true;
                line(&format!("FREEZE: no new frame for {:.1}s, stuck in: {step}", since as f64 / 1000.0));
            } else if since <= STALL_MS && stalled {
                stalled = false;
                line(&format!("recovered from the freeze (was stuck in: {step})"));
            }
        }
    });
}

/// Note what the main loop is about to do (so a freeze can say where it happened).
pub fn step(name: &'static str) {
    if let Ok(mut s) = STEP.lock() {
        *s = name;
    }
}

/// Note that a frame finished.
pub fn frame_done() {
    LAST_FRAME_MS.store(now_ms(), Ordering::Relaxed);
}
