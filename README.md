# Star Catchers

A touch-screen learning game for a 4-year-old (made for Sebastian): every right answer catches a star.
It's written in Rust with [macroquad](https://macroquad.rs).
Everything is drawn with code and all sound effects are synthesized. The voice is
[Piper](https://github.com/rhasspy/piper), a natural-sounding text-to-speech engine that runs offline.

## Screenshots

| | |
|:---:|:---:|
| ![Home screen with ten games](docs/screenshots/menu.png) **Ten games** | ![Who's playing](docs/screenshots/who.png) **Who's playing? (with 2-player mode)** |
| ![Colors](docs/screenshots/colors.png) **Colors:** sort everything by color | ![Patterns](docs/screenshots/patterns-necklace.png) **Patterns:** find the missing bead |
| ![Shadows](docs/screenshots/shadows.png) **Shadows:** match each picture to its shadow | ![Puzzles](docs/screenshots/puzzle.png) **Puzzles:** drag the pieces into place |
| ![Letters](docs/screenshots/letters.png) **Letters:** "Find the letter B! Buh, buh, ball!" | ![Spelling](docs/screenshots/spelling.png) **Spelling:** type the word on a big keyboard |
| ![Counting](docs/screenshots/counting.png) **Counting:** tap each one to count | ![Groups](docs/screenshots/groups.png) **Groups:** count by 2s, 5s and 10s |
| ![Drawing](docs/screenshots/drawing.png) **Drawing:** paint or color in a page | ![Level done](docs/screenshots/level-done.png) **Level done!** with fireworks |
| ![Two players](docs/screenshots/versus.png) **Two players** take turns | ![Winner](docs/screenshots/winner.png) **Winner!** "10 is more than 7" |
| ![Voices](docs/screenshots/voices.png) **Pick a voice** | ![Writing](docs/screenshots/writing.png) **Writing:** green where you're on the line, orange (with a red glow) where you strayed, gold where you missed |

## Setup

1. Install Rust: <https://rustup.rs> (on Windows you also need the Visual Studio C++ Build Tools).
2. Download Piper and five voices (Lessac, Amy, Kristin, Holly and LJ; about 350 MB, kept out of git):

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
| **Colors** | Lots of objects in different colors: drag each one into the bucket with the same color. Tapping an object or bucket says its name. | 2 colors / 4 things → 6 colors / 8 things |
| **Patterns** | Find the missing piece and drag it into the "?". Rows (red, blue, red, blue…), big/small and turning arrows, a necklace of beads, a checkerboard grid, and growing patterns (1, 2, 3, 4…). Tapping any card says its name. | AB colors → ABC → big/small & arrows → necklace → grid → growing |
| **Shadows** | Drag each colorful picture onto its dark shadow. | 2 → 5 shadows |
| **Puzzles** | Drag picture pieces onto the board; they snap into place. Early levels show a faint guide. | 2 → 12 pieces |
| **Letters** | A picture clue ("ball") and the letter's sound: "Find the letter bee! Buh, buh, ball!" Drag the letter onto the picture; tapping a letter or the picture says its name and sound. | 5 → 26 letters |
| **Spelling** | A picture and its word ("Let's spell cat! C, A, T"); type it on the big keyboard. Each letter is said as it's typed; a wrong spelling is read back, then try again. "Say it" reads out whatever's typed. | 3, 4, 5, longer letters; with or without guide letters |
| **Counting** | Tap each object to count it, then pick how many. Includes zero ("zero means none!"). | 0–3, 0–5, 4–7, 5–10, 7–10, 10–15, 11–20 |
| **Groups** | Early multiplication: tap each plate to count by 2s, 3s, 5s or 10s ("two, four, six"), then pick how many altogether. | 2s, 2s+, 5s, 10s, 3s, mixed |
| **Drawing** | Paint with your finger: 10 colors, small/medium/big/rainbow brushes, eraser. Pick a blank page or a shape or picture outline to color in. | (no levels or stars) |
| **Writing** | Trace with your finger and see how close you stay to the line. Dotted lines, curves and shapes, then dotted capital letters (a hand shows each stroke in order, numbered 1, 2, 3), then letters with a missing part to finish, and finally a blank page to write the whole letter from a small picture of it. See below. | lines → curves → shapes → easy letters → all letters → finish it → half blank → write it |

- In Colors, Patterns and Letters, **tapping** an answer just says its name; **dragging** it to
  the target answers. A cartoon hand shows how to drag when each game starts.
- Tapping a game opens a **level picker**; the current level glows. Games also level up on
  their own after **4 right answers in a row**.
- Hints after misses (the right answer glows or bounces). Wrong answers never cost anything.
- Words are labeled in capitals with the **first letter big**, to connect words and letters.
- **Stars:** every right answer earns a star. Tap the star counter to count them all together: by ones for a few, otherwise in groups of 2, 3, 5 or 10 chosen at random (never the same twice in a row), then ones for the leftovers.
- **Top buttons:** home, a "?" speech bubble (says the question again), and the speaker in the
  top-right corner, which turns all sound off (red X) and back on.
- Tap a letter in the title on the home screen to hear its name and sound.

### How Writing gives feedback

- **While you draw:** the ink is **green** where you're on the line and **orange** where you've
  drifted away from it, so you can correct as you go.
- **When you lift your finger and it isn't right:** the orange parts get a **red glow**, the parts
  of the line you missed light up **gold**, and the voice says what to fix ("Start at number 2!",
  "Go this way!", "You went off the line!", "Keep going to the end!"). Then you try that stroke again.
- **It gets easier if you're stuck:** each miss makes the path a little wider, and after two
  misses the hand shows the stroke again. It also shows it again if nothing is touched for a while,
  or when you tap the "?" button. Later levels only show the dots once you've missed.
- **Nothing is taken away.** A stroke only counts when it's right, and a level goes up after 4
  in a row that took at most one retry.
- Strokes have to be drawn in order (1, 2, 3) and the right way round, so the habit is built early.
  On the last level the strokes can be in any order; tap the green check when you're done, or the
  arrow to start over.

## Players

The first time the game opens it asks for a name (a grown-up types it on the big on-screen
keyboard or the laptop keyboard). Add more kids from the **"Who's playing?"** screen (the
people button, top-left on the home screen). Each kid keeps their own stars and levels, saved in
`%APPDATA%\star-catchers\players.json` (you can edit it by hand, e.g. to fix a name).

**Two players:** tap **2 Players** on "Who's playing?", pick two kids and a goal (first to 5, 10,
15 or 20 stars). Turns switch after every star; both scores show in the top-right corner, and the
winner gets a fireworks screen that compares the two scores ("10 is more than 7!"). Match scores
are separate from each kid's own saved stars.

**Voice:** tap **Voice** at the bottom of "Who's playing?" to hear each installed voice and pick
one. The choice is saved.

**Winner screen:** playing alone, every 50 stars brings fireworks and a "50 stars!" screen.

**Level tracking:** finishing a level (4 right in a row) shows a "Level 3 done! On to level 4!"
banner, and the level picker puts a green check on finished levels (saved per player).

To remove a player, tap **Edit** (bottom-right of "Who's playing?"), then the red X, then
**Remove**. When a player with stars is picked, the game asks whether to **keep** their stars or
**start at 0** (levels are kept either way).

## Grown-up controls

- `Esc` quits. `↑` / `↓` change the level of the current game.
- `cargo run -- --windowed` runs in a window instead of fullscreen.

## Customizing

- **Starting letters:** edit `LETTER_ORDER` in `src/games/letters.rs` and put the letters he knows first.
- **Your own voice:** every phrase is saved in `assets/voice-cache/<voice>/` as a `.wav` named after
  its words (e.g. `where-does-the-red-ball-go.wav`). Record your own clip with the same name to replace it.
- **More Piper voices:** `scripts\get-voice.ps1 -Voices en_US-ryan-medium` downloads another voice into
  `assets/piper/voices/`, and it shows up in the voice menu
  (samples: <https://rhasspy.github.io/piper-samples/>).
- **Font:** put any `.ttf` at `assets/font.ttf` (by default it uses Comic Sans from Windows).

## Developer options

- `--gallery` shows all 26 alphabet pictures.
- `--levels 0` opens game 0's level picker.
- `--versus 0:1:10` starts a match between players 0 and 1 (first to 10); `--demo-win` shows
  the winner screen; `--banner` shows the "level done" banner.
- `--voices` opens the voice menu; `--voices holly` also picks Holly.
- `--pick 0` picks player 0; `--edit-players` opens the remove-a-player mode.
- `--things` shows every object and shape; `--solve` fills in the puzzle; `--page 8` opens a coloring page.
- `--start 3:5` jumps straight into game 3 (Counting) at level 5. Games count from 0
  (0 Colors, 1 Patterns, 2 Shadows, 3 Puzzles, 4 Letters, 5 Spelling, 6 Counting, 7 Groups, 8 Drawing, 9 Writing).
- `--stars 12` starts with 12 stars; `--show-stars` opens the star panel.
- `--players test.json` uses a different players file (so testing doesn't touch the real one).
- `--snapshot out.png` saves a screenshot after 2.5 s (`--snapshot-after 5` to change) and quits.
- `cargo test` runs the unit tests.
- Every run writes `star-catchers.log` (taps, the voice, frame speed every 5 s, crashes, and
  freezes with where they happened); the run before is kept as `star-catchers.prev.log`. `--no-log` turns it off.
- `--fake-paint` makes the Drawing game paint by itself (a stress test).
- `--fake-write` makes a pretend finger trace the first stroke in Writing and wander off the line,
  to check the feedback (try `--start 9:5 --fake-write --snapshot out.png --snapshot-after 3.4`).
- `--menu` shows the home screen without picking a player.
- `--vsync` waits for the screen's refresh instead of pacing frames itself (the old way; on battery
  Windows can make that wait long enough to drop the game to 20 fps). Slow frames (over 40 ms) are
  logged with a breakdown of where the time went.

## Code tour (for learning Rust)

- `src/main.rs`: window setup and the main loop (update, then draw, every frame)
- `src/games/mod.rs`: the `MiniGame` **trait** that every game implements, plus shared helpers
- `src/games/*.rs`: one file per game; each is a **struct** holding its state
- `src/art.rs`, `src/pictures.rs`: all drawing; **enums** like `Paint`, `Thing` and `Picture` with `match`
- `src/alphabet.rs`: a table of letters built with a small **macro**
- `src/glyphs.rs`, `src/tracing.rs`, `src/games/writing.rs`: the Writing game. Stroke data for every
  letter (lines and arcs turned into points), the geometry that judges a finger stroke against a
  line (pure functions with unit tests), and the game that ties them to drawing and the voice
- `src/voice/`: an **enum** that holds either voice; Piper runs on a background **thread** and
  talks back over **channels**
- `src/wav.rs`, `src/sfx.rs`: reading and writing WAV files byte by byte
- `src/input.rs`: turns touch or mouse into one "finger"; calls Windows directly through **FFI**
- `src/power.rs`: asks Windows not to slow the game down on battery (more **FFI**)
