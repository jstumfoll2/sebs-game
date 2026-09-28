//! Drawing: a big canvas to paint on. Colors down the left, brushes down the right, and a
//! choice of pages: a blank one, or an outline (a shape or a picture) to color in.
//! The outline is drawn on top of the paint, so coloring never covers the lines.
//! There are no right answers here, so no stars and no levels.

use super::{MiniGame, Progress};
use crate::art::{self, Paint, Thing};
use crate::ctx::Ctx;
use crate::pictures::{self, Picture};
use crate::render;
use macroquad::prelude::*;

/// Paint pots: the game's colors plus black and white.
const COLORS: [(&str, Color); 10] = [
    ("red", Color::new(0.84, 0.09, 0.09, 1.0)),
    ("orange", Color::new(0.98, 0.55, 0.0, 1.0)),
    ("yellow", Color::new(0.99, 0.85, 0.21, 1.0)),
    ("green", Color::new(0.26, 0.63, 0.28, 1.0)),
    ("blue", Color::new(0.12, 0.53, 0.9, 1.0)),
    ("purple", Color::new(0.56, 0.14, 0.67, 1.0)),
    ("pink", Color::new(0.94, 0.38, 0.57, 1.0)),
    ("brown", Color::new(0.47, 0.33, 0.28, 1.0)),
    ("black", Color::new(0.1, 0.1, 0.12, 1.0)),
    ("white", Color::new(1.0, 1.0, 1.0, 1.0)),
];

#[derive(Clone, Copy, PartialEq, Debug)]
enum Tool {
    Small,
    Medium,
    Big,
    Rainbow,
    Eraser,
    Clear,
    Pages,
}

const TOOLS: [Tool; 7] = [Tool::Small, Tool::Medium, Tool::Big, Tool::Rainbow, Tool::Eraser, Tool::Clear, Tool::Pages];

/// A coloring page: blank, a basic shape, or one of the alphabet pictures.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Page {
    Blank,
    Shape(Thing),
    Picture(Picture, &'static str),
}

const PAGES: [Page; 18] = [
    Page::Blank,
    Page::Shape(Thing::Circle),
    Page::Shape(Thing::Square),
    Page::Shape(Thing::Triangle),
    Page::Shape(Thing::Star),
    Page::Shape(Thing::Heart),
    Page::Shape(Thing::Diamond),
    Page::Picture(Picture::Apple, "apple"),
    Page::Picture(Picture::Fish, "fish"),
    Page::Picture(Picture::Cat, "cat"),
    Page::Picture(Picture::Sun, "sun"),
    Page::Picture(Picture::Tree, "tree"),
    Page::Picture(Picture::Umbrella, "umbrella"),
    Page::Picture(Picture::Octopus, "octopus"),
    Page::Picture(Picture::Whale, "whale"),
    Page::Picture(Picture::Pig, "pig"),
    Page::Picture(Picture::Kite, "kite"),
    Page::Picture(Picture::Van, "van"),
];

impl Page {
    fn name(self) -> &'static str {
        match self {
            Page::Blank => "blank page",
            Page::Shape(t) => t.name(),
            Page::Picture(_, name) => name,
        }
    }

    /// Draw the page's shape (in any color; only its outline gets used).
    fn draw(self, c: Vec2, s: f32) {
        match self {
            Page::Blank => {}
            Page::Shape(t) => art::draw_thing(t, c, s, BLACK),
            Page::Picture(p, _) => pictures::draw(p, c, s),
        }
    }
}

pub struct Drawing {
    progress: Progress,
    canvas: Option<RenderTarget>,
    page: Page,
    /// Page shapes, made once (for the outline and the page chooser).
    page_images: Vec<RenderTarget>,
    outline: Option<Material>,
    color: usize,
    tool: Tool,
    /// Where the finger was last frame while painting (in canvas pixels).
    last: Option<Vec2>,
    /// Showing the page chooser.
    choosing_page: bool,
    pop: f32,
}

impl Drawing {
    pub fn new() -> Self {
        Drawing {
            progress: Progress::new(1),
            canvas: None,
            page: Page::Blank,
            page_images: Vec::new(),
            outline: None,
            color: 4,
            tool: Tool::Medium,
            last: None,
            choosing_page: false,
            pop: 0.0,
        }
    }

    /// The painting area.
    fn canvas_rect() -> Rect {
        let (w, h) = (screen_width(), screen_height());
        Rect::new(w * 0.13, h * 0.16, w * 0.74, h * 0.8)
    }

    /// The paint pots: two columns down the left side.
    fn color_rects() -> Vec<Rect> {
        let (w, h) = (screen_width(), screen_height());
        let size = (w * 0.055).min(h * 0.14);
        (0..COLORS.len())
            .map(|i| {
                let (col, row) = ((i % 2) as f32, (i / 2) as f32);
                Rect::new(w * 0.01 + col * size * 1.05, h * 0.2 + row * size * 1.12, size, size)
            })
            .collect()
    }

    /// The brushes and buttons down the right side.
    fn tool_rects() -> Vec<Rect> {
        let (w, h) = (screen_width(), screen_height());
        let size = (w * 0.1).min(h * 0.105);
        (0..TOOLS.len())
            .map(|i| Rect::new(w * 0.885 + (w * 0.11 - size) / 2.0, h * 0.17 + i as f32 * size * 1.1, size, size))
            .collect()
    }

    /// The page chooser: a grid of little pages.
    fn page_rects() -> Vec<Rect> {
        let (w, h) = (screen_width(), screen_height());
        let cols = 6;
        let cell = (w * 0.8 / cols as f32).min(h * 0.7 / 3.0);
        let size = cell * 0.88;
        let x0 = (w - cell * cols as f32) / 2.0;
        (0..PAGES.len())
            .map(|i| {
                let (col, row) = ((i % cols) as f32, (i / cols) as f32);
                Rect::new(x0 + col * cell + (cell - size) / 2.0, h * 0.2 + row * cell, size, size)
            })
            .collect()
    }

    /// Where the page outline goes on the canvas (a centered square).
    fn outline_rect() -> Rect {
        let c = Self::canvas_rect();
        let side = c.w.min(c.h) * 0.96;
        Rect::new(c.center().x - side / 2.0, c.center().y - side / 2.0, side, side)
    }

    fn brush(&self, time: f32) -> (Color, f32) {
        let c = Self::canvas_rect();
        let unit = c.h / 100.0;
        match self.tool {
            Tool::Small => (COLORS[self.color].1, unit * 0.8),
            Tool::Big => (COLORS[self.color].1, unit * 5.0),
            Tool::Rainbow => (rainbow(time), unit * 2.5),
            Tool::Eraser => (WHITE, unit * 6.0),
            _ => (COLORS[self.color].1, unit * 2.2),
        }
    }

    fn set_page(&mut self, page: Page, ctx: &mut Ctx) {
        self.page = page;
        if let Some(c) = &self.canvas {
            render::clear_canvas(c);
        }
        match page {
            Page::Blank => ctx.voice.say("A blank page! Draw anything you like."),
            _ => ctx.voice.say(&format!("Let's color the {}!", page.name())),
        }
    }
}

/// A color that slowly moves around the rainbow.
fn rainbow(time: f32) -> Color {
    let h = (time * 0.4).fract();
    let f = |n: f32| {
        let k = (n + h * 6.0) % 6.0;
        1.0 - (k.min(4.0 - k).clamp(0.0, 1.0))
    };
    Color::new(f(5.0), f(3.0), f(1.0), 1.0)
}

impl MiniGame for Drawing {
    fn enter(&mut self, ctx: &mut Ctx) {
        self.choosing_page = false;
        ctx.voice.then("Let's draw! Pick a color and paint with your finger.");
    }

    fn prompt(&self) -> String {
        "Pick a color and paint with your finger!".to_string()
    }

    fn update(&mut self, ctx: &mut Ctx) {
        self.pop = (self.pop - ctx.dt * 3.0).max(0.0);
        // Make the canvas (sized in real pixels so lines are crisp) and the page images.
        let area = Self::canvas_rect();
        if self.canvas.is_none() {
            let dpi = screen_dpi_scale();
            self.canvas = Some(render::canvas((area.w * dpi) as u32, (area.h * dpi) as u32));
        }
        if self.page_images.is_empty() {
            self.page_images = PAGES.iter().map(|p| render::shape_texture(512, |c, s| p.draw(c, s))).collect();
            self.outline = render::outline_material();
            // Developer helper: `--page 7` starts on page 7 (0 is the blank page).
            let args: Vec<String> = std::env::args().collect();
            if let Some(n) = args.iter().position(|a| a == "--page").and_then(|i| args.get(i + 1)) {
                if let Some(page) = n.parse::<usize>().ok().and_then(|n| PAGES.get(n)) {
                    self.page = *page;
                }
            }
        }

        if self.choosing_page {
            if ctx.input.pressed {
                if let Some(i) = Self::page_rects().iter().position(|r| r.contains(ctx.input.pos)) {
                    ctx.sfx.pop();
                    self.set_page(PAGES[i], ctx);
                }
                self.choosing_page = false;
            }
            return;
        }

        let input = ctx.input;
        if input.pressed {
            if let Some(i) = Self::color_rects().iter().position(|r| r.contains(input.pos)) {
                self.color = i;
                if matches!(self.tool, Tool::Eraser | Tool::Rainbow) {
                    self.tool = Tool::Medium;
                }
                self.pop = 1.0;
                ctx.sfx.pop();
                ctx.voice.say(&format!("{}!", crate::alphabet::capitalize(COLORS[i].0)));
                return;
            }
            if let Some(i) = Self::tool_rects().iter().position(|r| r.contains(input.pos)) {
                ctx.sfx.pop();
                match TOOLS[i] {
                    Tool::Clear => {
                        if let Some(c) = &self.canvas {
                            render::clear_canvas(c);
                        }
                        ctx.voice.say("All clean!");
                    }
                    Tool::Pages => {
                        self.choosing_page = true;
                        ctx.voice.say("Pick a page!");
                    }
                    tool => {
                        self.tool = tool;
                        let words = match tool {
                            Tool::Small => "Small brush!",
                            Tool::Big => "Big brush!",
                            Tool::Rainbow => "Rainbow brush!",
                            Tool::Eraser => "Eraser!",
                            _ => "Medium brush!",
                        };
                        ctx.voice.say(words);
                    }
                }
                return;
            }
        }

        // Painting: a line from last frame's spot to this one, so fast strokes stay smooth.
        let mut input = input;
        if std::env::args().any(|a| a == "--fake-paint") {
            // Developer stress test: paint circles by itself.
            let t = ctx.time;
            input.down = true;
            input.pos = area.center() + vec2(t.cos() * area.w * 0.3, (t * 1.3).sin() * area.h * 0.3);
            if (t * 10.0) as i32 % 50 == 0 {
                crate::log::line(&format!("fps {}", get_fps()));
            }
        }
        let Some(canvas) = &self.canvas else { return };
        if input.down && area.contains(input.pos) {
            let (cw, ch) = (canvas.texture.width(), canvas.texture.height());
            let p = vec2((input.pos.x - area.x) / area.w * cw, (input.pos.y - area.y) / area.h * ch);
            let (color, size) = self.brush(ctx.time);
            let size = size * cw / area.w;
            let from = self.last.unwrap_or(p);
            render::with_canvas(canvas, || {
                draw_line(from.x, from.y, p.x, p.y, size * 2.0, color);
                draw_circle(p.x, p.y, size, color);
                draw_circle(from.x, from.y, size, color);
            });
            self.last = Some(p);
        } else {
            self.last = None;
        }
    }

    fn draw(&self, ctx: &Ctx) {
        let font = ctx.font();
        let area = Self::canvas_rect();
        art::card(Rect::new(area.x - 6.0, area.y - 6.0, area.w + 12.0, area.h + 12.0), WHITE);
        if let Some(c) = &self.canvas {
            render::draw_picture(&c.texture, area, Rect::new(0.0, 0.0, 1.0, 1.0), WHITE);
        }
        // The page's outline, on top of the paint.
        if let (Some(m), Some(i)) = (&self.outline, PAGES.iter().position(|p| *p == self.page)) {
            if self.page != Page::Blank {
                if let Some(img) = self.page_images.get(i) {
                    render::draw_with_material(m, &img.texture, Self::outline_rect(), COLORS[8].1);
                }
            }
        }

        // Paint pots.
        for (i, r) in Self::color_rects().iter().enumerate() {
            let picked = i == self.color && !matches!(self.tool, Tool::Eraser | Tool::Rainbow);
            let grow = if picked { 1.12 + 0.1 * (self.pop * std::f32::consts::PI).sin() } else { 1.0 };
            let r = art::scale_rect(*r, grow);
            if picked {
                art::glow(r, ctx.time);
            }
            draw_circle(r.center().x, r.center().y + r.h * 0.05, r.w * 0.46, art::SHADOW);
            draw_circle(r.center().x, r.center().y, r.w * 0.46, art::darken(COLORS[i].1, 0.25));
            draw_circle(r.center().x, r.center().y, r.w * 0.4, COLORS[i].1);
        }

        // Brushes and buttons.
        for (i, r) in Self::tool_rects().iter().enumerate() {
            let tool = TOOLS[i];
            if tool == self.tool {
                art::glow(*r, ctx.time);
            }
            art::card(*r, WHITE);
            let c = r.center();
            let paint = COLORS[self.color].1;
            match tool {
                Tool::Small => draw_circle(c.x, c.y, r.w * 0.08, paint),
                Tool::Medium => draw_circle(c.x, c.y, r.w * 0.16, paint),
                Tool::Big => draw_circle(c.x, c.y, r.w * 0.3, paint),
                Tool::Rainbow => {
                    for (k, p) in Paint::ALL.iter().take(6).enumerate() {
                        let a = k as f32 * std::f32::consts::TAU / 6.0 + ctx.time;
                        let d = vec2(a.cos(), a.sin()) * r.w * 0.2;
                        draw_circle(c.x + d.x, c.y + d.y, r.w * 0.1, p.color());
                    }
                }
                Tool::Eraser => {
                    let e = Rect::new(c.x - r.w * 0.3, c.y - r.h * 0.15, r.w * 0.6, r.h * 0.3);
                    art::rounded_rect(e, r.w * 0.06, Color::from_rgba(255, 170, 190, 255));
                    art::rounded_rect(Rect::new(e.x, e.y, e.w * 0.35, e.h), r.w * 0.06, Paint::Blue.color());
                }
                Tool::Clear => {
                    // A little broom-sweep: sparkles.
                    art::star(c, r.w * 0.28, art::GOLD);
                    art::text_center(font, "new", vec2(c.x, c.y + r.h * 0.32), r.h * 0.2, art::INK);
                }
                Tool::Pages => {
                    for k in 0..3 {
                        let o = k as f32 * r.w * 0.07;
                        let pg = Rect::new(c.x - r.w * 0.22 + o, c.y - r.h * 0.28 + o, r.w * 0.36, r.h * 0.44);
                        art::rounded_rect(pg, r.w * 0.04, art::SHADOW);
                        art::rounded_rect(art::scale_rect(pg, 0.9), r.w * 0.04, WHITE);
                    }
                    art::star(vec2(c.x + r.w * 0.04, c.y + r.h * 0.06), r.w * 0.1, Paint::Red.color());
                }
            }
        }

        // The page chooser, on top of everything.
        if self.choosing_page {
            let (w, h) = (screen_width(), screen_height());
            draw_rectangle(0.0, 0.0, w, h, Color::new(0.1, 0.1, 0.2, 0.45));
            art::text_center(font, "Pick a page", vec2(w / 2.0, h * 0.12), h * 0.08, WHITE);
            for (i, r) in Self::page_rects().iter().enumerate() {
                art::card(*r, WHITE);
                match (PAGES[i], &self.outline, self.page_images.get(i)) {
                    (Page::Blank, _, _) => {
                        art::text_center(font, "blank", r.center(), r.h * 0.2, art::SHADOW);
                    }
                    (_, Some(m), Some(img)) => {
                        render::draw_with_material(m, &img.texture, art::scale_rect(*r, 0.9), COLORS[8].1);
                    }
                    _ => {}
                }
            }
        }
    }

    fn progress(&self) -> &Progress {
        &self.progress
    }

    fn progress_mut(&mut self) -> &mut Progress {
        &mut self.progress
    }

    fn level_label(&self, _level: u32) -> String {
        "draw".to_string()
    }
}
