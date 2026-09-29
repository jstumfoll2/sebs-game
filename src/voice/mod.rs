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

/// The voice used when none has been picked yet.
pub const DEFAULT_VOICE: &str = "en_US-lessac-medium";

/// The Piper voices installed on this computer (e.g. "en_US-amy-medium").
pub fn installed() -> Vec<String> {
    piper::installed()
}

/// A friendly name for a voice: "en_US-amy-medium" -> "Amy", "en_US-hfc_female-medium" -> "Holly".
/// (Some voices are named after the recording they were made from, so they get a real name here.)
pub fn display_name(model: &str) -> String {
    let name = model.split('-').nth(1).unwrap_or(model);
    match name {
        "ljspeech" => "LJ".to_string(),
        "hfc_female" => "Holly".to_string(),
        other => crate::alphabet::capitalize(other),
    }
}

impl Voice {
    /// Start the chosen Piper voice (or the default, or any installed one). If Piper isn't
    /// installed at all, use the Windows voice.
    pub fn new(preferred: Option<&str>) -> Self {
        let mut choices: Vec<String> = preferred.into_iter().map(str::to_string).collect();
        choices.push(DEFAULT_VOICE.to_string());
        choices.extend(installed());
        for model in choices {
            if let Some(p) = piper::PiperVoice::start(&model) {
                return Voice::Piper(p);
            }
        }
        Voice::Sapi(sapi::SapiVoice::new())
    }

    /// Which Piper voice is speaking (None for the Windows voice).
    pub fn model(&self) -> Option<&str> {
        match self {
            Voice::Piper(p) => Some(p.model()),
            Voice::Sapi(_) => None,
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

    /// Turn the voice's sound off or on.
    pub fn set_muted(&mut self, muted: bool) {
        match self {
            Voice::Piper(p) => p.set_muted(muted),
            Voice::Sapi(s) => s.set_muted(muted),
        }
    }

    /// Is the voice waiting for Piper to make something before it can speak? (The first time
    /// a voice is used, Piper takes a few seconds to start.)
    pub fn waiting(&self) -> bool {
        match self {
            Voice::Piper(p) => p.waiting(),
            Voice::Sapi(_) => false,
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
