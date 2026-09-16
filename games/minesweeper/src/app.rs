use crate::{
    engine::{Difficulty, Game, Status},
    storage::{self, Save},
};
use eframe::egui::{self, Align2, FontId, Key, Rect, Sense, Stroke, StrokeKind, Vec2};
use omarchy_chess::theme::Theme;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

pub struct App {
    state: Save,
    path: PathBuf,
    writable: bool,
    error: Option<String>,
    theme: Theme,
    themed: Instant,
    tick: Instant,
    saved: Instant,
    paused: bool,
    help: bool,
    restart: Option<Difficulty>,
    cursor: usize,
    input_enabled: bool,
    was_running: bool,
    #[cfg(test)]
    cell_rects: Vec<Rect>,
}
impl App {
    pub fn new() -> Result<Self, String> {
        Self::open(storage::path()?)
    }
    fn open(path: PathBuf) -> Result<Self, String> {
        let (state, error, writable) = match storage::load(&path) {
            Ok(s) => (s.unwrap_or_default(), None, true),
            Err(e) => (Save::default(), Some(e), false),
        };
        let paused = state.game.status == Status::Playing;
        Ok(Self {
            state,
            path,
            writable,
            error,
            theme: Theme::load(),
            themed: Instant::now(),
            tick: Instant::now(),
            saved: Instant::now(),
            paused,
            help: false,
            restart: None,
            cursor: 0,
            input_enabled: true,
            was_running: false,
            #[cfg(test)]
            cell_rects: vec![],
        })
    }
    fn flush(&mut self) {
        if self.writable {
            self.error = storage::save(&self.path, &self.state).err();
        }
        self.saved = Instant::now();
    }
    pub fn suspend(&mut self) {
        self.paused = true;
        self.was_running = false;
        self.flush();
    }
    pub fn set_input_enabled(&mut self, enabled: bool) {
        self.input_enabled = enabled;
    }
    fn action(&mut self, action: u8) {
        let before = self.state.game.status;
        match action {
            0 => self.state.game.reveal(self.cursor, &mut rand::thread_rng()),
            1 => self.state.game.flag(self.cursor),
            _ => self.state.game.chord(self.cursor),
        }
        self.state.account(before);
        self.flush();
    }
    fn new_game(&mut self, difficulty: Difficulty) {
        self.state.game = Game::new(difficulty);
        self.cursor = 0;
        self.paused = false;
        self.restart = None;
        self.was_running = false;
        self.flush();
    }
    pub fn ui(&mut self, ctx: &egui::Context) {
        let now = Instant::now();
        let focused = ctx.input(|i| i.focused);
        let blocked_at_start =
            self.paused || self.help || self.restart.is_some() || !self.input_enabled || !focused;
        if !blocked_at_start && self.was_running && self.state.game.status == Status::Playing {
            self.state.game.elapsed_ms = self.state.game.elapsed_ms.saturating_add(
                now.duration_since(self.tick)
                    .as_millis()
                    .min(u64::MAX as u128) as u64,
            );
        }
        self.tick = now;
        if !focused && self.state.game.status == Status::Playing {
            self.paused = true;
        }
        if self.themed.elapsed() > Duration::from_secs(1) {
            self.theme = Theme::load();
            self.themed = now;
        }
        if self.input_enabled
            && focused
            && !self.help
            && self.restart.is_none()
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape))
        {
            self.paused = !self.paused;
        }
        let mut visuals = if self.theme.light() {
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
        let enabled = !blocked_at_start && !self.paused;
        if enabled {
            // Consume gameplay keys before buttons can activate from Space/Enter.
            let action = ctx.input_mut(|i| {
                let (w, h, _) = self.state.game.difficulty.dimensions();
                if i.consume_key(egui::Modifiers::NONE, Key::ArrowLeft)
                    && !self.cursor.is_multiple_of(w)
                {
                    self.cursor -= 1;
                }
                if i.consume_key(egui::Modifiers::NONE, Key::ArrowRight) && self.cursor % w + 1 < w
                {
                    self.cursor += 1;
                }
                if i.consume_key(egui::Modifiers::NONE, Key::ArrowUp) && self.cursor >= w {
                    self.cursor -= w;
                }
                if i.consume_key(egui::Modifiers::NONE, Key::ArrowDown) && self.cursor + w < w * h {
                    self.cursor += w;
                }
                if i.consume_key(egui::Modifiers::NONE, Key::Space) {
                    Some(0)
                } else if i.consume_key(egui::Modifiers::NONE, Key::F) {
                    Some(1)
                } else if i.consume_key(egui::Modifiers::NONE, Key::C) {
                    Some(2)
                } else {
                    None
                }
            });
            if let Some(a) = action {
                self.action(a);
            }
        }
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(self.theme.background)
                    .inner_margin(20),
            )
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.heading(egui::RichText::new("MINESWEEPER").color(self.theme.foreground));
                    ui.separator();
                    ui.label(format!(
                        "MINES {:03}   TIME {:03}",
                        self.state.game.remaining(),
                        self.state.game.elapsed_ms / 1000
                    ));
                    if ui.button("New game").clicked() {
                        self.restart = Some(self.state.game.difficulty);
                    }
                    if ui
                        .button(if self.paused { "Resume" } else { "Pause" })
                        .clicked()
                    {
                        self.paused = !self.paused;
                    }
                    if ui.button("Help").clicked() {
                        self.help = true;
                    }
                });
                ui.horizontal(|ui| {
                    for d in Difficulty::ALL {
                        if ui
                            .selectable_label(self.state.game.difficulty == d, d.name())
                            .clicked()
                            && d != self.state.game.difficulty
                        {
                            self.restart = Some(d);
                        }
                    }
                });
                let record = &self.state.records[self.state.game.difficulty.index()];
                ui.label(format!(
                    "{} wins / {} played    Best: {}",
                    record.wins,
                    record.played,
                    record
                        .best_ms
                        .map_or("—".into(), |ms| format!("{:.1}s", ms as f64 / 1000.))
                ));
                match self.state.game.status {
                    Status::Won => {
                        ui.label("FIELD CLEARED. Nicely done.");
                    }
                    Status::Lost => {
                        ui.label("MINE HIT. One more go?");
                    }
                    Status::Ready => {
                        ui.label("Pick your opening. The first reveal is safe.");
                    }
                    Status::Playing => {
                        ui.label("Reveal · Flag · Read the numbers");
                    }
                }
                if let Some(e) = &self.error {
                    ui.colored_label(
                        self.theme.accent,
                        format!(
                            "{e}{}",
                            if self.writable {
                                ""
                            } else {
                                " — playing without saving"
                            }
                        ),
                    );
                }
                ui.add_space(10.);
                let enabled =
                    !blocked_at_start && !self.paused && !self.help && self.restart.is_none();
                let (w, h, _) = self.state.game.difficulty.dimensions();
                let cell = ((ui.available_width() - 8.) / w as f32)
                    .min((ui.available_height() - 40.) / h as f32)
                    .clamp(20., 42.);
                egui::ScrollArea::both().show(ui, |ui| {
                    let (area, _) = ui.allocate_exact_size(
                        Vec2::new(cell * w as f32, cell * h as f32),
                        Sense::hover(),
                    );
                    #[cfg(test)]
                    self.cell_rects.clear();
                    for n in 0..self.state.game.cells.len() {
                        let r = Rect::from_min_size(
                            area.min + Vec2::new((n % w) as f32 * cell, (n / w) as f32 * cell),
                            Vec2::splat(cell),
                        )
                        .shrink(1.);
                        #[cfg(test)]
                        self.cell_rects.push(r);
                        let response = ui.interact(r, ui.id().with(n), Sense::click());
                        if enabled
                            && (response.clicked()
                                || response.secondary_clicked()
                                || response.middle_clicked())
                        {
                            self.cursor = n;
                            let a = if response.secondary_clicked() {
                                1
                            } else if response.middle_clicked() || self.state.game.cells[n].revealed
                            {
                                2
                            } else {
                                0
                            };
                            self.action(a);
                        }
                        let c = self.state.game.cells[n];
                        let lost = self.state.game.status == Status::Lost;
                        let hidden = self.paused || self.help || self.restart.is_some();
                        let fill = if !hidden && self.state.game.exploded == Some(n) {
                            self.theme.accent
                        } else {
                            self.theme.background.lerp_to_gamma(
                                self.theme.foreground,
                                if !hidden && c.revealed { 0.04 } else { 0.16 },
                            )
                        };
                        ui.painter().rect_filled(r, 0., fill);
                        let text = if hidden {
                            "".into()
                        } else if lost && c.flagged && !c.mine {
                            "X".into()
                        } else if c.flagged {
                            "F".into()
                        } else if lost && c.mine {
                            "*".into()
                        } else if c.revealed {
                            let a = self.state.game.adjacent(n);
                            if a == 0 {
                                "".into()
                            } else {
                                a.to_string()
                            }
                        } else {
                            "".into()
                        };
                        ui.painter().text(
                            r.center(),
                            Align2::CENTER_CENTER,
                            &text,
                            FontId::monospace(cell * 0.58),
                            self.theme.foreground,
                        );
                        if n == self.cursor {
                            ui.painter().rect_stroke(
                                r,
                                0.,
                                Stroke::new(2_f32, self.theme.accent),
                                StrokeKind::Inside,
                            );
                        }
                        response.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::WidgetType::Button,
                                enabled,
                                format!(
                                    "Row {}, column {}: {}",
                                    n / w + 1,
                                    n % w + 1,
                                    if hidden {
                                        "paused".into()
                                    } else if text.is_empty() {
                                        if c.revealed {
                                            "empty".into()
                                        } else {
                                            "covered".into()
                                        }
                                    } else {
                                        text.clone()
                                    }
                                ),
                            )
                        });
                    }
                });
                ui.label("Arrows move · Space reveal · F flag · C chord · Esc pause");
            });
        if self.paused && !self.help && self.restart.is_none() {
            egui::Modal::new(egui::Id::new("mines-pause")).show(ctx, |ui| {
                ui.heading("Paused");
                if ui.button("Resume").clicked() {
                    self.paused = false;
                }
            });
        }
        if self.help {
            egui::Modal::new(egui::Id::new("mines-help")).show(ctx,|ui| { ui.set_max_width(440.); ui.heading("Clear the field"); ui.label("Reveal every safe square. Numbers count adjacent mines. Left-click reveals; right-click flags. Click a revealed number, middle-click, or press C to chord when its adjacent flag count matches. Incorrect flags can explode a mine. F marks the focused cell; X marks an incorrect flag after a loss. Random boards may require guessing."); if ui.button("Back").clicked() { self.help = false; } });
        }
        if let Some(d) = self.restart {
            egui::Modal::new(egui::Id::new("mines-restart")).show(ctx, |ui| {
                ui.heading(format!("New {} game?", d.name()));
                ui.label("This replaces the current board. Records are kept.");
                ui.horizontal(|ui| {
                    if ui.button("Start").clicked() {
                        self.new_game(d);
                    }
                    if ui.button("Cancel").clicked() {
                        self.restart = None;
                    }
                });
            });
        }
        self.was_running = !blocked_at_start
            && !self.paused
            && !self.help
            && self.restart.is_none()
            && self.state.game.status == Status::Playing;
        if self.saved.elapsed() >= Duration::from_secs(5) {
            self.flush();
        }
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.ui(ctx);
    }
    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame(app: &mut App, ctx: &egui::Context, keys: &[Key]) {
        let mut input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(900., 760.))),
            focused: true,
            ..Default::default()
        };
        input.events = keys
            .iter()
            .flat_map(|&key| {
                [true, false].map(move |pressed| egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                })
            })
            .collect();
        let _ = ctx.run(input, |ctx| app.ui(ctx));
    }
    #[test]
    fn keyboard_pause_help_restart_and_external_input_gate() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = App::open(dir.path().join("save.json")).unwrap();
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, &[]);
        frame(&mut app, &ctx, &[Key::F]);
        assert!(app.state.game.cells[0].flagged);
        frame(&mut app, &ctx, &[Key::Space]);
        assert_eq!(app.state.game.status, Status::Ready);
        frame(&mut app, &ctx, &[Key::F]);
        frame(&mut app, &ctx, &[Key::Space]);
        assert_eq!(app.state.game.status, Status::Playing);
        frame(&mut app, &ctx, &[Key::Escape]);
        assert!(app.paused);
        let board = app.state.game.clone();
        frame(
            &mut app,
            &ctx,
            &[Key::ArrowRight, Key::Space, Key::F, Key::C],
        );
        assert_eq!(app.state.game, board);
        frame(&mut app, &ctx, &[Key::Escape]);
        assert!(!app.paused);
        app.help = true;
        frame(&mut app, &ctx, &[Key::Space, Key::F]);
        assert_eq!(app.state.game, board);
        app.help = false;
        app.restart = Some(Difficulty::Expert);
        frame(&mut app, &ctx, &[Key::Space, Key::F]);
        assert_eq!(app.state.game, board);
        app.restart = None;
        app.set_input_enabled(false);
        frame(&mut app, &ctx, &[Key::F]);
        assert_eq!(app.state.game, board);
    }
    #[test]
    fn mouse_reveal_flag_and_modal_blocking_at_all_sizes() {
        for d in Difficulty::ALL {
            let dir = tempfile::tempdir().unwrap();
            let mut app = App::open(dir.path().join("save.json")).unwrap();
            app.new_game(d);
            let ctx = egui::Context::default();
            frame(&mut app, &ctx, &[]);
            let pos = app.cell_rects[0].center();
            let click = |app: &mut App, button| {
                for pressed in [true, false] {
                    let input = egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(
                            egui::Pos2::ZERO,
                            Vec2::new(900., 760.),
                        )),
                        focused: true,
                        events: vec![
                            egui::Event::PointerMoved(pos),
                            egui::Event::PointerButton {
                                pos,
                                button,
                                pressed,
                                modifiers: egui::Modifiers::NONE,
                            },
                        ],
                        ..Default::default()
                    };
                    let _ = ctx.run(input, |ctx| app.ui(ctx));
                }
            };
            click(&mut app, egui::PointerButton::Secondary);
            assert!(app.state.game.cells[0].flagged);
            click(&mut app, egui::PointerButton::Secondary);
            click(&mut app, egui::PointerButton::Primary);
            assert!(app.state.game.cells[0].revealed);
            assert!(app.state.game.valid());
            assert!(app.cell_rects.last().unwrap().right() <= 900.);
            app.help = true;
            let board = app.state.game.cells.clone();
            click(&mut app, egui::PointerButton::Secondary);
            assert_eq!(app.state.game.cells, board);
        }
    }
    #[test]
    fn rejected_save_survives_gameplay_and_exit() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("save.json");
        std::fs::write(&p, b"future version").unwrap();
        let mut app = App::open(p.clone()).unwrap();
        app.action(0);
        app.flush();
        assert_eq!(std::fs::read(p).unwrap(), b"future version");
    }
    #[test]
    fn elapsed_time_excludes_pause_and_closed_time() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("save.json");
        let mut app = App::open(p.clone()).unwrap();
        let ctx = egui::Context::default();
        app.action(0);
        frame(&mut app, &ctx, &[]);
        app.tick = Instant::now() - Duration::from_secs(2);
        frame(&mut app, &ctx, &[]);
        assert!(app.state.game.elapsed_ms >= 2000);
        app.suspend();
        let elapsed = app.state.game.elapsed_ms;
        app.tick = Instant::now() - Duration::from_secs(600);
        frame(&mut app, &ctx, &[]);
        assert_eq!(app.state.game.elapsed_ms, elapsed);
        let mut reopened = App::open(p).unwrap();
        reopened.tick = Instant::now() - Duration::from_secs(600);
        frame(&mut reopened, &ctx, &[]);
        assert_eq!(reopened.state.game.elapsed_ms, elapsed);
    }
}
