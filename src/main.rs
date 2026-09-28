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
mod players;
mod sfx;
mod stars;
mod voice;
mod wav;
mod who;

use ctx::Ctx;
use games::{
    color_sort::ColorSort, counting::Counting, letters::Letters, pattern::Pattern, MiniGame,
};
use macroquad::prelude::*;

fn window_conf() -> Conf {
    // `cargo run -- --windowed` runs in a window instead of fullscreen (handy while coding).
    let windowed = std::env::args().any(|a| a == "--windowed");
    Conf {
        window_title: "Play and Learn".to_string(),
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
    /// "Who's playing?"
    Players,
    /// Typing a new player's name.
    NewName,
    Menu,
    Levels(usize),
    Playing(usize),
}

/// Everything the game keeps track of between frames (besides `Ctx`).
struct App {
    screen: Screen,
    /// Same order as the menu tiles.
    games: Vec<Box<dyn MiniGame>>,
    menu: menu::Menu,
    players: players::Players,
    /// Which player is playing (index into `players.players`).
    current: Option<usize>,
    name_entry: who::NameEntry,
    star_panel: stars::StarPanel,
}

impl App {
    /// The name we save each game's level under ("colors", "counting", ...).
    fn game_key(i: usize) -> String {
        menu::LABELS[i].to_lowercase()
    }

    /// Start playing as player `i`: bring back their stars and levels.
    fn pick_player(&mut self, i: usize, ctx: &mut Ctx) {
        let p = self.players.players[i].clone();
        ctx.name = p.name.clone();
        ctx.stars = p.stars;
        for (g, game) in self.games.iter_mut().enumerate() {
            let level = p.levels.get(&Self::game_key(g)).copied().unwrap_or(1);
            game.progress_mut().set_level(level);
        }
        self.current = Some(i);
        self.players.last_player = Some(i);
        self.players.save();
        self.menu = menu::Menu::new();
        self.screen = Screen::Menu;
        ctx.voice.say(&format!("Hi {}! Pick a game!", p.name));
    }

    /// Save stars and levels whenever they change.
    fn save_progress(&mut self, ctx: &Ctx) {
        let Some(i) = self.current else { return };
        let levels = (0..self.games.len())
            .map(|g| (Self::game_key(g), self.games[g].progress().level))
            .collect();
        let now = players::Player { name: ctx.name.clone(), stars: ctx.stars, levels };
        if self.players.players.get(i) != Some(&now) {
            self.players.players[i] = now;
            self.players.save();
        }
    }

    fn show_players(&mut self, ctx: &mut Ctx) {
        self.screen = Screen::Players;
        ctx.voice.say("Who's playing?");
    }

    fn new_name(&mut self, ctx: &mut Ctx) {
        self.screen = Screen::NewName;
        self.name_entry.start(ctx);
    }

    fn picking_player(&self) -> bool {
        matches!(self.screen, Screen::Players | Screen::NewName)
    }

    /// Handle taps and game logic for whichever screen we're on.
    fn update(&mut self, ctx: &mut Ctx) {
        // The speaker button (sound on/off) works on every screen.
        if ctx.input.tapped(hud::mute_rect()) {
            ctx.button_pop[hud::MUTE] = 1.0;
            ctx.toggle_mute();
            return;
        }
        // The star panel (tap the star counter) sits on top of everything while it's open.
        if !self.picking_player() && self.star_panel.update(ctx) {
            return;
        }

        match self.screen {
            Screen::Players => match who::update_picker(&self.players, ctx) {
                who::Pick::Player(i) => {
                    ctx.sfx.pop();
                    self.pick_player(i, ctx);
                }
                who::Pick::AddNew => {
                    ctx.sfx.pop();
                    self.new_name(ctx);
                }
                who::Pick::Nothing => {}
            },
            Screen::NewName => {
                let can_cancel = !self.players.players.is_empty();
                match self.name_entry.update(ctx, can_cancel) {
                    who::Typed::Done(name) => {
                        let i = self.players.add(&name);
                        self.pick_player(i, ctx);
                    }
                    who::Typed::Cancel => self.show_players(ctx),
                    who::Typed::Nothing => {}
                }
            }
            Screen::Menu => {
                // Top-left on the menu: switch player.
                if ctx.input.tapped(hud::home_rect()) {
                    ctx.button_pop[hud::HOME] = 1.0;
                    ctx.sfx.pop();
                    self.show_players(ctx);
                } else if let Some(i) = self.menu.update(ctx) {
                    ctx.sfx.pop();
                    ctx.voice.then("Pick a level!");
                    self.screen = Screen::Levels(i);
                }
            }
            Screen::Levels(i) => {
                if ctx.input.tapped(hud::home_rect()) {
                    ctx.button_pop[hud::HOME] = 1.0;
                    ctx.sfx.pop();
                    ctx.voice.say("Pick a game!");
                    self.screen = Screen::Menu;
                } else if let Some(level) = levels::update(self.games[i].as_ref(), ctx) {
                    ctx.sfx.pop();
                    self.games[i].progress_mut().set_level(level);
                    ctx.voice.say(&format!("Level {level}!"));
                    self.games[i].enter(ctx);
                    self.screen = Screen::Playing(i);
                }
            }
            Screen::Playing(i) => {
                let game = &mut self.games[i];
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
                    ctx.button_pop[hud::HOME] = 1.0;
                    ctx.sfx.pop();
                    ctx.voice.say("Pick a game!");
                    self.screen = Screen::Menu;
                } else if ctx.input.tapped(hud::repeat_rect()) {
                    ctx.button_pop[hud::REPEAT] = 1.0;
                    ctx.sfx.pop();
                    let prompt = game.prompt();
                    ctx.voice.say(&prompt);
                } else {
                    game.update(ctx);
                }
            }
        }
        self.save_progress(ctx);
    }

    fn draw(&self, ctx: &Ctx) {
        art::background(ctx.time);
        match self.screen {
            Screen::Players => who::draw_picker(&self.players, ctx),
            Screen::NewName => self.name_entry.draw(ctx, !self.players.players.is_empty()),
            Screen::Menu => {
                self.menu.draw(ctx);
                hud::draw_players_button(ctx);
            }
            Screen::Levels(i) => {
                levels::draw(self.games[i].as_ref(), menu::LABELS[i], ctx);
                hud::draw_home_button(ctx);
            }
            Screen::Playing(i) => {
                self.games[i].draw(ctx);
                let level = self.games[i].progress().level;
                hud::draw_game_buttons(ctx, level);
            }
        }
        if !self.picking_player() {
            hud::draw_stars(ctx);
        }
        hud::draw_mute_button(ctx);
        ctx.confetti.draw();
        self.star_panel.draw(ctx);
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as u64);

    let mut ctx = Ctx {
        input: input::Input::default(),
        font: art::load_font(),
        sfx: sfx::Sfx::load().await,
        voice: voice::Voice::new(),
        confetti: fx::Confetti::default(),
        name: "friend".to_string(),
        stars: 0,
        star_pop: 0.0,
        time: 0.0,
        dt: 0.0,
        muted: false,
        button_pop: [0.0; 3],
        last_mute_toggle: -1.0,
    };

    let mut app = App {
        screen: Screen::Menu,
        games: vec![
            Box::new(ColorSort::new()),
            Box::new(Pattern::new()),
            Box::new(Letters::new()),
            Box::new(Counting::new()),
        ],
        menu: menu::Menu::new(),
        players: players::Players::load(),
        current: None,
        name_entry: who::NameEntry::default(),
        star_panel: stars::StarPanel::default(),
    };
    let mut taps = input::TapFilter::default();

    // Get common phrases ready in the background so they play instantly later.
    let mut common: Vec<String> = ctx::PRAISE.iter().filter(|s| !s.contains("NAME")).map(|s| s.to_string()).collect();
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

    // Who's playing? New here: ask for a name. One player: go straight in. Several: ask.
    match app.players.players.len() {
        0 => app.new_name(&mut ctx),
        1 => app.pick_player(0, &mut ctx),
        _ => app.show_players(&mut ctx),
    }

    // `--stars 7` starts with 7 stars; `--show-stars` opens the star panel right away.
    if let Some(n) = arg_value("--stars").and_then(|s| s.parse().ok()) {
        ctx.stars = n;
    }
    if std::env::args().any(|a| a == "--show-stars") {
        app.star_panel.open(&mut ctx);
    }

    // `--start 2:3` jumps straight into game 2 (Letters) at level 3. Games count from 0.
    if let Some(start) = arg_value("--start") {
        let mut parts = start.split(':').map(|p| p.parse::<u32>().ok());
        if let Some(Some(i)) = parts.next() {
            let i = (i as usize).min(app.games.len() - 1);
            if let Some(Some(level)) = parts.next() {
                app.games[i].progress_mut().set_level(level);
            }
            app.games[i].enter(&mut ctx);
            app.screen = Screen::Playing(i);
        }
    }

    loop {
        // Grown-up control: Esc quits.
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        ctx.dt = get_frame_time().min(0.05);
        ctx.time += ctx.dt;
        ctx.input = taps.read();
        if ctx.input.pressed {
            let (mx, my) = mouse_position();
            log::line(&format!(
                "tap at {:?} (macroquad says {mx:.0},{my:.0}; screen {}x{}, dpi {}; {}; touches {}) on {:?}",
                ctx.input.pos,
                screen_width(),
                screen_height(),
                screen_dpi_scale(),
                input::debug_pointer(),
                touches().len(),
                app.screen,
            ));
        }
        ctx.star_pop = (ctx.star_pop - ctx.dt * 2.0).max(0.0);
        games::fade(&mut ctx.button_pop, ctx.dt, 3.0);
        ctx.confetti.update(ctx.dt);
        ctx.voice.update().await;

        app.update(&mut ctx);
        app.draw(&ctx);

        if snapshot_done(ctx.time) {
            break;
        }
        next_frame().await;
    }
}
