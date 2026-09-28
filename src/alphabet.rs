//! Everything we know about each letter: what it's called, the sound it makes, and a word
//! (with a picture) that starts with that sound. Used by the Letters game, the menu title,
//! and the word labels in the other games.

use crate::pictures::Picture;

pub struct Letter {
    pub letter: char,
    /// How the voice says the letter's *name*. Plain capitals ("B") are read right; "A" needs
    /// its sound spelled out in IPA, or at the start of a sentence it's read as the word "a".
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

    /// "B says buh. Buh, buh, ball!" If the player's name starts with this letter, it's
    /// mentioned too: "...And Sebastian!"
    pub fn teach(&self, player: &str) -> String {
        let s = self.sound();
        let starts_name = player.chars().next().map(|c| c.to_ascii_uppercase()) == Some(self.letter);
        let extra = if starts_name { format!(" And {player}!") } else { String::new() };
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
    letter!('A', "{ˈeɪ|A}", "ˈæ", "ah", "apple", Apple),
    letter!('B', "B", "bə", "buh", "ball", Ball),
    letter!('C', "C", "kə", "kuh", "cat", Cat),
    letter!('D', "D", "də", "duh", "dog", Dog),
    letter!('E', "E", "ˈɛ", "eh", "egg", Egg),
    letter!('F', "F", "fːː", "fff", "fish", Fish),
    letter!('G', "G", "ɡə", "guh", "grapes", Grapes),
    letter!('H', "H", "hə", "huh", "heart", Heart),
    letter!('I', "I", "ˈɪ", "ih", "igloo", Igloo),
    letter!('J', "J", "dʒə", "juh", "jellyfish", Jellyfish),
    letter!('K', "K", "kə", "kuh", "kite", Kite),
    letter!('L', "L", "lːː", "lll", "lollipop", Lollipop),
    letter!('M', "M", "mːː", "mmm", "moon", Moon),
    letter!('N', "N", "nːː", "nnn", "nest", Nest),
    letter!('O', "O", "ˈɑ", "ah", "octopus", Octopus),
    letter!('P', "P", "pə", "puh", "pig", Pig),
    letter!('Q', "Q", "kwə", "kwuh", "queen", Crown),
    letter!('R', "R", "ɹːː", "rrr", "rainbow", Rainbow),
    letter!('S', "S", "sːː", "sss", "sun", Sun),
    letter!('T', "T", "tə", "tuh", "tree", Tree),
    letter!('U', "U", "ˈʌ", "uh", "umbrella", Umbrella),
    letter!('V', "V", "vːː", "vvv", "van", Van),
    letter!('W', "W", "wə", "wuh", "whale", Whale),
    letter!('X', "X", "ks", "ks", "box", Box),
    letter!('Y', "Y", "jə", "yuh", "yo-yo", YoYo),
    letter!('Z', "Z", "zːː", "zzz", "zebra", Zebra),
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
