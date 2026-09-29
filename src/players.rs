//! The kids who play, saved to a file so we only ask for a name once.
//!
//! The file lives at `%APPDATA%\sebastians-game\players.json` and looks like:
//! ```json
//! { "players": [ { "name": "Sebastian", "stars": 12, "levels": { "colors": 3 } } ],
//!   "last_player": 0 }
//! ```
//! A grown-up can edit it by hand (for example to fix a name or remove a player).
//!
//! `#[derive(Serialize, Deserialize)]` is the `serde` library writing the code that turns
//! these structs into JSON text and back.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Player {
    pub name: String,
    #[serde(default)]
    pub stars: u32,
    /// Each game's level, by game name ("colors", "counting", ...).
    #[serde(default)]
    pub levels: BTreeMap<String, u32>,
    /// Which levels of each game have been finished (ticked off in the level picker).
    #[serde(default)]
    pub completed: BTreeMap<String, Vec<u32>>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Players {
    #[serde(default)]
    pub players: Vec<Player>,
    /// Who played last, so they're picked automatically next time.
    #[serde(default)]
    pub last_player: Option<usize>,
    /// The voice picked in the voice menu (e.g. "en_US-amy-medium").
    #[serde(default)]
    pub voice: Option<String>,
}

/// Longest name we allow.
pub const MAX_NAME: usize = 14;

impl Players {
    /// Load the players file (or start empty if there isn't one yet, or it can't be read).
    pub fn load() -> Players {
        std::fs::read_to_string(path())
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let path = path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        match serde_json::to_string_pretty(self) {
            Ok(text) => {
                if let Err(e) = std::fs::write(&path, text) {
                    crate::log::line(&format!("couldn't save players to {}: {e}", path.display()));
                }
            }
            Err(e) => crate::log::line(&format!("couldn't save players: {e}")),
        }
    }

    /// Add a new player (or pick the existing one with the same name). Returns their number.
    pub fn add(&mut self, name: &str) -> usize {
        if let Some(i) = self.players.iter().position(|p| p.name.eq_ignore_ascii_case(name)) {
            return i;
        }
        self.players.push(Player { name: name.to_string(), ..Default::default() });
        self.players.len() - 1
    }
}

/// Where the players file is kept. `--players file.json` uses a different file (handy for testing).
pub fn path() -> PathBuf {
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--players") {
        if let Some(p) = args.get(i + 1) {
            return PathBuf::from(p);
        }
    }
    match std::env::var_os("APPDATA") {
        Some(appdata) => PathBuf::from(appdata).join("sebastians-game").join("players.json"),
        None => PathBuf::from("players.json"),
    }
}

/// Tidy up a typed name: trim spaces and capitalize each word ("sebastian" -> "Sebastian").
pub fn tidy_name(name: &str) -> String {
    name.split_whitespace()
        .map(crate::alphabet::capitalize)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_tidy() {
        let mut ps = Players::default();
        let i = ps.add("Sebastian");
        ps.players[i].stars = 5;
        ps.players[i].levels.insert("colors".into(), 3);
        assert_eq!(ps.add("sebastian"), i, "same name picks the same player");
        let text = serde_json::to_string(&ps).unwrap();
        let back: Players = serde_json::from_str(&text).unwrap();
        assert_eq!(back.players, ps.players);
        assert_eq!(tidy_name("  mary   jane "), "Mary Jane");
    }
}
