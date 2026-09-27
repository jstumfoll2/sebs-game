//! Talking! Uses the text-to-speech voice built into Windows.
//!
//! We start ONE hidden PowerShell process when the game opens and keep it running.
//! Each line we write to its input gets spoken. Starting PowerShell is slow (~1s),
//! so doing it once up front keeps speech snappy afterwards.

use std::io::Write;
use std::process::{Child, ChildStdin, Command, Stdio};

/// The PowerShell program that runs in the background.
/// Lines starting with "!" interrupt whatever is being said; other lines wait their turn.
const SPEAKER_SCRIPT: &str = r#"
Add-Type -AssemblyName System.Speech
$s = New-Object System.Speech.Synthesis.SpeechSynthesizer
$s.Rate = -1
$zira = $s.GetInstalledVoices() | ForEach-Object { $_.VoiceInfo.Name } | Where-Object { $_ -like '*Zira*' } | Select-Object -First 1
if ($zira) { $s.SelectVoice($zira) }
while ($null -ne ($line = [Console]::In.ReadLine())) {
    if ($line.StartsWith('!')) { $s.SpeakAsyncCancelAll(); $line = $line.Substring(1) }
    if ($line.Length -gt 0) { [void]$s.SpeakAsync($line) }
}
"#;

pub struct Voice {
    stdin: Option<ChildStdin>,
    child: Option<Child>,
}

impl Voice {
    pub fn new() -> Self {
        match spawn_speaker() {
            Some((child, stdin)) => Voice {
                stdin: Some(stdin),
                child: Some(child),
            },
            // No voice available: the game still works, just silently.
            None => Voice {
                stdin: None,
                child: None,
            },
        }
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
            if ok.is_err() {
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
fn spawn_speaker() -> Option<(Child, ChildStdin)> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let mut child = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", SPEAKER_SCRIPT])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .ok()?;
    let stdin = child.stdin.take()?;
    Some((child, stdin))
}

#[cfg(not(windows))]
fn spawn_speaker() -> Option<(Child, ChildStdin)> {
    None
}
