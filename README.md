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
| **Shadows** | Drag each colorful picture onto its dark shadow. | 2 → 5 shadows |
| **Puzzles** | Drag picture pieces onto the board; they snap into place. Early levels show a faint guide. | 2 → 12 pieces |
| **Letters** | A picture clue ("ball") and the letter's sound: "Find the letter bee! Buh, buh, ball!" Drag the letter onto the picture; tapping a letter or the picture says its name and sound. | 5 → 26 letters |
| **Spelling** | A picture and its word ("Let's spell cat! C, A, T"); type it on the big keyboard. Each letter is said as it's typed; a wrong spelling is read back, then try again. "Say it" reads out whatever's typed. | 3, 4, 5, longer letters; with or without guide letters |
| **Counting** | Tap each object to count it, then pick how many. Includes zero ("zero means none!"). | 0–3, 0–5, 4–7, 5–10, 7–10, 10–15, 11–20 |
| **Groups** | Early multiplication: tap each plate to count by 2s, 3s, 5s or 10s ("two, four, six"), then pick how many altogether. | 2s, 2s+, 5s, 10s, 3s, mixed |
| **Drawing** | Paint with your finger: 10 colors, small/medium/big/rainbow brushes, eraser. Pick a blank page or a shape or picture outline to color in. | (no levels or stars) |

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
`%APPDATA%\sebastians-game\players.json` (you can edit it by hand, e.g. to fix a name).

To remove a player, tap **Edit** (bottom-right of "Who's playing?"), then the red X, then
**Remove**. When a player with stars is picked, the game asks whether to **keep** their stars or
**start at 0** (levels are kept either way).

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
- `--pick 0` picks player 0; `--edit-players` opens the remove-a-player mode.
- `--things` shows every object and shape; `--solve` fills in the puzzle; `--page 8` opens a coloring page.
- `--start 3:5` jumps straight into game 3 (Counting) at level 5. Games count from 0
  (0 Colors, 1 Patterns, 2 Shadows, 3 Puzzles, 4 Letters, 5 Spelling, 6 Counting, 7 Groups, 8 Drawing).
- `--stars 12` starts with 12 stars; `--show-stars` opens the star panel.
- `--players test.json` uses a different players file (so testing doesn't touch the real one).
- `--snapshot out.png` saves a screenshot after 2.5 s (`--snapshot-after 5` to change) and quits.
- `cargo test` runs the unit tests.
- Every run writes `sebastians-game.log` (taps, the voice, crashes, and freezes with where they
  happened); the run before is kept as `sebastians-game.prev.log`. `--no-log` turns it off.
- `--fake-paint` makes the Drawing game paint by itself (a stress test).

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
