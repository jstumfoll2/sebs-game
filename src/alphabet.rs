//! Everything we know about each letter: what it's called, the sound it makes, and a word
//! (with a picture) that starts with that sound. Used by the Letters game, the menu title,
//! and the word labels in the other games.

use crate::pictures::Picture;

pub struct Letter {
    pub letter: char,
    /// How to say the letter's *name* so the voice gets it right ("bee").
    pub name: &'static str,
    /// The letter's *sound* in IPA phonetic symbols ("bə"), for the Piper voice.
    pub ipa: &'static str,
    /// The same sound spelled out ("buh"), for voices that can't read IPA.
    pub spelled: &'static str,
    pub word: &'static str,
    pub picture: Picture,
}

impl Letter {
    /// The letter's sound in the voice's markup, e.g. "{bə|buh}".
    pub fn sound(&self) -> String {
        format!("{{{}|{}}}", self.ipa, self.spelled)
    }

    /// "Bee says buh. Buh, buh, ball!"
    pub fn teach(&self) -> String {
        let s = self.sound();
        let extra = if self.letter == 'S' { " And Sebastian!" } else { "" };
        let word = if self.letter == 'X' { "like the end of box" } else { self.word };
        format!("{} says {s}. {s}, {s}, {word}!{extra}", capitalize(self.name))
    }
}

macro_rules! letter {
    ($l:literal, $name:literal, $ipa:literal, $spelled:literal, $word:literal, $pic:ident) => {
        Letter {
            letter: $l,
            name: $name,
            ipa: $ipa,
            spelled: $spelled,
            word: $word,
            picture: Picture::$pic,
        }
    };
}

pub const LETTERS: [Letter; 26] = [
    letter!('A', "ay", "ˈæ", "ah", "apple", Apple),
    letter!('B', "bee", "bə", "buh", "ball", Ball),
    letter!('C', "see", "kə", "kuh", "cat", Cat),
    letter!('D', "dee", "də", "duh", "dog", Dog),
    letter!('E', "ee", "ˈɛ", "eh", "egg", Egg),
    letter!('F', "eff", "fːː", "fff", "fish", Fish),
    letter!('G', "gee", "ɡə", "guh", "grapes", Grapes),
    letter!('H', "aitch", "hə", "huh", "heart", Heart),
    letter!('I', "eye", "ˈɪ", "ih", "igloo", Igloo),
    letter!('J', "jay", "dʒə", "juh", "jellyfish", Jellyfish),
    letter!('K', "kay", "kə", "kuh", "kite", Kite),
    letter!('L', "ell", "lːː", "lll", "lollipop", Lollipop),
    letter!('M', "em", "mːː", "mmm", "moon", Moon),
    letter!('N', "en", "nːː", "nnn", "nest", Nest),
    letter!('O', "oh", "ˈɑ", "ah", "octopus", Octopus),
    letter!('P', "pee", "pə", "puh", "pig", Pig),
    letter!('Q', "cue", "kwə", "kwuh", "queen", Crown),
    letter!('R', "ar", "ɹːː", "rrr", "rainbow", Rainbow),
    letter!('S', "ess", "sːː", "sss", "sun", Sun),
    letter!('T', "tee", "tə", "tuh", "tree", Tree),
    letter!('U', "you", "ˈʌ", "uh", "umbrella", Umbrella),
    letter!('V', "vee", "vːː", "vvv", "van", Van),
    letter!('W', "double you", "wə", "wuh", "whale", Whale),
    letter!('X', "ex", "ks", "ks", "box", Box),
    letter!('Y', "why", "jə", "yuh", "yo-yo", YoYo),
    letter!('Z', "zee", "zːː", "zzz", "zebra", Zebra),
];

/// Look up a letter (either case). Returns None for anything that isn't A-Z.
pub fn get(c: char) -> Option<&'static Letter> {
    let c = c.to_ascii_uppercase();
    c.is_ascii_uppercase().then(|| &LETTERS[(c as u8 - b'A') as usize])
}

pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_in_order() {
        for (i, l) in LETTERS.iter().enumerate() {
            assert_eq!(l.letter, (b'A' + i as u8) as char);
            assert!(l.word.to_uppercase().starts_with(l.letter) || l.letter == 'X' || l.letter == 'Q');
        }
        assert_eq!(get('b').unwrap().word, "ball");
        assert!(get('?').is_none());
    }
}
