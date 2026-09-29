//! The Piper voice: natural-sounding speech made on this computer, no internet needed.
//!
//! How it works:
//! 1. Every phrase becomes a `.wav` clip in `assets/voice-cache/<voice>/`, named after its
//!    words (e.g. `pick-a-game.wav`). Clips are saved, so each phrase is only made once.
//!    You can even replace a clip with your own recording of the same name!
//! 2. All the slow work happens on a background *thread*, so the game never stutters:
//!    making missing clips with Piper, then reading and preparing them to play. Piper runs
//!    as two long-lived processes (at low priority, so drawing always comes first): one
//!    reads English text, the other reads phonetic symbols (IPA) for letter sounds like
//!    "sss" that English spelling can't express.
//! 3. The game plays the clips one after another. Because we know how long each clip is,
//!    we always know exactly when the voice has finished.

use super::{parse, Part};
use crate::wav;
use macroquad::audio::{
    load_sound_from_bytes, play_sound, set_sound_volume, stop_sound, PlaySoundParams, Sound,
};
use macroquad::time::get_time;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};

/// A request for one clip: made by Piper if it isn't saved yet, then prepared to play.
struct Job {
    name: String,
    text: String,
    /// True if `text` is IPA phonetic symbols rather than English.
    phonetic: bool,
}

/// A clip, ready to hand to the sound system: WAV bytes at 44.1 kHz, and its length.
type Prepared = (Vec<u8>, f32);

pub struct PiperVoice {
    /// Which voice this is (e.g. "en_US-amy-medium").
    model: String,
    /// Send jobs to the background thread...
    jobs: Sender<Job>,
    /// ...and hear back with the prepared clip (or None if it couldn't be made).
    done: Receiver<(String, Option<Prepared>)>,
    /// Clips we've asked for but that aren't ready yet.
    making: HashSet<String>,
    /// Clips that couldn't be made or loaded (skipped instead of waiting forever).
    broken: HashSet<String>,
    /// Loaded clips, ready to play, with their length in seconds.
    loaded: HashMap<String, (Sound, f32)>,
    /// What to say next, in order.
    queue: VecDeque<Job>,
    playing: Option<Sound>,
    playing_until: f64,
    /// Muted clips still "play" (silently), so games keep waiting for the voice as usual.
    muted: bool,
}

/// The installed voices, by name (the files in `assets/piper/voices`).
pub fn installed() -> Vec<String> {
    let dir = crate::assets::dir().join("piper").join("voices");
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().to_str().map(str::to_string))
        .filter_map(|f| f.strip_suffix(".onnx").map(str::to_string))
        .collect();
    names.sort();
    names
}

impl PiperVoice {
    /// Start the voice called `model` (e.g. "en_US-amy-medium").
    /// Returns None if it isn't installed, so the caller can pick another or fall back.
    pub fn start(model: &str) -> Option<Self> {
        let assets = crate::assets::dir();
        let piper = assets.join("piper");
        let exe = piper.join("piper").join(exe_name());
        let model_file = piper.join("voices").join(format!("{model}.onnx"));
        let config = piper.join("voices").join(format!("{model}.onnx.json"));
        if !exe.exists() || !model_file.exists() || !config.exists() {
            return None;
        }
        // Each voice keeps its own clips, so switching voices doesn't mix them up.
        let cache = assets.join("voice-cache").join(model);
        let tmp = cache.join("tmp");
        std::fs::create_dir_all(&tmp).ok()?;

        // Piper normally turns English into sounds itself. A copy of the voice settings with
        // `phoneme_type` set to "text" makes it take phonetic symbols directly instead.
        let phonetic_config = cache.join("phonetic.onnx.json");
        let settings = std::fs::read_to_string(&config).ok()?;
        let phonetic = settings.replacen("\"phoneme_type\": \"espeak\"", "\"phoneme_type\": \"text\"", 1);
        std::fs::write(&phonetic_config, phonetic).ok()?;

        let (jobs, job_rx) = channel::<Job>();
        let (done_tx, done) = channel::<(String, Option<Prepared>)>();
        // `move` gives the thread its own copies of these values.
        std::thread::spawn(move || {
            // Piper starts on first use, so a voice that's never used costs nothing.
            let mut english: Option<Piper> = None;
            let mut phonetic: Option<Piper> = None;
            for job in job_rx {
                let target = cache.join(format!("{}.wav", job.name));
                if !target.exists() {
                    let slot = if job.phonetic { &mut phonetic } else { &mut english };
                    if slot.is_none() {
                        let cfg = if job.phonetic { &phonetic_config } else { &config };
                        *slot = Piper::spawn(&exe, &model_file, cfg, &tmp);
                    }
                    if let Some(made) = slot.as_mut().and_then(|p| p.make(&job.text)) {
                        // Piper reports the file a moment before it closes it, and Windows
                        // won't move an open file, so try a few times.
                        for _ in 0..50 {
                            if std::fs::rename(&made, &target).is_ok() {
                                break;
                            }
                            std::thread::sleep(std::time::Duration::from_millis(20));
                        }
                    }
                }
                if done_tx.send((job.name, prepare(&target))).is_err() {
                    break; // the game has closed (or switched voices)
                }
            }
        });

        Some(PiperVoice {
            model: model.to_string(),
            jobs,
            done,
            making: HashSet::new(),
            broken: HashSet::new(),
            loaded: HashMap::new(),
            queue: VecDeque::new(),
            playing: None,
            playing_until: 0.0,
            muted: false,
        })
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn speak(&mut self, text: &str, interrupt: bool) {
        crate::log::line(&format!("voice {}: {text}", if interrupt { "say" } else { "then" }));
        if interrupt {
            if let Some(s) = self.playing.take() {
                stop_sound(&s);
            }
            self.playing_until = 0.0;
            self.queue.clear();
        }
        self.queue.extend(parse(text).into_iter().map(job_for));
        // Ask for it right away, so it gets made before anything that was only `prepare`d.
        self.request_upcoming();
    }

    /// Is the voice waiting for a clip to be made before it can say the next thing?
    pub fn waiting(&self) -> bool {
        get_time() >= self.playing_until
            && self.queue.front().is_some_and(|j| !self.loaded.contains_key(&j.name) && !self.broken.contains(&j.name))
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        if let Some(s) = &self.playing {
            set_sound_volume(s, if muted { 0.0 } else { 1.0 });
        }
    }

    pub fn busy(&self) -> bool {
        !self.queue.is_empty() || get_time() < self.playing_until
    }

    /// Start getting clips for this text ready now, without saying it.
    pub fn prepare(&mut self, text: &str) {
        for job in parse(text).into_iter().map(job_for) {
            self.request(job);
        }
    }

    pub async fn update(&mut self) {
        // Hear back from the background thread: hand ready clips to the sound system.
        while let Ok((name, prepared)) = self.done.try_recv() {
            self.making.remove(&name);
            let sound = match &prepared {
                Some((bytes, _)) => load_sound_from_bytes(bytes).await.ok(),
                None => None,
            };
            match (sound, prepared) {
                (Some(sound), Some((_, secs))) => {
                    self.loaded.insert(name, (sound, secs));
                }
                _ => {
                    self.broken.insert(name);
                }
            }
        }

        self.request_upcoming();

        // When the current clip ends, play the next one (if it's ready).
        if get_time() >= self.playing_until {
            self.playing = None;
            while let Some(next) = self.queue.front() {
                if self.broken.contains(&next.name) {
                    self.queue.pop_front(); // skip it rather than get stuck
                    continue;
                }
                if let Some((sound, secs)) = self.loaded.get(&next.name) {
                    crate::log::line(&format!("voice plays {} ({secs:.1}s)", next.name));
                    let volume = if self.muted { 0.0 } else { 1.0 };
                    play_sound(sound, PlaySoundParams { looped: false, volume });
                    self.playing = Some(sound.clone());
                    self.playing_until = get_time() + *secs as f64;
                    self.queue.pop_front();
                }
                break;
            }
        }
    }

    /// Make sure the next few clips in the queue are on their way.
    fn request_upcoming(&mut self) {
        let upcoming: Vec<Job> = self
            .queue
            .iter()
            .take(4)
            .map(|j| Job { name: j.name.clone(), text: j.text.clone(), phonetic: j.phonetic })
            .collect();
        for job in upcoming {
            self.request(job);
        }
    }

    /// Ask the background thread for a clip (unless it's ready, coming, or broken).
    fn request(&mut self, job: Job) {
        let known = self.loaded.contains_key(&job.name) || self.broken.contains(&job.name);
        if !known && !self.making.contains(&job.name) {
            self.making.insert(job.name.clone());
            let _ = self.jobs.send(job);
        }
    }
}

/// Read a saved clip and convert it to 44.1 kHz (what the sound system plays), with smooth
/// resampling. Runs on the background thread.
fn prepare(path: &Path) -> Option<Prepared> {
    let audio = wav::decode(&std::fs::read(path).ok()?)?.resample(44_100);
    Some((wav::encode(&audio), audio.seconds()))
}

/// Turn a piece of speech into a clip request with a readable file name.
fn job_for(part: Part) -> Job {
    match part {
        Part::Words(text) => Job { name: slug(&text), text, phonetic: false },
        Part::Sound { ipa, spelled } => Job {
            // Name sounds by their spelling plus a short code, e.g. "sound-buh-1a2b".
            name: format!("sound-{}-{:04x}", slug(&spelled), short_hash(&ipa)),
            text: ipa,
            phonetic: true,
        },
    }
}

/// "Where does the red ball go?" -> "where-does-the-red-ball-go"
fn slug(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    let out = out.trim_end_matches('-').to_string();
    if out.len() > 80 {
        // Very long phrases: keep it short and add a code so names stay unique.
        format!("{}-{:04x}", &out[..70], short_hash(text))
    } else if out.is_empty() {
        format!("clip-{:04x}", short_hash(text))
    } else {
        out
    }
}

/// A small, stable fingerprint of some text (FNV-1a hash, folded to 16 bits).
fn short_hash(text: &str) -> u16 {
    let mut h: u32 = 0x811c9dc5;
    for b in text.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(0x01000193);
    }
    (h ^ (h >> 16)) as u16
}

fn exe_name() -> &'static str {
    if cfg!(windows) {
        "piper.exe"
    } else {
        "piper"
    }
}

/// One running Piper program. We type a line in, it writes a .wav and tells us where.
struct Piper {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Piper {
    fn spawn(exe: &Path, model: &Path, config: &Path, out_dir: &Path) -> Option<Piper> {
        let mut cmd = Command::new(exe);
        cmd.arg("--model").arg(model)
            .arg("--config").arg(config)
            .arg("--output_dir").arg(out_dir)
            .args(["--quiet", "--length_scale", "1.1", "--sentence_silence", "0.15"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // No console window, and below-normal priority so the game's drawing always goes first.
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            const BELOW_NORMAL_PRIORITY_CLASS: u32 = 0x0000_4000;
            cmd.creation_flags(CREATE_NO_WINDOW | BELOW_NORMAL_PRIORITY_CLASS);
        }
        let mut child = cmd.spawn().ok()?;
        let stdin = child.stdin.take()?;
        let stdout = BufReader::new(child.stdout.take()?);
        Some(Piper { child, stdin, stdout })
    }

    /// Make one clip. Returns the path of the new .wav file.
    fn make(&mut self, text: &str) -> Option<PathBuf> {
        writeln!(self.stdin, "{}", text.replace(['\n', '\r'], " ")).ok()?;
        self.stdin.flush().ok()?;
        let mut line = String::new();
        self.stdout.read_line(&mut line).ok()?;
        let path = PathBuf::from(line.trim());
        path.exists().then_some(path)
    }
}

impl Drop for Piper {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}
