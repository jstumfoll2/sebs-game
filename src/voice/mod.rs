//! Talking! The game speaks every instruction, since Sebastian can't read yet.
//!
//! Two voices:
//! - **Piper** (`piper.rs`): a natural-sounding voice that runs offline. Used when it's
//!   installed (run `scripts/get-voice.ps1`).
//! - **Windows speech** (`sapi.rs`): robotic, but always there. The fallback.
//!
//! Text can include letter *sounds* in curly braces: `"{bə|buh}"` means "say the sound
//! written `bə` in IPA (phonetic symbols); a voice that can't do IPA says `buh` instead".
//! Example: `"B says {bə|buh}, like ball!"`

mod piper;
mod sapi;

/// One piece of something to say.
#[derive(Clone, Debug, PartialEq)]
pub enum Part {
    /// Ordinary words, like "Find the letter bee!"
    Words(String),
    /// A letter sound in IPA (e.g. "bə"), plus a normal spelling (e.g. "buh") as a backup.
    Sound { ipa: String, spelled: String },
}

/// Split `"B says {bə|buh}, like ball!"` into Words("B says"), Sound(bə), Words(", like ball!").
pub fn parse(text: &str) -> Vec<Part> {
    let mut parts = Vec::new();
    let push_words = |parts: &mut Vec<Part>, s: &str| {
        let s = s.trim();
        // Skip bits that are only punctuation, like the "," between two sounds.
        if s.chars().any(|c| c.is_alphanumeric()) {
            parts.push(Part::Words(s.to_string()));
        }
    };

    let mut rest = text;
    while let Some(start) = rest.find('{') {
        let Some(len) = rest[start..].find('}') else { break };
        push_words(&mut parts, &rest[..start]);
        let inner = &rest[start + 1..start + len];
        let (ipa, spelled) = inner.split_once('|').unwrap_or((inner, inner));
        parts.push(Part::Sound {
            ipa: ipa.to_string(),
            spelled: spelled.to_string(),
        });
        rest = &rest[start + len + 1..];
    }
    push_words(&mut parts, rest);
    parts
}

/// Plain-text version, with sounds replaced by their backup spelling.
fn flatten(text: &str) -> String {
    parse(text)
        .iter()
        .map(|p| match p {
            Part::Words(w) => w.as_str(),
            Part::Sound { spelled, .. } => spelled.as_str(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Whichever voice we ended up with. An `enum` lets one type hold either kind.
pub enum Voice {
    Piper(piper::PiperVoice),
    Sapi(sapi::SapiVoice),
}

impl Voice {
    pub fn new() -> Self {
        match piper::PiperVoice::start() {
            Some(p) => Voice::Piper(p),
            None => Voice::Sapi(sapi::SapiVoice::new()),
        }
    }

    /// Stop talking and say this right now.
    pub fn say(&mut self, text: &str) {
        match self {
            Voice::Piper(p) => p.speak(text, true),
            Voice::Sapi(s) => s.say(&flatten(text)),
        }
    }

    /// Say this after whatever is currently being said.
    pub fn then(&mut self, text: &str) {
        match self {
            Voice::Piper(p) => p.speak(text, false),
            Voice::Sapi(s) => s.then(&flatten(text)),
        }
    }

    /// Is the voice still talking (or about to)?
    pub fn busy(&self) -> bool {
        match self {
            Voice::Piper(p) => p.busy(),
            Voice::Sapi(s) => s.busy(),
        }
    }

    /// Get phrases ready ahead of time so they play instantly later.
    pub fn prepare(&mut self, texts: &[String]) {
        if let Voice::Piper(p) = self {
            for t in texts {
                p.prepare(t);
            }
        }
    }

    /// Call once per frame: starts the next clip when the last one ends.
    pub async fn update(&mut self) {
        if let Voice::Piper(p) = self {
            p.update().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sounds_between_words() {
        assert_eq!(
            parse("B says {bə|buh}, {bə|buh}, ball!"),
            vec![
                Part::Words("B says".into()),
                Part::Sound { ipa: "bə".into(), spelled: "buh".into() },
                Part::Sound { ipa: "bə".into(), spelled: "buh".into() },
                Part::Words(", ball!".into()),
            ]
        );
        assert_eq!(flatten("Find {bə|buh}!"), "Find buh");
    }
}
