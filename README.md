# Sebastian's Game

A touch-screen learning game for a 4-year-old, written in Rust with [macroquad](https://macroquad.rs).
Everything is drawn with code and all sounds are synthesized, so there are no asset files.
The voice uses the text-to-speech built into Windows.

## Games

| Game | What it teaches | How it gets harder |
|------|-----------------|--------------------|
| **Colors** | Drag (or tap) the object into the matching bucket. The voice says color names. | 2 → 6 buckets, then all 8 colors |
| **Patterns** | Red, blue, red, blue… what comes next? | AB → ABC, AAB, ABB, AABB; color, shape, or both change |
| **Letters** | "Find the letter B!" | 5 → 26 letters, 3 → 5 choices |
| **Counting** | Tap each object to count it (a number badge appears), then pick how many | 1–3 → up to 10 objects |

Each game moves up a level after **4 right answers in a row**. After two misses, the right answer
glows or bounces as a hint. Wrong answers get a gentle "uh-oh" and never a penalty.

## Running

```bash
cargo run --release
```

Add `-- --windowed` to run in a window instead of fullscreen.

**Grown-up controls:** `Esc` quits. `↑` / `↓` change the level of the current game.

## Customizing

- **Starting letters:** edit `LETTER_ORDER` in `src/games/letters.rs` and put the letters he knows first.
- **Font:** put any `.ttf` at `assets/font.ttf` (by default it uses Comic Sans from Windows).
- **Voice:** it uses "Microsoft Zira" if installed. Change it in `src/speech.rs`.

## Code tour (for learning Rust)

- `src/main.rs`: window setup and the main loop (update, then draw, every frame)
- `src/games/mod.rs`: the `MiniGame` **trait** that every game implements, plus shared helpers
- `src/games/*.rs`: one file per game; each is a **struct** holding its state
- `src/art.rs`: all drawing; **enums** `Paint` and `Thing` with `match`
- `src/speech.rs`: spawns a background process and talks to it over a pipe; uses `#[cfg(windows)]`
- `src/sfx.rs`: builds WAV files byte by byte in memory
- `src/input.rs`: turns touch or mouse into one simple "finger"
