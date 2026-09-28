# Sebastian's Game

A touch-screen learning game for a 4-year-old, written in Rust with [macroquad](https://macroquad.rs).
Everything is drawn with code and all sound effects are synthesized. The voice is
[Piper](https://github.com/rhasspy/piper), a natural-sounding text-to-speech engine that runs offline.

## Setup

1. Install Rust: <https://rustup.rs> (on Windows you also need the Visual Studio C++ Build Tools).
2. Download the voice (about 90 MB, kept out of git):

   ```bash
   powershell -ExecutionPolicy Bypass -File scripts\get-voice.ps1
   ```

   Without it the game falls back to the robotic built-in Windows voice.
3. Run:

   ```bash
   cargo run --release
   ```

## Games

| Game | What it teaches | Levels |
|------|-----------------|--------|
| **Colors** | Drag the object into the matching bucket. Tapping a bucket says its color. | 2 → 6 colors |
| **Patterns** | Red, blue, red, blue… drag what comes next into the "?". Tapping a choice says its name. | AB → ABC, AAB, ABB, AABB |
| **Letters** | A picture clue ("ball") and the letter's sound: "Find the letter bee! Buh, buh, ball!" Drag the letter onto the picture; tapping a letter or the picture says its name and sound. | 5 → 26 letters |
| **Counting** | Tap each object to count it, then pick how many. Includes zero ("zero means none!"). | 0–3, 0–5, 4–7, 5–10, 7–10, 10–15, 11–20 |
| **Groups** | Early multiplication: tap each plate to count by 2s, 3s, 5s or 10s ("two, four, six"), then pick how many altogether. | 2s, 2s+, 5s, 10s, 3s, mixed |

- In Colors, Patterns and Letters, **tapping** an answer just says its name; **dragging** it to
  the target answers. A cartoon hand shows how to drag when each game starts.
- Tapping a game opens a **level picker**; the current level glows. Games also level up on
  their own after **4 right answers in a row**.
- Hints after misses (the right answer glows or bounces). Wrong answers never cost anything.
- Words are labeled in capitals with the **first letter big**, to connect words and letters.
- **Stars:** every right answer earns a star. Tap the star counter to count them all together.
- **Top buttons:** home, a "?" speech bubble (says the question again), and the speaker in the
  top-right corner, which turns all sound off (red X) and back on.
- Tap a letter in the title on the home screen to hear its name and sound.

## Players

The first time the game opens it asks for a name (a grown-up types it on the big on-screen
keyboard or the laptop keyboard). Add more kids from the **"Who's playing?"** screen (the
people button, top-left on the home screen). Each kid keeps their own stars and levels, saved in
`%APPDATA%sebastians-gameplayers.json`. Edit or delete that file to rename or remove a player.

## Grown-up controls

- `Esc` quits. `↑` / `↓` change the level of the current game.
- `cargo run -- --windowed` runs in a window instead of fullscreen.

## Customizing

- **Starting letters:** edit `LETTER_ORDER` in `src/games/letters.rs` and put the letters he knows first.
- **Your own voice:** every phrase is saved in `assets/voice-cache/` as a `.wav` named after its words
  (e.g. `where-does-the-red-ball-go.wav`). Record your own clip with the same name to replace it.
- **Different Piper voice:** `scripts\get-voice.ps1 -Voice en_US-amy-medium`
  (samples: <https://rhasspy.github.io/piper-samples/>).
- **Font:** put any `.ttf` at `assets/font.ttf` (by default it uses Comic Sans from Windows).

## Developer options

- `--gallery` shows all 26 alphabet pictures.
- `--levels 0` opens game 0's level picker.
- `--start 3:5` jumps straight into game 3 (Counting) at level 5. Games count from 0
  (0 Colors, 1 Patterns, 2 Letters, 3 Counting, 4 Groups).
- `--stars 12` starts with 12 stars; `--show-stars` opens the star panel.
- `--players test.json` uses a different players file (so testing doesn't touch the real one).
- `--snapshot out.png` saves a screenshot after 2.5 s (`--snapshot-after 5` to change) and quits.
- `cargo test` runs the unit tests.
- Every run writes `sebastians-game.log` (taps and what the voice does); `--no-log` turns it off.

## Code tour (for learning Rust)

- `src/main.rs`: window setup and the main loop (update, then draw, every frame)
- `src/games/mod.rs`: the `MiniGame` **trait** that every game implements, plus shared helpers
- `src/games/*.rs`: one file per game; each is a **struct** holding its state
- `src/art.rs`, `src/pictures.rs`: all drawing; **enums** like `Paint`, `Thing` and `Picture` with `match`
- `src/alphabet.rs`: a table of letters built with a small **macro**
- `src/voice/`: an **enum** that holds either voice; Piper runs on a background **thread** and
  talks back over **channels**
- `src/wav.rs`, `src/sfx.rs`: reading and writing WAV files byte by byte
- `src/input.rs`: turns touch or mouse into one "finger"; calls Windows directly through **FFI**
