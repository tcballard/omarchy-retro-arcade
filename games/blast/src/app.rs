use crate::{
    rules::*,
    sound::Sound,
    storage::{self, Settings},
};
use eframe::egui::{self, Align2, Color32, FontId, Key, Pos2, Rect, RichText, Stroke, Vec2};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
const COLOURS: [Color32; 4] = [
    Color32::from_rgb(96, 217, 198),
    Color32::from_rgb(255, 179, 90),
    Color32::from_rgb(176, 160, 250),
    Color32::from_rgb(245, 128, 155),
];
const NAMES: [&str; 4] = ["SCOUT", "ROOK", "SPARK", "PATCH"];
pub struct App {
    pub game: Match,
    settings: Settings,
    dir: PathBuf,
    paused: bool,
    setup: bool,
    settings_open: bool,
    rebind: Option<(usize, usize)>,
    error: Option<String>,
    writable: bool,
    leave: bool,
    accumulator: f64,
    last: Instant,
    sound: Sound,
    theme: omarchy_chess::theme::Theme,
    themed: Instant,
    recorded: bool,
    pending_bombs: [bool; 4],
}
impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
impl App {
    pub fn prepare_to_leave(&mut self) -> Result<(), String> {
        self.leave = false;
        self.suspend();
        if !self.writable {
            return Err("The original Blast save needs recovery before saving is allowed.".into());
        }
        self.settings
            .save(&self.dir)
            .map_err(|e| format!("Could not save Blast preferences: {e}"))
    }
    pub fn new() -> Self {
        Self::from_dir(storage::state_dir())
    }
    fn from_dir(dir: PathBuf) -> Self {
        let (settings, error) = match Settings::load(&dir) {
            Ok(s) => (s, None),
            Err(e) => (
                Settings::default(),
                Some(format!("Could not load Blast records: {e}. Saving is disabled to protect the original. Preserve or repair blast.json, then reopen Blast.")),
            ),
        };
        let writable = error.is_none();
        let game = Match::new(settings.arena, settings.humans, settings.bots);
        Self {
            game,
            settings,
            dir,
            paused: true,
            setup: true,
            settings_open: false,
            rebind: None,
            error,
            writable,
            leave: false,
            accumulator: 0.,
            last: Instant::now(),
            sound: Sound::default(),
            theme: omarchy_chess::theme::Theme::load(),
            themed: Instant::now(),
            recorded: false,
            pending_bombs: [false; 4],
        }
    }
    pub fn finished(&self) -> bool {
        self.leave
    }
    pub fn suspend(&mut self) {
        self.paused = true;
        self.pending_bombs = [false; 4];
        self.accumulator = 0.;
        self.sound.stop();
    }
    fn persist(&mut self) {
        if !self.writable {
            return;
        }
        if let Err(e) = self.settings.save(&self.dir) {
            self.error = Some(format!("Could not save Blast preferences: {e}"));
        }
    }
    fn start(&mut self) {
        self.game = Match::new(
            self.settings.arena,
            self.settings.humans,
            self.settings.bots,
        );
        self.paused = false;
        self.setup = false;
        self.recorded = false;
        self.settings.seen_help = true;
        self.pending_bombs = [false; 4];
        self.accumulator = 0.;
        self.last = Instant::now();
        self.sound.stop();
        self.persist();
    }
    fn controls(&self, ctx: &egui::Context) -> [Input; 4] {
        let mut result = [Input::default(); 4];
        ctx.input(|i| {
            if i.modifiers.ctrl || i.modifiers.alt || i.modifiers.command {
                return;
            }
            for (id, action) in result.iter_mut().enumerate().take(self.settings.humans) {
                let keys = self.settings.bindings[id]
                    .each_ref()
                    .map(|s| Key::from_name(s).expect("validated bindings"));
                // Opposite directions cancel; vertical/horizontal diagonal input is explicit vertical priority.
                let v = i.key_down(keys[0]) as i8 - i.key_down(keys[1]) as i8;
                let h = i.key_down(keys[2]) as i8 - i.key_down(keys[3]) as i8;
                action.direction = match (v, h) {
                    (1, _) => Some(Direction::Up),
                    (-1, _) => Some(Direction::Down),
                    (0, 1) => Some(Direction::Left),
                    (0, -1) => Some(Direction::Right),
                    _ => None,
                };
                action.bomb = i.key_pressed(keys[4]);
            }
        });
        result
    }
    fn header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("BLAST").size(32.).strong());
            ui.label(RichText::new(" / ").weak());
            ui.label(ARENAS[self.game.arena.arena]);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Settings").clicked() {
                    self.settings_open = true;
                    self.suspend();
                }
                if ui.button("Pause  Esc").clicked() {
                    self.suspend();
                }
                let tick = self.game.arena.tick;
                if tick < ROUND {
                    let seconds = (ROUND - tick).div_ceil(HZ);
                    ui.label(
                        RichText::new(format!("{:02}:{:02}", seconds / 60, seconds % 60))
                            .monospace()
                            .size(25.),
                    );
                } else {
                    ui.label(RichText::new("SUDDEN DEATH").color(COLOURS[1]).strong());
                }
            });
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(format!("ROUND {:02}  /  FIRST TO THREE", self.game.rounds))
                    .monospace()
                    .small(),
            );
            ui.separator();
            for (id, player) in self.game.arena.players.iter().enumerate() {
                let colour = if player.alive {
                    if self.theme.light() {
                        let c = COLOURS[id];
                        Color32::from_rgb(c.r() / 2, c.g() / 2, c.b() / 2)
                    } else {
                        COLOURS[id]
                    }
                } else {
                    Color32::GRAY
                };
                ui.label(
                    RichText::new(format!(
                        "{} {}  {} / 3",
                        id + 1,
                        NAMES[id],
                        self.game.wins[id]
                    ))
                    .color(colour)
                    .strong(),
                );
                ui.add_space(8.);
            }
        });
    }
    fn board(&self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let cell = ((available.x - 26.) / W as f32)
            .min((available.y - 26.) / H as f32)
            .max(1.);
        let size = Vec2::new(cell * W as f32, cell * H as f32);
        let (_, area) = ui.allocate_space(available);
        let rect = Rect::from_center_size(area.center(), size);
        arcade_presentation::bezel(ui.painter(), rect, self.theme.accent);
        let painter = ui.painter_at(rect);
        let a = &self.game.arena;
        let tile_rect = |p: usize| {
            Rect::from_min_size(
                rect.min + Vec2::new((p % W) as f32 * cell, (p / W) as f32 * cell),
                Vec2::splat(cell),
            )
        };
        painter.rect_filled(rect, 8., Color32::from_rgb(12, 19, 27));
        for p in 0..N {
            let r = tile_rect(p).shrink(cell * 0.025);
            match a.tiles[p] {
                Tile::Wall => {
                    let rim = Arena::ring(p) == 0;
                    painter.rect_filled(
                        r.translate(Vec2::new(0., cell * 0.035)),
                        cell * 0.09,
                        Color32::from_rgb(6, 11, 18),
                    );
                    painter.rect_filled(
                        r.shrink(cell * 0.03),
                        cell * 0.07,
                        if rim {
                            Color32::from_rgb(36, 48, 59)
                        } else {
                            Color32::from_rgb(51, 67, 79)
                        },
                    );
                    painter.line_segment(
                        [
                            r.left_top() + Vec2::splat(cell * 0.09),
                            r.right_top() + Vec2::new(-cell * 0.09, cell * 0.09),
                        ],
                        Stroke::new(1_f32, Color32::from_rgb(88, 108, 116)),
                    );
                    arcade_presentation::tile(
                        &painter,
                        r.shrink(cell * 0.035),
                        if rim {
                            Color32::from_rgb(47, 56, 55)
                        } else {
                            Color32::from_rgb(65, 78, 77)
                        },
                        false,
                    );
                    if !rim {
                        for dx in [-0.27, 0.27] {
                            painter.circle_filled(
                                r.center() + Vec2::new(dx * cell, 0.),
                                cell * 0.035,
                                Color32::from_rgb(26, 36, 46),
                            );
                        }
                    }
                }
                Tile::Floor | Tile::Crate => {
                    painter.rect_filled(
                        r,
                        2.,
                        if (p % W + p / W).is_multiple_of(2) {
                            Color32::from_rgb(21, 32, 42)
                        } else {
                            Color32::from_rgb(25, 37, 46)
                        },
                    );
                    painter.circle_filled(r.center(), cell * 0.025, Color32::from_rgb(54, 67, 73));
                    if a.tiles[p] == Tile::Crate {
                        let c = r.shrink(cell * 0.095);
                        painter.rect_filled(
                            c.translate(Vec2::new(0., cell * 0.04)),
                            cell * 0.08,
                            Color32::from_rgb(9, 15, 20),
                        );
                        arcade_presentation::tile(
                            &painter,
                            c,
                            Color32::from_rgb(139, 88, 55),
                            false,
                        );
                        painter.rect_filled(
                            c.shrink(cell * 0.08),
                            cell * 0.025,
                            Color32::from_rgb(101, 62, 42),
                        );
                        for t in [0.28, 0.5, 0.72] {
                            let y = c.top() + c.height() * t;
                            painter.line_segment(
                                [
                                    Pos2::new(c.left() + cell * 0.09, y),
                                    Pos2::new(c.right() - cell * 0.09, y),
                                ],
                                Stroke::new(cell * 0.035, Color32::from_rgb(150, 96, 58)),
                            );
                        }
                        painter.line_segment(
                            [
                                c.left_bottom() + Vec2::new(cell * 0.1, -cell * 0.1),
                                c.right_top() + Vec2::new(-cell * 0.1, cell * 0.1),
                            ],
                            Stroke::new(cell * 0.075, Color32::from_rgb(186, 131, 77)),
                        );
                    }
                }
            }
            if a.warning(p) {
                painter.rect_stroke(
                    r.shrink(2.),
                    1.,
                    Stroke::new(2_f32, COLOURS[1]),
                    egui::StrokeKind::Inside,
                );
                painter.line_segment(
                    [r.left_bottom(), r.right_top()],
                    Stroke::new(1_f32, COLOURS[1].gamma_multiply(0.4)),
                );
            }
            if let Some(upgrade) = a.upgrades[p] {
                let c = r.center();
                painter.rect_filled(
                    Rect::from_center_size(c, Vec2::splat(cell * 0.6)),
                    cell * 0.12,
                    Color32::from_rgb(74, 210, 177),
                );
                painter.text(
                    c,
                    Align2::CENTER_CENTER,
                    if upgrade == Upgrade::Capacity {
                        "+"
                    } else {
                        "↔"
                    },
                    FontId::proportional(cell * 0.48),
                    Color32::from_rgb(9, 30, 34),
                );
            }
        }
        // Subtle dashed cross shows actual reach, clipped by current walls and crates.
        for bomb in &a.bombs {
            for p in Arena::blast(&a.tiles, bomb.pos, bomb.range) {
                let r = tile_rect(p);
                painter.circle_stroke(
                    r.center(),
                    cell * 0.08,
                    Stroke::new(1_f32, COLOURS[bomb.owner].gamma_multiply(0.55)),
                );
            }
        }
        for (p, expiry) in a.flames.iter().enumerate() {
            if *expiry <= a.tick {
                continue;
            }
            let r = tile_rect(p).shrink(cell * 0.07);
            painter.rect_filled(r, cell * 0.14, Color32::from_rgb(196, 82, 38));
            painter.rect_filled(
                r.shrink(cell * 0.1),
                cell * 0.18,
                Color32::from_rgb(250, 170, 61),
            );
            painter.circle_filled(r.center(), cell * 0.18, Color32::from_rgb(255, 231, 163));
            if !self.settings.reduced_motion {
                let t = (*expiry - a.tick) as f32 / FLAME as f32;
                painter.circle_stroke(
                    r.center(),
                    cell * (0.17 + (1. - t) * 0.2),
                    Stroke::new(cell * 0.035, Color32::from_rgb(255, 218, 124)),
                );
            }
        }
        for bomb in &a.bombs {
            let r = tile_rect(bomb.pos);
            let c = r.center();
            painter.circle_filled(
                c + Vec2::new(0., cell * 0.12),
                cell * 0.32,
                Color32::from_black_alpha(110),
            );
            painter.circle_filled(c, cell * 0.32, Color32::from_rgb(9, 14, 22));
            painter.circle_stroke(
                c,
                cell * 0.32,
                Stroke::new(cell * 0.05, COLOURS[bomb.owner]),
            );
            painter.circle_filled(
                c + Vec2::new(-cell * 0.1, -cell * 0.12),
                cell * 0.08,
                Color32::from_rgb(84, 98, 105),
            );
            let remain = bomb.due.saturating_sub(a.tick) as f32 / FUSE as f32;
            let points = (0..=30)
                .map(|i| {
                    let angle = -std::f32::consts::FRAC_PI_2
                        + std::f32::consts::TAU * remain * i as f32 / 30.;
                    c + Vec2::angled(angle) * cell * 0.4
                })
                .collect();
            painter.add(egui::Shape::line(
                points,
                Stroke::new(cell * 0.05, Color32::from_rgb(255, 219, 135)),
            ));
            painter.text(
                c + Vec2::new(0., cell * 0.08),
                Align2::CENTER_CENTER,
                bomb.due.saturating_sub(a.tick).div_ceil(HZ).to_string(),
                FontId::monospace(cell * 0.25),
                Color32::WHITE,
            );
        }
        for (id, player) in a.players.iter().enumerate() {
            let r = tile_rect(player.pos);
            let c = r.center();
            let colour = COLOURS[id];
            if !player.alive {
                painter.line_segment(
                    [c - Vec2::splat(cell * 0.14), c + Vec2::splat(cell * 0.14)],
                    Stroke::new(2_f32, colour.gamma_multiply(0.5)),
                );
                painter.line_segment(
                    [
                        c + Vec2::new(-cell * 0.14, cell * 0.14),
                        c + Vec2::new(cell * 0.14, -cell * 0.14),
                    ],
                    Stroke::new(2_f32, colour.gamma_multiply(0.5)),
                );
                continue;
            }
            painter.circle_filled(
                c + Vec2::new(0., cell * 0.24),
                cell * 0.29,
                Color32::from_black_alpha(120),
            );
            let body = Rect::from_center_size(
                c + Vec2::new(0., cell * 0.08),
                Vec2::new(cell * 0.47, cell * 0.5),
            );
            painter.rect_filled(body, cell * 0.12, colour.gamma_multiply(0.65));
            for dx in [-0.15, 0.15] {
                painter.rect_filled(
                    Rect::from_center_size(
                        c + Vec2::new(dx * cell, cell * 0.32),
                        Vec2::new(cell * 0.18, cell * 0.14),
                    ),
                    2.,
                    Color32::from_rgb(12, 18, 27),
                );
            }
            let head = c - Vec2::new(0., cell * 0.15);
            painter.circle_filled(head, cell * 0.3, colour);
            // Four silhouettes: antenna, square ears, twin fins, side patches.
            match id {
                0 => {
                    painter.line_segment(
                        [head, head - Vec2::new(0., cell * 0.34)],
                        Stroke::new(cell * 0.055, colour),
                    );
                    painter.circle_filled(
                        head - Vec2::new(0., cell * 0.34),
                        cell * 0.065,
                        Color32::WHITE,
                    );
                }
                1 => {
                    for dx in [-0.23, 0.23] {
                        painter.rect_filled(
                            Rect::from_center_size(
                                head + Vec2::new(dx * cell, -cell * 0.17),
                                Vec2::splat(cell * 0.18),
                            ),
                            1.,
                            colour,
                        );
                    }
                }
                2 => {
                    for dx in [-0.24, 0.24] {
                        painter.add(egui::Shape::convex_polygon(
                            vec![
                                head + Vec2::new(dx * cell, 0.),
                                head + Vec2::new(dx * cell * 1.5, -cell * 0.38),
                                head - Vec2::new(0., cell * 0.15),
                            ],
                            colour,
                            Stroke::NONE,
                        ));
                    }
                }
                _ => {
                    for dx in [-0.31, 0.31] {
                        painter.circle_filled(
                            head + Vec2::new(dx * cell, 0.),
                            cell * 0.105,
                            colour,
                        );
                    }
                }
            }
            painter.rect_filled(
                Rect::from_center_size(
                    head + Vec2::new(0., cell * 0.025),
                    Vec2::new(cell * 0.42, cell * 0.18),
                ),
                cell * 0.07,
                Color32::from_rgb(14, 31, 43),
            );
            for dx in [-0.105, 0.105] {
                painter.circle_filled(
                    head + Vec2::new(dx * cell, cell * 0.02),
                    cell * 0.036,
                    Color32::WHITE,
                );
            }
            painter.text(
                c + Vec2::new(0., cell * 0.15),
                Align2::CENTER_CENTER,
                (id + 1).to_string(),
                FontId::monospace(cell * 0.22),
                Color32::from_rgb(8, 23, 31),
            );
        }
    }
    fn dialogs(&mut self, ctx: &egui::Context) {
        if self.settings_open {
            egui::Window::new("Blast settings")
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label("Changes to controls apply immediately. Match setup is on New match.");
                    let changed = ui
                        .checkbox(&mut self.settings.sound, "Sound  ·  Ctrl+M")
                        .changed()
                        | ui.checkbox(&mut self.settings.reduced_motion, "Reduce motion")
                            .changed();
                    ui.label("Sound uses the desktop audio service (paplay).");
                    if changed {
                        if !self.settings.sound {
                            self.sound.stop();
                        }
                        self.persist();
                    }
                    ui.separator();
                    for (player, colour) in COLOURS.iter().enumerate().take(2) {
                        ui.label(RichText::new(format!("PLAYER {}", player + 1)).color(*colour));
                        ui.horizontal(|ui| {
                            for (action, name) in
                                ["Up", "Down", "Left", "Right", "Bomb"].iter().enumerate()
                            {
                                if ui
                                    .button(format!(
                                        "{}\n{}",
                                        name, self.settings.bindings[player][action]
                                    ))
                                    .clicked()
                                {
                                    self.rebind = Some((player, action));
                                }
                            }
                        });
                    }
                    if self.rebind.is_some() {
                        ui.label(
                            "Press a new key. Escape cancels. All ten bindings must be distinct.",
                        );
                    }
                    if ui.button("Done").clicked() {
                        self.settings_open = false;
                        self.rebind = None;
                        self.persist();
                    }
                });
        } else if self.setup {
            let start_key = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter));
            egui::Window::new("Ready in the yard").collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER, Vec2::ZERO).default_width(390.).show(ctx, |ui| {
                ui.heading("Make room. Leave an exit.");
                ui.label("Place bombs to open the arena. Step away before the fuse runs out. Last survivor takes the round; three rounds win the match.");
                ui.add_space(10.);
                ui.horizontal(|ui| {
                    ui.label("Players");
                    if ui.selectable_value(&mut self.settings.humans, 1, "Solo").changed() { self.settings.bots = 3; }
                    if ui.selectable_value(&mut self.settings.humans, 2, "Two local").changed() { self.settings.bots = 2; }
                });
                if self.settings.humans == 2 {
                    ui.horizontal(|ui| { ui.label("Bots"); for n in 0..=2 { ui.selectable_value(&mut self.settings.bots, n, n.to_string()); } });
                }
                egui::ComboBox::from_label("Arena").selected_text(ARENAS[self.settings.arena]).show_ui(ui, |ui| { for (i, name) in ARENAS.iter().enumerate() { ui.selectable_value(&mut self.settings.arena, i, *name); } });
                ui.add_space(8.);
                for id in 0..self.settings.humans { ui.label(format!("P{}  {} · {} · {} · {}    Bomb: {}", id + 1, self.settings.bindings[id][0], self.settings.bindings[id][1], self.settings.bindings[id][2], self.settings.bindings[id][3], self.settings.bindings[id][4])); }
                ui.label("+ adds a bomb slot. ↔ extends reach. Dotted tiles show bomb reach; the ring is its fuse. Gold hatching warns of closing walls after 2 minutes.");
                ui.label("Esc pauses everyone. Losing focus pauses until you resume.");
                ui.add_space(10.);
                if ui.add_sized([ui.available_width(), 40.], egui::Button::new("Start match  ·  Enter")).clicked() || start_key { self.start(); }
                ui.horizontal(|ui| { if ui.button("Controls / settings").clicked() { self.settings_open = true; }
                    if ui.button("Return to Arcade").clicked() { self.leave = true; } });
                ui.label(RichText::new(format!("LOCAL RECORD  {} completed matches · P1 won {}", self.settings.matches, self.settings.wins[0])).small().weak());
            });
        } else if self.paused {
            egui::Window::new("Match paused")
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label("All fuses, players and the round clock are stopped.");
                    if ui.button("Resume").clicked() {
                        self.paused = false;
                        self.last = Instant::now();
                    }
                    if ui.button("Restart match").clicked() {
                        self.start();
                    }
                    if ui.button("New match / arena").clicked() {
                        self.setup = true;
                    }
                    if ui.button("Return to Arcade").clicked() {
                        self.leave = true;
                        self.sound.stop();
                    }
                });
        } else if let Some(outcome) = self.game.arena.outcome {
            let next_key = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter));
            let title = if let Some(id) = self.game.champion {
                format!("{} wins the match", NAMES[id])
            } else {
                match outcome {
                    Outcome::Winner(id) => format!("{} takes the round", NAMES[id]),
                    Outcome::Draw => "Draw. Everyone went together.".into(),
                }
            };
            egui::Window::new(title)
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    if self.game.champion.is_some() {
                        ui.label("Three wins. Nicely done.");
                        if ui.button("Play again").clicked() {
                            self.start();
                        }
                        if ui.button("New match / arena").clicked() {
                            self.setup = true;
                        }
                    } else if ui.button("Next round  ·  Enter").clicked() || next_key {
                        self.game.next_round();
                        self.pending_bombs = [false; 4];
                        self.accumulator = 0.;
                        self.last = Instant::now();
                    }
                    if ui.button("Return to Arcade").clicked() {
                        self.leave = true;
                        self.sound.stop();
                    }
                });
        }
        if let Some(error) = self.error.clone() {
            egui::Window::new("Blast notice")
                .anchor(Align2::CENTER_BOTTOM, Vec2::new(0., -15.))
                .show(ctx, |ui| {
                    ui.label(error);
                    if ui.button("Dismiss").clicked() {
                        self.error = None;
                    }
                });
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        arcade_presentation::apply(ctx);
        if self.themed.elapsed() > Duration::from_secs(2) {
            self.theme = omarchy_chess::theme::Theme::load();
            self.themed = Instant::now();
        }
        let light = self.theme.background.r() as u32
            + self.theme.background.g() as u32
            + self.theme.background.b() as u32
            > 400;
        let mut visuals = if light {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };
        visuals.panel_fill = self.theme.background;
        visuals.window_fill = self.theme.background;
        visuals.override_text_color = Some(self.theme.foreground);
        visuals.selection.bg_fill = self.theme.accent;
        ctx.set_visuals(visuals);
        arcade_presentation::apply(ctx);
        let now = Instant::now();
        let elapsed = now.duration_since(self.last).as_secs_f64().min(0.1);
        self.last = now;
        if !ctx.input(|i| i.focused) {
            self.suspend();
        }
        if let Some((player, action)) = self.rebind {
            let key = ctx.input(|i| {
                i.events.iter().find_map(|event| match event {
                    egui::Event::Key {
                        key,
                        pressed: true,
                        repeat: false,
                        ..
                    } => Some(*key),
                    _ => None,
                })
            });
            if let Some(key) = key {
                if key != Key::Escape {
                    let mut candidate = self.settings.clone();
                    candidate.bindings[player][action] = key.name().into();
                    match candidate.validate() {
                        Ok(()) => {
                            self.settings = candidate;
                            self.persist();
                            self.error = None;
                        }
                        Err(e) => self.error = Some(e),
                    }
                }
                self.rebind = None;
                ctx.input_mut(|i| i.events.clear());
            }
        } else {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape)) {
                if self.settings_open {
                    self.settings_open = false;
                } else if self.paused && !self.setup {
                    self.paused = false;
                } else {
                    self.suspend();
                }
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, Key::Comma)) {
                self.settings_open = true;
                self.suspend();
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, Key::M)) {
                self.settings.sound = !self.settings.sound;
                self.sound.stop();
                self.persist();
            }
        }
        if !self.paused && !self.setup && !self.settings_open && self.game.arena.outcome.is_none() {
            let mut human = self.controls(ctx);
            self.accumulator += elapsed;
            for (id, action) in human.iter_mut().enumerate() {
                self.pending_bombs[id] |= action.bomb;
                action.bomb = self.pending_bombs[id];
            }
            while self.accumulator >= 1. / HZ as f64 {
                self.accumulator -= 1. / HZ as f64;
                let mut input = self.game.arena.bot_inputs();
                input[..self.settings.humans].copy_from_slice(&human[..self.settings.humans]);
                let events = self.game.step(input);
                self.pending_bombs = [false; 4];
                if self.settings.sound {
                    if let Some(event) = events.last() {
                        self.sound.play(*event);
                    }
                }
                for i in &mut human {
                    i.bomb = false;
                }
            }
        } else {
            self.pending_bombs = [false; 4];
            self.accumulator = 0.;
        }
        if self.game.champion.is_some() && !self.recorded {
            self.recorded = true;
            self.settings.matches = self.settings.matches.saturating_add(1);
            let id = self.game.champion.unwrap();
            self.settings.wins[id] = self.settings.wins[id].saturating_add(1);
            self.persist();
        }
        egui::TopBottomPanel::bottom("blast-status").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (id, player) in self
                    .game
                    .arena
                    .players
                    .iter()
                    .enumerate()
                    .take(self.settings.humans)
                {
                    ui.label(format!(
                        "P{}  Bombs {} / {}   Reach {}",
                        id + 1,
                        self.game
                            .arena
                            .bombs
                            .iter()
                            .filter(|b| b.owner == id)
                            .count(),
                        player.capacity,
                        player.range
                    ));
                    ui.separator();
                }
                ui.label("Esc  Pause · Ctrl+,  Settings · Ctrl+M  Sound");
            });
        });
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(self.theme.background)
                    .inner_margin(16.),
            )
            .show(ctx, |ui| {
                arcade_presentation::backdrop(ui);
                self.header(ui);
                ui.add_space(10.);
                self.board(ui);
            });
        self.dialogs(ctx);
        ctx.request_repaint_after(Duration::from_secs_f64(1. / HZ as f64));
    }
    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        self.sound.stop();
        self.persist();
    }
}

#[cfg(test)]
mod save_protection_tests {
    use super::*;
    use eframe::App as _;

    #[test]
    fn rejected_files_survive_changes_and_exit_until_reopened() {
        for original in [
            b"broken".to_vec(),
            br#"{"version":99}"#.to_vec(),
            vec![b' '; 16385],
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("blast.json");
            std::fs::write(&path, &original).unwrap();
            std::fs::write(dir.path().join("other-game.json"), b"keep").unwrap();
            let mut app = App::from_dir(dir.path().to_owned());
            assert!(!app.writable);
            app.settings.sound = true;
            app.persist();
            assert!(app.prepare_to_leave().is_err());
            app.on_exit(None);
            assert_eq!(std::fs::read(&path).unwrap(), original);
            std::fs::rename(&path, dir.path().join("preserved.json")).unwrap();
            app.persist();
            assert!(
                !path.exists(),
                "repair must not silently unlock the existing app"
            );
            let mut reopened = App::from_dir(dir.path().to_owned());
            assert!(reopened.writable);
            reopened.on_exit(None);
            assert!(Settings::load(dir.path()).is_ok());
            assert_eq!(
                std::fs::read(dir.path().join("preserved.json")).unwrap(),
                original
            );
            assert_eq!(
                std::fs::read(dir.path().join("other-game.json")).unwrap(),
                b"keep"
            );
        }
    }

    #[test]
    fn ordinary_write_failure_can_retry() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = App::from_dir(dir.path().to_owned());
        let path = dir.path().join("blast.json");
        std::fs::create_dir(&path).unwrap();
        app.persist();
        assert!(app.error.is_some());
        assert!(app.writable);
        std::fs::remove_dir(&path).unwrap();
        app.settings.matches = 7;
        app.on_exit(None);
        assert_eq!(Settings::load(dir.path()).unwrap().matches, 7);
    }
}
