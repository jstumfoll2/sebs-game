// In release builds, don't open a console window next to the game.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod alphabet;
mod art;
mod assets;
mod banner;
mod ctx;
mod fx;
mod games;
mod hud;
mod input;
mod keyboard;
mod levels;
mod log;
mod menu;
mod pictures;
mod players;
mod render;
mod sfx;
mod stars;
mod versus;
mod voice;
mod wav;
mod who;
mod winner;

use ctx::Ctx;
use games::{
    color_sort::ColorSort, counting::Counting, drawing::Drawing, groups::Groups, letters::Letters, pattern::Pattern,
    puzzle::Puzzle, shadows::Shadows, spelling::Spelling, MiniGame,
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
    /// "Keep your stars, or start over?" for player number `usize`.
    KeepStars(usize),
    Menu,
    Levels(usize),
    Playing(usize),
    /// Setting up a two-player match.
    VersusSetup,
    /// Somebody won!
    Winner,
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
    picker: who::Picker,
    star_panel: stars::StarPanel,
    /// "Level 3 done!" while it's showing.
    banner: Option<banner::LevelBanner>,
    versus_setup: versus::VersusSetup,
    winner: Option<winner::Winner>,
}

impl App {
    /// The name we save each game's level under ("colors", "counting", ...).
    fn game_key(i: usize) -> String {
        menu::LABELS[i].to_lowercase()
    }

    /// Start playing as player `i`: bring back their stars and levels.
    /// A player was picked. If they have stars, ask whether to keep them first.
    fn choose_player(&mut self, i: usize, ctx: &mut Ctx) {
        let p = &self.players.players[i];
        if p.stars > 0 {
            ctx.voice.say(&format!(
                "Hi {}! You have {} stars. Do you want to keep them, or start over?",
                p.name, p.stars
            ));
            self.screen = Screen::KeepStars(i);
        } else {
            self.pick_player(i, false, ctx);
        }
    }

    /// Start playing as player `i`: bring back their stars (unless starting over) and levels.
    fn pick_player(&mut self, i: usize, start_over: bool, ctx: &mut Ctx) {
        let p = self.players.players[i].clone();
        ctx.name = p.name.clone();
        ctx.stars = if start_over { 0 } else { p.stars };
        for (g, game) in self.games.iter_mut().enumerate() {
            let key = Self::game_key(g);
            let level = p.levels.get(&key).copied().unwrap_or(1);
            game.progress_mut().set_level(level);
            game.progress_mut().completed = p.completed.get(&key).into_iter().flatten().copied().collect();
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
        let completed = (0..self.games.len())
            .filter(|&g| !self.games[g].progress().completed.is_empty())
            .map(|g| (Self::game_key(g), self.games[g].progress().completed.iter().copied().collect()))
            .collect();
        let now = players::Player { name: ctx.name.clone(), stars: ctx.stars, levels, completed };
        if self.players.players.get(i) != Some(&now) {
            self.players.players[i] = now;
            self.players.save();
        }
    }

    fn show_players(&mut self, ctx: &mut Ctx) {
        ctx.versus = None; // going back to "Who's playing?" ends any two-player match
        self.screen = Screen::Players;
        ctx.voice.say("Who's playing?");
    }

    fn new_name(&mut self, ctx: &mut Ctx) {
        self.screen = Screen::NewName;
        self.name_entry.start(ctx);
    }

    /// Take a player off the list for good.
    fn remove_player(&mut self, i: usize, ctx: &mut Ctx) {
        let name = self.players.players.remove(i).name;
        self.players.last_player = None;
        self.current = match self.current {
            Some(c) if c == i => None,
            Some(c) if c > i => Some(c - 1),
            other => other,
        };
        self.players.save();
        if self.players.players.is_empty() {
            self.new_name(ctx);
        } else {
            ctx.voice.say(&format!("Bye bye, {name}!"));
        }
    }

    /// Start a two-player match: first to `goal` stars wins. Player `a` goes first.
    fn start_versus(&mut self, a: usize, b: usize, goal: u32, ctx: &mut Ctx) {
        let names = [self.players.players[a].name.clone(), self.players.players[b].name.clone()];
        ctx.voice.say(&format!(
            "{} versus {}! First to {goal} stars wins. {} goes first! Pick a game!",
            names[0], names[1], names[0]
        ));
        ctx.name = names[0].clone();
        ctx.versus = Some(ctx::Match { names, colors: [a, b], scores: [0, 0], turn: 0, goal });
        self.current = None; // match scores are separate: nobody's saved stars change
        self.menu = menu::Menu::new();
        self.screen = Screen::Menu;
    }

    fn picking_player(&self) -> bool {
        matches!(
            self.screen,
            Screen::Players | Screen::NewName | Screen::KeepStars(_) | Screen::VersusSetup | Screen::Winner
        )
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
        if !self.picking_player() && ctx.versus.is_none() && self.star_panel.update(ctx) {
            return;
        }

        match self.screen {
            Screen::Players => match self.picker.update(&self.players, ctx) {
                who::Pick::Player(i) => {
                    ctx.sfx.pop();
                    self.choose_player(i, ctx);
                }
                who::Pick::AddNew => {
                    ctx.sfx.pop();
                    self.new_name(ctx);
                }
                who::Pick::Remove(i) => self.remove_player(i, ctx),
                who::Pick::Versus => {
                    self.versus_setup.start(ctx);
                    self.screen = Screen::VersusSetup;
                }
                who::Pick::Nothing => {}
            },
            Screen::VersusSetup => match self.versus_setup.update(&self.players, ctx) {
                versus::Setup::Start { first, second, goal } => self.start_versus(first, second, goal, ctx),
                versus::Setup::Back => self.show_players(ctx),
                versus::Setup::Nothing => {}
            },
            Screen::Winner => {
                if let Some(w) = &mut self.winner {
                    if w.update(ctx) {
                        let was_versus = w.is_versus();
                        self.winner = None;
                        if was_versus {
                            self.show_players(ctx);
                        } else {
                            ctx.voice.say("Let's keep playing! Pick a game!");
                            self.screen = Screen::Menu;
                        }
                    }
                }
            }
            Screen::KeepStars(i) => match who::update_star_choice(ctx) {
                who::StarChoice::Keep => {
                    ctx.sfx.pop();
                    self.pick_player(i, false, ctx);
                }
                who::StarChoice::StartOver => {
                    ctx.sfx.pop();
                    self.pick_player(i, true, ctx);
                }
                who::StarChoice::Nothing => {}
            },
            Screen::NewName => {
                let can_cancel = !self.players.players.is_empty();
                match self.name_entry.update(ctx, can_cancel) {
                    who::Typed::Done(name) => {
                        let i = self.players.add(&name);
                        self.pick_player(i, false, ctx);
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
                    if self.games[i].progress().max == 1 {
                        // Games with only one level (like Drawing) skip the level picker.
                        self.games[i].enter(ctx);
                        self.screen = Screen::Playing(i);
                    } else {
                        ctx.voice.then("Pick a level!");
                        self.screen = Screen::Levels(i);
                    }
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
                // Just finished a level? Show the banner (and fireworks).
                let max = self.games[i].progress().max;
                if let Some(done) = self.games[i].progress_mut().take_finished() {
                    self.banner = Some(banner::LevelBanner::new(done, max, ctx));
                }
            }
        }
        if let Some(b) = &mut self.banner {
            if !b.update(ctx.dt) {
                self.banner = None;
            }
        }
        // Somebody won? On to the winner screen.
        if let Some(win) = ctx.win.take() {
            self.banner = None;
            self.winner = Some(winner::Winner::new(win, ctx));
            self.screen = Screen::Winner;
        }
        self.save_progress(ctx);
    }

    fn draw(&self, ctx: &Ctx) {
        art::background(ctx.time);
        match self.screen {
            Screen::Players => self.picker.draw(&self.players, ctx),
            Screen::VersusSetup => self.versus_setup.draw(&self.players, ctx),
            Screen::Winner => {
                if let Some(w) = &self.winner {
                    w.draw(ctx);
                }
            }
            Screen::KeepStars(i) => {
                let p = &self.players.players[i];
                who::draw_star_choice(&p.name, p.stars, ctx);
            }
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
                let p = self.games[i].progress();
                hud::draw_game_buttons(ctx, (p.max > 1).then_some(p.level));
            }
        }
        if !self.picking_player() {
            match &ctx.versus {
                Some(m) => hud::draw_scores(ctx, m),
                None => hud::draw_stars(ctx),
            }
        }
        hud::draw_mute_button(ctx);
        if let Some(b) = &self.banner {
            b.draw(ctx);
        }
        ctx.confetti.draw();
        ctx.fireworks.draw();
        self.star_panel.draw(ctx);
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    log::start();
    rand::srand(macroquad::miniquad::date::now() as u64);

    let mut ctx = Ctx {
        input: input::Input::default(),
        font: art::load_font(),
        sfx: sfx::Sfx::load().await,
        voice: voice::Voice::new(),
        confetti: fx::Confetti::default(),
        fireworks: fx::Fireworks::default(),
        name: "friend".to_string(),
        stars: 0,
        star_pop: 0.0,
        time: 0.0,
        dt: 0.0,
        muted: false,
        button_pop: [0.0; 3],
        last_mute_toggle: -1.0,
        versus: None,
        win: None,
    };

    let mut app = App {
        screen: Screen::Menu,
        games: vec![
            Box::new(ColorSort::new()),
            Box::new(Pattern::new()),
            Box::new(Shadows::new()),
            Box::new(Puzzle::new()),
            Box::new(Letters::new()),
            Box::new(Spelling::new()),
            Box::new(Counting::new()),
            Box::new(Groups::new()),
            Box::new(Drawing::new()),
        ],
        menu: menu::Menu::new(),
        players: players::Players::load(),
        current: None,
        name_entry: who::NameEntry::default(),
        picker: who::Picker::default(),
        star_panel: stars::StarPanel::default(),
        banner: None,
        versus_setup: versus::VersusSetup::default(),
        winner: None,
    };
    let mut taps = input::TapFilter::default();

    // Get common phrases ready in the background so they play instantly later.
    let mut common: Vec<String> = ctx::PRAISE.iter().filter(|s| !s.contains("NAME")).map(|s| s.to_string()).collect();
    common.extend(games::counting::NUMBER_WORDS.iter().map(|s| s.to_string()));
    common.extend(art::Paint::ALL.iter().map(|p| p.name().to_string()));
    common.extend(["Pick a game!", "Pick a level!"].map(String::from));
    ctx.voice.prepare(&common);

    // `cargo run -- --things` shows every object and shape the games use, in all the colors.
    if std::env::args().any(|a| a == "--things") {
        loop {
            art::background(get_time() as f32);
            let (w, h) = (screen_width(), screen_height());

            let cell = (w / 6.0).min(h / 2.2);
            for (i, t) in art::Thing::ALL.iter().enumerate() {
                let (col, row) = ((i % 6) as f32, (i / 6) as f32);
                let r = Rect::new(col * cell + cell * 0.05, row * cell * 1.05 + cell * 0.05, cell * 0.9, cell * 0.9);
                art::card(r, WHITE);
                let paint = art::Paint::ALL[i % art::Paint::ALL.len()];
                art::draw_thing(*t, vec2(r.center().x, r.y + r.h * 0.42), r.h * 0.26, paint.color());
                art::word_label(ctx.font(), t.name(), vec2(r.center().x, r.y + r.h * 0.86), r.h * 0.1, r.w * 0.9, art::readable(paint.color()), None);
            }

            if is_key_pressed(KeyCode::Escape) || snapshot_done(get_time() as f32) {
                return;
            }
            next_frame().await;
        }
    }

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

    // Start on "Who's playing?" (or ask for a name if nobody has played yet).
    if app.players.players.is_empty() {
        app.new_name(&mut ctx);
    } else {
        app.show_players(&mut ctx);
    }

    // `--pick 0` picks player 0; `--edit-players` opens the remove-a-player mode.
    if let Some(i) = arg_value("--pick").and_then(|s| s.parse::<usize>().ok()) {
        if i < app.players.players.len() {
            app.choose_player(i, &mut ctx);
        }
    }
    if std::env::args().any(|a| a == "--edit-players") {
        app.picker.start_editing();
    }

    // `--stars 7` starts with 7 stars; `--show-stars` opens the star panel right away.
    if let Some(n) = arg_value("--stars").and_then(|s| s.parse().ok()) {
        ctx.stars = n;
    }
    if std::env::args().any(|a| a == "--show-stars") {
        app.star_panel.open(&mut ctx);
    }

    // `--versus 0:1:10` starts a match between players 0 and 1, first to 10 stars.
    if let Some(v) = arg_value("--versus") {
        let n: Vec<usize> = v.split(':').filter_map(|s| s.parse().ok()).collect();
        if n.len() == 3 && n[0].max(n[1]) < app.players.players.len() {
            app.start_versus(n[0], n[1], n[2] as u32, &mut ctx);
        }
    }
    // `--demo-win` shows the winner screen (a pretend 10-7 match, or 50 stars solo).
    if std::env::args().any(|a| a == "--demo-win") {
        ctx.win = Some(match &ctx.versus {
            Some(m) => ctx::Win::Versus {
                winner: m.names[0].clone(),
                winner_stars: 10,
                other: m.names[1].clone(),
                other_stars: 7,
                colors: m.colors,
            },
            None => ctx::Win::Solo { name: ctx.name.clone(), stars: 50 },
        });
    }

    // `--banner` shows the "Level 2 done!" banner (to check how it looks).
    if std::env::args().any(|a| a == "--banner") {
        app.banner = Some(banner::LevelBanner::new(2, 5, &mut ctx));
    }

    // `--levels 0` opens game 0's level picker.
    if let Some(i) = arg_value("--levels").and_then(|s| s.parse::<usize>().ok()) {
        app.screen = Screen::Levels(i.min(app.games.len() - 1));
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
        log::step("reading taps");
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
        ctx.fireworks.update(ctx.dt);
        log::step("voice");
        ctx.voice.update().await;

        log::step("game logic");
        app.update(&mut ctx);
        log::step("drawing");
        app.draw(&ctx);

        if snapshot_done(ctx.time) {
            break;
        }
        log::step("showing the frame");
        next_frame().await;
        log::frame_done();
    }
}
