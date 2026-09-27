//! Talking! Uses the text-to-speech voice built into Windows.
//!
//! We start ONE hidden PowerShell process when the game opens and keep it running.
//! Each line we write to its input gets spoken. Starting PowerShell is slow (~1s),
//! so doing it once up front keeps speech snappy afterwards.
//!
//! Speech is slow compared to the screen, so the game needs to know when the voice is
//! done (e.g. to not show the next round while still saying "Red!"). PowerShell reports
//! back: whenever it goes quiet, it prints how many lines it has received so far.
//! If that number matches how many we've sent, the voice is finished.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// The PowerShell program that runs in the background.
/// Lines starting with "!" interrupt whatever is being said; other lines wait their turn.
const SPEAKER_SCRIPT: &str = r#"
Add-Type -AssemblyName System.Speech
$s = New-Object System.Speech.Synthesis.SpeechSynthesizer
$s.Rate = -1
$zira = $s.GetInstalledVoices() | ForEach-Object { $_.VoiceInfo.Name } | Where-Object { $_ -like '*Zira*' } | Select-Object -First 1
if ($zira) { $s.SelectVoice($zira) }
$in = New-Object System.IO.StreamReader([Console]::OpenStandardInput())
$read = $in.ReadLineAsync()
$prompts = New-Object System.Collections.ArrayList
$n = 0; $reported = 0
while ($true) {
    if ($read.Wait(20)) {
        $line = $read.Result
        if ($null -eq $line) { break }
        $n++
        if ($line.StartsWith('!')) { $s.SpeakAsyncCancelAll(); $line = $line.Substring(1) }
        if ($line.Length -gt 0) { [void]$prompts.Add($s.SpeakAsync($line)) }
        $read = $in.ReadLineAsync()
    }
    if (-not ($prompts | Where-Object { -not $_.IsCompleted })) {
        $prompts.Clear()
        if ($n -ne $reported) { [Console]::Out.WriteLine($n); [Console]::Out.Flush(); $reported = $n }
    }
}
"#;

pub struct Voice {
    stdin: Option<ChildStdin>,
    child: Option<Child>,
    /// Lines we've sent to be spoken.
    sent: u64,
    /// Lines PowerShell has finished speaking (updated by a background thread).
    finished: Arc<AtomicU64>,
}

impl Voice {
    pub fn new() -> Self {
        let finished = Arc::new(AtomicU64::new(0));
        match spawn_speaker() {
            Some((child, stdin, stdout)) => {
                // A thread that waits for PowerShell's "I'm done" messages.
                let finished_writer = Arc::clone(&finished);
                std::thread::spawn(move || {
                    for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                        if let Ok(n) = line.trim().parse::<u64>() {
                            finished_writer.store(n, Ordering::Relaxed);
                        }
                    }
                });
                Voice {
                    stdin: Some(stdin),
                    child: Some(child),
                    sent: 0,
                    finished,
                }
            }
            // No voice available: the game still works, just silently.
            None => Voice {
                stdin: None,
                child: None,
                sent: 0,
                finished,
            },
        }
    }

    /// Is the voice still talking (or about to)?
    pub fn busy(&self) -> bool {
        self.stdin.is_some() && self.finished.load(Ordering::Relaxed) < self.sent
    }

    /// Stop talking and say this right now.
    pub fn say(&mut self, text: &str) {
        self.send(&format!("!{}", clean(text)));
    }

    /// Say this after whatever is currently being said.
    pub fn then(&mut self, text: &str) {
        self.send(&clean(text));
    }

    fn send(&mut self, line: &str) {
        if let Some(stdin) = &mut self.stdin {
            let ok = writeln!(stdin, "{line}").and_then(|_| stdin.flush());
            if ok.is_ok() {
                self.sent += 1;
            } else {
                self.stdin = None; // the speaker process died; go quiet instead of crashing
            }
        }
    }
}

impl Drop for Voice {
    fn drop(&mut self) {
        self.stdin = None;
        if let Some(child) = &mut self.child {
            let _ = child.kill();
        }
    }
}

/// Keep only simple characters so nothing odd gets sent to PowerShell.
fn clean(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_ascii_alphanumeric() || " ,.!?'-".contains(*c))
        .collect()
}

#[cfg(windows)]
fn spawn_speaker() -> Option<(Child, ChildStdin, ChildStdout)> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let mut child = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", SPEAKER_SCRIPT])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .ok()?;
    let stdin = child.stdin.take()?;
    let stdout = child.stdout.take()?;
    Some((child, stdin, stdout))
}

#[cfg(not(windows))]
fn spawn_speaker() -> Option<(Child, ChildStdin, ChildStdout)> {
    None
}
