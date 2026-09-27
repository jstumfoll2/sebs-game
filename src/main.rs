// In release builds, don't open a console window next to the game.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod art;
mod ctx;
mod fx;
mod games;
mod hud;
mod input;
mod menu;
mod sfx;
mod speech;

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

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as u64);

    let mut ctx = Ctx {
        input: Input::default(),
        font: art::load_font(),
        sfx: sfx::Sfx::load().await,
        voice: speech::Voice::new(),
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
    // `None` = on the menu, `Some(i)` = playing game number i.
    let mut current: Option<usize> = None;

    ctx.voice.say("Hi Sebastian! Pick a game!");

    loop {
        // Grown-up controls: Esc quits, Up/Down arrows change the level.
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        ctx.dt = get_frame_time().min(0.05);
        ctx.time += ctx.dt;
        ctx.input = Input::read();
        ctx.star_pop = (ctx.star_pop - ctx.dt * 2.0).max(0.0);
        ctx.confetti.update(ctx.dt);

        // ----- update -----
        match current {
            None => {
                if let Some(i) = menu.update(&mut ctx) {
                    ctx.sfx.pop();
                    current = Some(i);
                    games[i].enter(&mut ctx);
                }
            }
            Some(i) => {
                let game = &mut games[i];
                let level_change = if is_key_pressed(KeyCode::Up) {
                    1
                } else if is_key_pressed(KeyCode::Down) {
                    -1
                } else {
                    0
                };
                if level_change != 0 {
                    let level = game.progress().level as i32 + level_change;
                    game.progress().set_level(level.max(1) as u32);
                    game.enter(&mut ctx);
                } else if ctx.input.tapped(hud::home_rect()) {
                    ctx.sfx.pop();
                    ctx.voice.say("Pick a game!");
                    current = None;
                } else if ctx.input.tapped(hud::repeat_rect()) {
                    ctx.sfx.pop();
                    let prompt = game.prompt();
                    ctx.voice.say(&prompt);
                } else {
                    game.update(&mut ctx);
                }
            }
        }

        // ----- draw -----
        art::background(ctx.time);
        match current {
            None => menu.draw(&ctx),
            Some(i) => {
                games[i].draw(&ctx);
                let level = games[i].progress().level;
                hud::draw_game_buttons(&ctx, level);
            }
        }
        hud::draw_stars(&ctx);
        ctx.confetti.draw();

        next_frame().await;
    }
}
