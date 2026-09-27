// In release builds, don't open a console window next to the game.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod alphabet;
mod art;
mod assets;
mod ctx;
mod fx;
mod games;
mod hud;
mod input;
mod levels;
mod log;
mod menu;
mod pictures;
mod sfx;
mod stars;
mod voice;
mod wav;

use ctx::Ctx;
use games::{
    color_sort::ColorSort, counting::Counting, letters::Letters, pattern::Pattern, MiniGame,
};
use input::Input;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    // `cargo run -- --windowed` runs in a window instead of fullscreen (handy while coding).
    let windowed = std::env::args().any(|a| a == "--windowed");
    Conf {
        window_title: "Sebastian's Game".to_string(),
        fullscreen: !windowed,
        window_width: 1280,
        window_height: 800,
        window_resizable: true,
        // Without this, Windows display scaling (e.g. 150%) makes taps land in the wrong spot.
        high_dpi: true,
        ..Default::default()
    }
}

/// The value after a command-line flag, e.g. `arg_value("--start")` for `--start 2:3`.
fn arg_value(flag: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    let i = args.iter().position(|a| a == flag)?;
    args.get(i + 1).cloned()
}

/// Developer helper: `--snapshot out.png` saves a picture of the screen after a couple of
/// seconds and quits (works even when the laptop is locked). Returns true when it's time to quit.
fn snapshot_done(time: f32) -> bool {
    let Some(path) = arg_value("--snapshot") else { return false };
    let after: f32 = arg_value("--snapshot-after").and_then(|s| s.parse().ok()).unwrap_or(2.5);
    if time < after {
        return false;
    }
    get_screen_data().export_png(&path);
    true
}

/// Which screen we're on. The number is the game (same order as the menu tiles).
#[derive(Clone, Copy, PartialEq, Debug)]
enum Screen {
    Menu,
    Levels(usize),
    Playing(usize),
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as u64);

    let mut ctx = Ctx {
        input: Input::default(),
        font: art::load_font(),
        sfx: sfx::Sfx::load().await,
        voice: voice::Voice::new(),
        confetti: fx::Confetti::default(),
        stars: 0,
        star_pop: 0.0,
        time: 0.0,
        dt: 0.0,
    };

    // Same order as the menu tiles.
    let mut games: Vec<Box<dyn MiniGame>> = vec![
        Box::new(ColorSort::new()),
        Box::new(Pattern::new()),
        Box::new(Letters::new()),
        Box::new(Counting::new()),
    ];
    let mut menu = menu::Menu::new();
    let mut screen = Screen::Menu;
    let mut star_panel = stars::StarPanel::default();

    ctx.voice.say("Hi Sebastian! Pick a game!");
    // Get common phrases ready in the background so they play instantly later.
    let mut common: Vec<String> = ctx::PRAISE.iter().map(|s| s.to_string()).collect();
    common.extend(games::counting::NUMBER_WORDS.iter().map(|s| s.to_string()));
    common.extend(art::Paint::ALL.iter().map(|p| p.name().to_string()));
    common.extend(["Pick a game!", "Pick a level!"].map(String::from));
    ctx.voice.prepare(&common);

    // `cargo run -- --gallery` shows every alphabet picture (handy when drawing new ones).
    if std::env::args().any(|a| a == "--gallery") {
        loop {
            art::background(get_time() as f32);
            let (w, h) = (screen_width(), screen_height());
            let (cols, rows) = (7, 4);
            let cell = (w / cols as f32).min(h / rows as f32);
            for (i, l) in alphabet::LETTERS.iter().enumerate() {
                let (col, row) = (i % cols, i / cols);
                let r = Rect::new(col as f32 * cell + cell * 0.05, row as f32 * cell + cell * 0.05, cell * 0.9, cell * 0.9);
                art::card(r, WHITE);
                pictures::draw(l.picture, vec2(r.center().x, r.y + r.h * 0.42), r.h * 0.3);
                art::word_label(ctx.font(), l.word, vec2(r.center().x, r.y + r.h * 0.86), r.h * 0.1, r.w * 0.9, art::Paint::Red.color(), None);
            }
            if is_key_pressed(KeyCode::Escape) || snapshot_done(get_time() as f32) {
                return;
            }
            next_frame().await;
        }
    }

    // `--stars 7` starts with 7 stars; `--show-stars` opens the star panel right away.
    if let Some(n) = arg_value("--stars").and_then(|s| s.parse().ok()) {
        ctx.stars = n;
    }
    if std::env::args().any(|a| a == "--show-stars") {
        star_panel.open(&mut ctx);
    }

    // `--start 2:3` jumps straight into game 2 (Letters) at level 3. Games count from 0.
    if let Some(start) = arg_value("--start") {
        let mut parts = start.split(':').map(|p| p.parse::<u32>().ok());
        if let Some(Some(i)) = parts.next() {
            let i = (i as usize).min(games.len() - 1);
            if let Some(Some(level)) = parts.next() {
                games[i].progress_mut().set_level(level);
            }
            games[i].enter(&mut ctx);
            screen = Screen::Playing(i);
        }
    }

    loop {
        // Grown-up control: Esc quits.
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        ctx.dt = get_frame_time().min(0.05);
        ctx.time += ctx.dt;
        ctx.input = Input::read();
        if ctx.input.pressed {
            let (mx, my) = mouse_position();
            log::line(&format!(
                "tap at {:?} (macroquad says {mx:.0},{my:.0}; screen {}x{}, dpi {}; {}; touches {}) on {screen:?}; sound button {:?}",
                ctx.input.pos,
                screen_width(),
                screen_height(),
                screen_dpi_scale(),
                input::debug_pointer(),
                touches().len(),
                hud::repeat_rect()
            ));
        }
        ctx.star_pop = (ctx.star_pop - ctx.dt * 2.0).max(0.0);
        ctx.confetti.update(ctx.dt);
        ctx.voice.update().await;

        // ----- update -----
        // The star panel (tap the star counter) sits on top of everything while it's open.
        let panel_open = star_panel.update(&mut ctx);
        if !panel_open {
            update_screen(&mut screen, &mut games, &mut menu, &mut ctx);
        }

        // ----- draw -----
        art::background(ctx.time);
        match screen {
            Screen::Menu => menu.draw(&ctx),
            Screen::Levels(i) => {
                levels::draw(games[i].as_ref(), menu::LABELS[i], &ctx);
                hud::draw_home_button();
            }
            Screen::Playing(i) => {
                games[i].draw(&ctx);
                let level = games[i].progress().level;
                hud::draw_game_buttons(&ctx, level);
            }
        }
        hud::draw_stars(&ctx);
        ctx.confetti.draw();
        star_panel.draw(&ctx);

        if snapshot_done(ctx.time) {
            break;
        }
        next_frame().await;
    }
}

/// Handle taps and game logic for whichever screen we're on.
fn update_screen(screen: &mut Screen, games: &mut [Box<dyn MiniGame>], menu: &mut menu::Menu, ctx: &mut Ctx) {
    match *screen {
        Screen::Menu => {
            if let Some(i) = menu.update(ctx) {
                ctx.sfx.pop();
                ctx.voice.then("Pick a level!");
                *screen = Screen::Levels(i);
            }
        }
        Screen::Levels(i) => {
            if ctx.input.tapped(hud::home_rect()) {
                ctx.sfx.pop();
                ctx.voice.say("Pick a game!");
                *screen = Screen::Menu;
            } else if let Some(level) = levels::update(games[i].as_ref(), ctx) {
                ctx.sfx.pop();
                games[i].progress_mut().set_level(level);
                ctx.voice.say(&format!("Level {level}!"));
                games[i].enter(ctx);
                *screen = Screen::Playing(i);
            }
        }
        Screen::Playing(i) => {
            let game = &mut games[i];
            // Grown-up control: Up/Down arrows change the level.
            let level_change = if is_key_pressed(KeyCode::Up) {
                1
            } else if is_key_pressed(KeyCode::Down) {
                -1
            } else {
                0
            };
            if level_change != 0 {
                let level = game.progress().level as i32 + level_change;
                game.progress_mut().set_level(level.max(1) as u32);
                game.enter(ctx);
            } else if ctx.input.tapped(hud::home_rect()) {
                ctx.sfx.pop();
                ctx.voice.say("Pick a game!");
                *screen = Screen::Menu;
            } else if ctx.input.tapped(hud::repeat_rect()) {
                ctx.sfx.pop();
                let prompt = game.prompt();
                ctx.voice.say(&prompt);
            } else {
                game.update(ctx);
            }
        }
    }
}
