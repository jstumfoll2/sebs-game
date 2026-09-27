//! The Piper voice: natural-sounding speech made on this computer, no internet needed.
//!
//! How it works:
//! 1. Every phrase becomes a `.wav` clip in `assets/voice-cache/`, named after its words
//!    (e.g. `where-does-the-red-ball-go.wav`). Clips are saved, so each phrase is only
//!    made once. You can even replace a clip with your own recording of the same name!
//! 2. Missing clips are made by Piper on a background *thread*, so the game never freezes.
//!    Piper runs as two long-lived processes: one reads English text, the other reads
//!    phonetic symbols (IPA) for letter sounds like "sss" that English spelling can't express.
//! 3. The game plays the clips one after another. Because we know how long each clip is,
//!    we always know exactly when the voice has finished.

use super::{parse, Part};
use crate::wav;
use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};
use macroquad::time::get_time;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};

/// A request for Piper to make one clip.
struct Job {
    name: String,
    text: String,
    /// True if `text` is IPA phonetic symbols rather than English.
    phonetic: bool,
}

pub struct PiperVoice {
    cache: PathBuf,
    /// Send jobs to the background thread...
    jobs: Sender<Job>,
    /// ...and hear back (by clip name) when each one is done.
    done: Receiver<String>,
    /// Clips we've asked for but that aren't finished yet.
    making: HashSet<String>,
    /// Clips that couldn't be made or loaded (skipped instead of waiting forever).
    broken: HashSet<String>,
    /// Loaded clips, ready to play, with their length in seconds.
    loaded: HashMap<String, (Sound, f32)>,
    /// What to say next, in order.
    queue: VecDeque<Job>,
    playing: Option<Sound>,
    playing_until: f64,
}

impl PiperVoice {
    /// Returns None if Piper isn't installed, so the caller can fall back to Windows speech.
    pub fn start() -> Option<Self> {
        let assets = crate::assets::dir();
        let piper = assets.join("piper");
        let exe = piper.join("piper").join(exe_name());
        let model = piper.join("voice.onnx");
        let config = piper.join("voice.onnx.json");
        if !exe.exists() || !model.exists() || !config.exists() {
            return None;
        }
        let cache = assets.join("voice-cache");
        std::fs::create_dir_all(cache.join("tmp")).ok()?;

        // Piper normally turns English into sounds itself. A copy of the voice settings with
        // `phoneme_type` set to "text" makes it take phonetic symbols directly instead.
        let phonetic_config = piper.join("voice-phonetic.onnx.json");
        let settings = std::fs::read_to_string(&config).ok()?;
        let phonetic = settings.replacen("\"phoneme_type\": \"espeak\"", "\"phoneme_type\": \"text\"", 1);
        std::fs::write(&phonetic_config, phonetic).ok()?;

        let (jobs, job_rx) = channel::<Job>();
        let (done_tx, done) = channel::<String>();
        let tmp = cache.join("tmp");
        let cache_dir = cache.clone();
        // `move` gives the thread its own copies of these values.
        std::thread::spawn(move || {
            let mut english = Piper::spawn(&exe, &model, &config, &tmp);
            let mut phonetic = Piper::spawn(&exe, &model, &phonetic_config, &tmp);
            for job in job_rx {
                let piper = if job.phonetic { &mut phonetic } else { &mut english };
                if let Some(made) = piper.as_mut().and_then(|p| p.make(&job.text)) {
                    // Piper reports the file a moment before it closes it, and Windows
                    // won't move an open file, so try a few times.
                    let target = cache_dir.join(format!("{}.wav", job.name));
                    for _ in 0..50 {
                        if std::fs::rename(&made, &target).is_ok() {
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(20));
                    }
                }
                if done_tx.send(job.name).is_err() {
                    break; // the game has closed
                }
            }
        });

        Some(PiperVoice {
            cache,
            jobs,
            done,
            making: HashSet::new(),
            broken: HashSet::new(),
            loaded: HashMap::new(),
            queue: VecDeque::new(),
            playing: None,
            playing_until: 0.0,
        })
    }

    pub fn speak(&mut self, text: &str, interrupt: bool) {
        if interrupt {
            if let Some(s) = self.playing.take() {
                stop_sound(&s);
            }
            self.playing_until = 0.0;
            self.queue.clear();
        }
        self.queue.extend(parse(text).into_iter().map(job_for));
    }

    pub fn busy(&self) -> bool {
        !self.queue.is_empty() || get_time() < self.playing_until
    }

    /// Start making clips for this text now, without saying it.
    pub fn prepare(&mut self, text: &str) {
        for job in parse(text).into_iter().map(job_for) {
            self.request(job);
        }
    }

    pub async fn update(&mut self) {
        // Hear back from the background thread.
        while let Ok(name) = self.done.try_recv() {
            self.making.remove(&name);
            if !self.clip_path(&name).exists() {
                self.broken.insert(name);
            }
        }

        // Make sure the next few clips are loaded, or being made.
        let upcoming: Vec<(String, String, bool)> = self
            .queue
            .iter()
            .take(4)
            .map(|j| (j.name.clone(), j.text.clone(), j.phonetic))
            .collect();
        for (name, text, phonetic) in upcoming {
            if self.loaded.contains_key(&name) || self.broken.contains(&name) {
                continue;
            }
            if self.clip_path(&name).exists() {
                self.load(&name).await;
            } else {
                self.request(Job { name, text, phonetic });
            }
        }

        // When the current clip ends, play the next one (if it's ready).
        if get_time() >= self.playing_until {
            self.playing = None;
            while let Some(next) = self.queue.front() {
                if self.broken.contains(&next.name) {
                    self.queue.pop_front(); // skip it rather than get stuck
                    continue;
                }
                if let Some((sound, secs)) = self.loaded.get(&next.name) {
                    play_sound(sound, PlaySoundParams { looped: false, volume: 1.0 });
                    self.playing = Some(sound.clone());
                    self.playing_until = get_time() + *secs as f64;
                    self.queue.pop_front();
                }
                break;
            }
        }
    }

    fn request(&mut self, job: Job) {
        let exists = self.clip_path(&job.name).exists();
        if !exists && !self.making.contains(&job.name) && !self.broken.contains(&job.name) {
            self.making.insert(job.name.clone());
            let _ = self.jobs.send(job);
        }
    }

    async fn load(&mut self, name: &str) {
        // Convert to 44.1 kHz (what the sound system plays) with smooth resampling.
        let prepared = std::fs::read(self.clip_path(name))
            .ok()
            .and_then(|bytes| wav::decode(&bytes))
            .map(|audio| {
                let audio = audio.resample(44_100);
                (wav::encode(&audio), audio.seconds())
            });
        let sound = match &prepared {
            Some((bytes, _)) => load_sound_from_bytes(bytes).await.ok(),
            None => None,
        };
        match (sound, prepared) {
            (Some(sound), Some((_, secs))) => {
                self.loaded.insert(name.to_string(), (sound, secs));
            }
            _ => {
                self.broken.insert(name.to_string());
            }
        }
    }

    fn clip_path(&self, name: &str) -> PathBuf {
        self.cache.join(format!("{name}.wav"))
    }
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
            cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
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
