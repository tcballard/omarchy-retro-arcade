use crate::{audio::Audio, engine::*};
use eframe::egui::{self, Color32, Key, Pos2, Rect, RichText, Stroke, Vec2};
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::PathBuf,
    time::{Duration, Instant},
};
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub keys: [String; 7],
    pub das: u32,
    pub arr: u32,
    pub audio: bool,
    pub reduced_motion: bool,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            keys: [
                "ArrowLeft",
                "ArrowRight",
                "ArrowDown",
                "Space",
                "ArrowUp",
                "Z",
                "C",
            ]
            .map(str::to_owned),
            das: 10,
            arr: 2,
            audio: false,
            reduced_motion: false,
        }
    }
}
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Saved {
    pub version: u32,
    pub preferences: Preferences,
    pub marathon: Option<Sim>,
    pub sprint: Option<Sim>,
    pub best_score: u64,
    pub best_ticks: Option<u64>,
}
#[derive(Default)]
struct InputLatch {
    held: u8,
    pressed: u8,
}
impl InputLatch {
    fn frame(&mut self, held: u8, pressed: u8) {
        self.held = held;
        self.pressed |= pressed;
    }
    fn tick(&mut self) -> u8 {
        let input = self.held | self.pressed;
        self.pressed = 0;
        input
    }
}
pub struct StackApp {
    input: InputLatch,
    online: crate::online::Online,
    pub sim: Option<Sim>,
    pub saved: Saved,
    dir: PathBuf,
    pub paused: bool,
    settings: bool,
    rebind: Option<usize>,
    last: Instant,
    accumulator: f64,
    last_save: Instant,
    theme_at: Instant,
    audio: Audio,
    finished: bool,
    error: Option<String>,
    save_blocked: Option<String>,
    restart: bool,
    flash: f32,
}
impl StackApp {
    pub fn prepare_to_leave(&mut self) -> Result<(), String> {
        self.finished = false;
        // Clear only the previous write error; the rejected-load guard is separate.
        self.error = None;
        self.suspend();
        if let Some(error) = &self.save_blocked {
            return Err(error.clone());
        }
        self.error.clone().map_or(Ok(()), Err)
    }
    pub fn new() -> Self {
        let dir = std::env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/state")
            })
            .join("omarchy-stack");
        Self::from_dir(dir)
    }
    fn from_dir(dir: PathBuf) -> Self {
        let mut save_blocked = None;
        let saved = match std::fs::read(dir.join("session.json")) {
            Ok(b) => match serde_json::from_slice::<Saved>(&b) {
                Ok(s)
                    if s.version == 1
                        && s.marathon.as_ref().is_none_or(Sim::valid)
                        && s.sprint.as_ref().is_none_or(Sim::valid) =>
                {
                    s
                }
                _ => {
                    save_blocked =
                        Some("The saved run is invalid or uses an unsupported version.".into());
                    Saved::default()
                }
            },
            Err(e)
                if e.kind() == std::io::ErrorKind::NotFound
                    && matches!(
                        std::fs::symlink_metadata(dir.join("session.json")),
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound
                    ) =>
            {
                Saved::default()
            }
            Err(e) => {
                save_blocked = Some(format!("Could not read the saved run: {e}"));
                Saved::default()
            }
        };
        Self {
            input: InputLatch::default(),
            online: crate::online::Online::new(),
            sim: None,
            saved,
            dir,
            paused: true,
            settings: false,
            rebind: None,
            last: Instant::now(),
            accumulator: 0.,
            last_save: Instant::now(),
            theme_at: Instant::now() - Duration::from_secs(3),
            audio: Audio::default(),
            finished: false,
            error: None,
            save_blocked,
            restart: false,
            flash: 0.,
        }
    }
    pub fn finished(&self) -> bool {
        self.finished
    }
    pub fn suspend(&mut self) {
        self.paused = true;
        self.input = InputLatch::default();
        self.audio.stop();
        self.persist();
    }
    fn start(&mut self, mode: Mode, resume: bool) {
        if !resume
            && self
                .online
                .prepare(mode, self.saved.preferences.das, self.saved.preferences.arr)
        {
            self.paused = true;
            return;
        }
        self.online.discard();
        let stored = match mode {
            Mode::Marathon => &self.saved.marathon,
            Mode::Sprint => &self.saved.sprint,
        };
        self.sim = if resume { stored.clone() } else { None };
        if self.sim.is_none() {
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            self.sim = Some(Sim::new(
                seed,
                mode,
                self.saved.preferences.das,
                self.saved.preferences.arr,
            ));
        }
        self.paused = false;
        self.last = Instant::now();
        self.accumulator = 0.;
        self.persist();
    }
    fn snapshot(&mut self) {
        if let Some(s) = &self.sim {
            if s.outcome != Outcome::Playing {
                match s.mode {
                    Mode::Marathon => self.saved.best_score = self.saved.best_score.max(s.score),
                    Mode::Sprint => {
                        if s.outcome == Outcome::Complete {
                            self.saved.best_ticks =
                                Some(self.saved.best_ticks.unwrap_or(u64::MAX).min(s.ticks));
                        }
                    }
                }
            }
            let slot = match s.mode {
                Mode::Marathon => &mut self.saved.marathon,
                Mode::Sprint => &mut self.saved.sprint,
            };
            *slot = if s.outcome == Outcome::Playing {
                Some(s.clone())
            } else {
                None
            };
        }
        self.saved.version = 1;
    }
    fn persist(&mut self) {
        self.snapshot();
        self.last_save = Instant::now();
        // Keep this session read-only after a load failure, even if the player
        // starts a new run or the original file becomes writable later.
        if self.save_blocked.is_some() {
            return;
        }
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            std::fs::create_dir_all(&self.dir)?;
            let mut f = tempfile::NamedTempFile::new_in(&self.dir)?;
            f.write_all(&serde_json::to_vec_pretty(&self.saved)?)?;
            f.as_file().sync_all()?;
            f.persist(self.dir.join("session.json"))?;
            std::fs::File::open(&self.dir)?.sync_all()?;
            Ok(())
        })();
        if let Err(e) = result {
            self.error = Some(format!(
                "Save failed: {e}. Keep Arcade open and free some space."
            ));
        }
    }
    fn key(&self, i: usize) -> Key {
        Key::from_name(&self.saved.preferences.keys[i]).unwrap_or(
            [
                Key::ArrowLeft,
                Key::ArrowRight,
                Key::ArrowDown,
                Key::Space,
                Key::ArrowUp,
                Key::Z,
                Key::C,
            ][i],
        )
    }
    fn theme(&mut self, ctx: &egui::Context) {
        if self.theme_at.elapsed() < Duration::from_secs(2) {
            return;
        }
        self.theme_at = Instant::now();
        let t = omarchy_chess::theme::Theme::load();
        let light =
            t.background.r() as u32 + t.background.g() as u32 + t.background.b() as u32 > 400;
        let mut v = if light {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };
        v.panel_fill = t.background;
        v.window_fill = t.background;
        v.override_text_color = Some(t.foreground);
        v.selection.bg_fill = t.accent;
        ctx.set_visuals(v);
        arcade_presentation::apply(ctx);
    }
    fn menu(&mut self, ui: &mut egui::Ui) {
        ui.add_space(20.);
        ui.label(
            RichText::new("STACK / OMARCHY ARCADE")
                .monospace()
                .color(ui.visuals().selection.bg_fill),
        );
        ui.heading(RichText::new("STACK").size(54.));
        ui.label("Seven shapes. One more possibility.");
        ui.add_space(22.);
        let art = Rect::from_min_size(
            ui.max_rect().right_top() + Vec2::new(-290., 90.),
            Vec2::new(240., 330.),
        );
        if ui.available_width() > 850. {
            arcade_presentation::bezel(ui.painter(), art, ui.visuals().selection.bg_fill);
            ui.painter()
                .rect_filled(art, 0., Color32::from_rgb(9, 16, 18));
            for (row, pattern) in [
                "      ", "    11", "    1 ", "  221 ", "332444", "355466", "775666", "777555",
            ]
            .iter()
            .enumerate()
            {
                for (col, k) in pattern.bytes().enumerate() {
                    if k != b' ' {
                        let r = Rect::from_min_size(
                            art.min + Vec2::new(8. + col as f32 * 37., 26. + row as f32 * 37.),
                            Vec2::splat(34.),
                        );
                        arcade_presentation::tile(ui.painter(), r, colour(k - b'0'), true);
                    }
                }
            }
        }
        for mode in [Mode::Marathon, Mode::Sprint] {
            egui::Frame::group(ui.style())
                .fill(ui.visuals().window_fill)
                .stroke(Stroke::new(
                    1_f32,
                    arcade_presentation::BRASS.gamma_multiply(0.6),
                ))
                .inner_margin(20.)
                .show(ui, |ui| {
                    ui.set_width(410.);
                    ui.heading(format!("{mode:?}"));
                    ui.label(if mode == Mode::Marathon {
                        "Keep clearing as the pace rises."
                    } else {
                        "Forty lines. Your fastest clear."
                    });
                    ui.add_space(12.);
                    let best = match mode {
                        Mode::Marathon => format!("Local best  {}", self.saved.best_score),
                        Mode::Sprint => format!(
                            "Local best  {}",
                            self.saved.best_ticks.map(clock).unwrap_or("—".into())
                        ),
                    };
                    ui.label(best);
                    ui.horizontal(|ui| {
                        if ui.button("New run").clicked() {
                            self.start(mode, false);
                        }
                        let available = match mode {
                            Mode::Marathon => self.saved.marathon.is_some(),
                            Mode::Sprint => self.saved.sprint.is_some(),
                        };
                        if ui
                            .add_enabled(available, egui::Button::new("Resume saved run"))
                            .clicked()
                        {
                            self.start(mode, true);
                        }
                    });
                });
            ui.add_space(15.);
        }
        ui.label("Enter  Marathon     S  Sprint     R  Resume Marathon");
        ui.add_space(15.);
        if ui.button("Controls & preferences").clicked() {
            self.settings = true;
        }
        self.online.menu(ui);
        ui.label(
            RichText::new("Local records always work offline.")
                .small()
                .weak(),
        );
    }
    fn board(&self, ui: &mut egui::Ui, s: &Sim) {
        let avail = ui.available_size();
        let cell = ((avail.y - 24.) / 20.)
            .min((avail.x - 210.) / 10.)
            .clamp(8., 40.);
        let width = cell * 10.;
        ui.horizontal_top(|ui| {
            ui.add_space(((avail.x - width - 165.) / 2.).max(0.));
            let (r, _) = ui.allocate_exact_size(Vec2::new(width, cell * 20.), egui::Sense::hover());
            let p = ui.painter();
            arcade_presentation::bezel(p, r, ui.visuals().selection.bg_fill);
            let dark = ui.visuals().dark_mode;
            p.rect_filled(
                r,
                4.,
                if dark {
                    Color32::from_rgb(14, 20, 28)
                } else {
                    Color32::from_rgb(225, 230, 236)
                },
            );
            for y in 0..20 {
                for x in 0..10 {
                    let tile = Rect::from_min_size(
                        r.min + Vec2::new(x as f32 * cell, y as f32 * cell),
                        Vec2::splat(cell),
                    );
                    p.rect_stroke(
                        tile,
                        0.,
                        Stroke::new(
                            0.4f32,
                            if dark {
                                Color32::from_gray(37)
                            } else {
                                Color32::from_gray(200)
                            },
                        ),
                        egui::StrokeKind::Inside,
                    );
                    let k = s.board[y + 4][x];
                    if k != 0 {
                        block(p, tile, k, false, false);
                    }
                }
            }
            if s.outcome == Outcome::Playing {
                for (piece, ghost) in [(s.ghost(), true), (s.active, false)] {
                    for (x, y) in piece.cells() {
                        if y >= 4 {
                            let tile = Rect::from_min_size(
                                r.min + Vec2::new(x as f32 * cell, (y - 4) as f32 * cell),
                                Vec2::splat(cell),
                            );
                            block(p, tile, piece.kind, !ghost, ghost);
                        }
                    }
                }
            }
            if self.flash > 0. && !self.saved.preferences.reduced_motion {
                p.rect_stroke(
                    r.shrink(1.),
                    4.,
                    Stroke::new(
                        3.0_f32,
                        Color32::from_white_alpha((self.flash * 180.) as u8),
                    ),
                    egui::StrokeKind::Inside,
                );
            }
            ui.add_space(18.);
            ui.vertical(|ui| {
                ui.set_min_width(135.);
                ui.add_space(8.);
                ui.label(
                    RichText::new("RESERVE")
                        .monospace()
                        .color(arcade_presentation::BRASS),
                );
                preview(ui, s.held, cell.min(24.), s.hold_used);
                ui.add_space(12.);
                ui.label(
                    RichText::new("UP NEXT")
                        .monospace()
                        .color(arcade_presentation::BRASS),
                );
                for &k in s.queue.iter().take(5) {
                    preview(ui, Some(k), cell.min(20.), false);
                    ui.add_space(4.);
                }
            });
        });
    }
}
impl Default for StackApp {
    fn default() -> Self {
        Self::new()
    }
}
impl eframe::App for StackApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        arcade_presentation::apply(ctx);
        if let Some(sim) = self.online.poll() {
            self.sim = Some(sim);
            self.paused = false;
            self.last = Instant::now();
            self.accumulator = 0.;
            self.persist();
        }
        self.theme(ctx);
        let now = Instant::now();
        let dt = now.duration_since(self.last).as_secs_f64();
        self.last = now;
        let focused = ctx.input(|i| i.focused);
        if !focused && !self.paused {
            self.suspend();
        }
        let press = |key| ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key));
        if !self.settings && !self.restart && !self.online.has_modal() {
            if self.sim.is_none() {
                if press(Key::Enter) {
                    self.start(Mode::Marathon, false);
                } else if press(Key::S) {
                    self.start(Mode::Sprint, false);
                } else if press(Key::R) && self.saved.marathon.is_some() {
                    self.start(Mode::Marathon, true);
                }
            } else if press(Key::Escape) || press(Key::P) {
                self.paused = !self.paused;
                self.accumulator = 0.;
                self.audio.stop();
                self.persist();
            }
            if self.sim.is_some() && press(Key::R) {
                self.restart = true;
                self.suspend();
            }
        }
        if let Some(s) = &self.sim {
            self.online.pause(
                s.ticks,
                self.paused || !focused || self.settings || self.restart,
            );
        }
        if !self.paused && focused && !self.settings && !self.restart {
            let mut held = 0;
            let mut pressed = 0;
            for i in 0..7 {
                ctx.input(|input| {
                    if input.key_down(self.key(i)) {
                        held |= 1 << i;
                    }
                    if input.key_pressed(self.key(i)) {
                        pressed |= 1 << i;
                    }
                });
            }
            self.input.frame(held, pressed);
            self.accumulator += dt.min(0.25);
            while self.accumulator >= 1. / 60. {
                self.accumulator -= 1. / 60.;
                let input = self.input.tick();
                if let Some(s) = &mut self.sim {
                    let locks = s.locks;
                    self.online.tick(s, input);
                    s.tick(input);
                    if s.locks != locks {
                        if self.saved.preferences.audio {
                            self.audio.play(if s.last_clear > 0 { 2 } else { 0 });
                        }
                        if s.last_clear > 0 {
                            self.flash = 1.;
                        }
                    }
                }
            }
            if self
                .sim
                .as_ref()
                .is_some_and(|s| s.outcome != Outcome::Playing)
            {
                self.paused = true;
                self.persist();
            }
        } else {
            self.input = InputLatch::default();
            self.accumulator = 0.;
        }
        self.flash = (self.flash - dt as f32 * 3.).max(0.);
        if self.last_save.elapsed() > Duration::from_secs(2) && self.sim.is_some() {
            self.persist();
        }
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(18.),
            )
            .show(ctx, |ui| {
                arcade_presentation::backdrop(ui);
                if let Some(reason) = &self.save_blocked {
                    ui.group(|ui| {
                        ui.strong("Progress is not being saved");
                        ui.label("Your existing save is untouched. You can play, but progress and settings will not be saved.");
                        ui.collapsing("Save recovery details", |ui| {
                            ui.label(reason);
                            ui.label(format!("Save: {}", self.dir.join("session.json").display()));
                            ui.label("Close Stack, recover or move the existing save, then reopen Stack to enable saving.");
                        });
                    });
                }
                if self.sim.is_none() {
                    if self.save_blocked.is_some() {
                        // Recovery information must not push menu actions out
                        // of reach in a compact window.
                        egui::ScrollArea::vertical().show(ui, |ui| self.menu(ui));
                    } else {
                        self.menu(ui);
                    }
                } else {
                    let s = self.sim.as_ref().unwrap();
                    ui.horizontal(|ui| {
                        ui.heading(RichText::new("STACK").size(26.));
                        ui.label(format!("/ {:?}", s.mode));
                        ui.separator();
                        ui.label(
                            RichText::new(format!("{:06}", s.score))
                                .monospace()
                                .size(24.),
                        );
                        ui.label(format!(
                            "Level {}  ·  Lines {}{}  ·  {}",
                            s.level(),
                            s.lines,
                            if s.mode == Mode::Sprint { " / 40" } else { "" },
                            clock(s.ticks)
                        ));
                    });
                    ui.horizontal(|ui| {
                        if ui
                            .button(if self.paused {
                                "Resume · P"
                            } else {
                                "Pause · P"
                            })
                            .clicked()
                        {
                            self.paused = !self.paused;
                            self.audio.stop();
                        }
                        if ui.button("Restart · R").clicked() {
                            self.restart = true;
                            self.suspend();
                        }
                        if ui.button("Preferences").clicked() {
                            self.settings = true;
                            self.suspend();
                        }
                    });
                    ui.add_space(8.);
                    ui.label(
                        RichText::new(format!(
                            "{} / {} move   {} drop   {} / {} turn   {} hold",
                            self.saved.preferences.keys[0],
                            self.saved.preferences.keys[1],
                            self.saved.preferences.keys[3],
                            self.saved.preferences.keys[4],
                            self.saved.preferences.keys[5],
                            self.saved.preferences.keys[6]
                        ))
                        .small()
                        .weak(),
                    );
                    ui.add_space(8.);
                    self.board(ui, self.sim.as_ref().unwrap());
                }
            });
        if self.paused && self.sim.is_some() && !self.settings && !self.restart {
            let s = self.sim.as_ref().unwrap();
            let outcome = s.outcome;
            let mode = s.mode;
            let score = s.score;
            let ticks = s.ticks;
            egui::Window::new(if outcome == Outcome::Playing {
                "Paused"
            } else if outcome == Outcome::Complete {
                "Forty. Finished."
            } else {
                "Room for another?"
            })
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                if outcome == Outcome::Playing {
                    ui.label("Your board and clock are on hold.");
                    if ui.button("Resume · P").clicked() && focused {
                        self.paused = false;
                        self.last = Instant::now();
                    }
                } else {
                    ui.heading(if mode == Mode::Sprint && outcome == Outcome::Complete {
                        clock(ticks)
                    } else {
                        format!("{score} points")
                    });
                    ui.label("Result saved locally.");
                    self.online.result(ui, self.sim.as_ref().unwrap());
                    if ui.button("Play again · R").clicked() {
                        self.start(mode, false);
                    }
                }
                if ui.button("Choose mode").clicked() {
                    self.persist();
                    self.sim = None;
                }
                if ui.button("Back to Arcade").clicked() {
                    self.suspend();
                    self.finished = true;
                }
            });
        }
        if self.restart {
            egui::Window::new("Start a fresh run?")
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("This replaces the current saved run for this mode.");
                    ui.horizontal(|ui| {
                        if ui.button("Restart").clicked() {
                            let mode = self.sim.as_ref().unwrap().mode;
                            self.start(mode, false);
                            self.restart = false;
                        }
                        if ui.button("Keep run").clicked() {
                            self.restart = false;
                        }
                    });
                });
        }
        if self.settings {
            egui::Window::new("Controls & preferences")
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .collapsible(false)
                .show(ctx, |ui| {
                    for (i, name) in [
                        "Left",
                        "Right",
                        "Soft drop",
                        "Hard drop",
                        "Clockwise",
                        "Anticlockwise",
                        "Hold",
                    ]
                    .iter()
                    .enumerate()
                    {
                        ui.horizontal(|ui| {
                            ui.label(*name);
                            if ui
                                .button(if self.rebind == Some(i) {
                                    "Press a key…"
                                } else {
                                    &self.saved.preferences.keys[i]
                                })
                                .clicked()
                            {
                                self.rebind = Some(i);
                            }
                        });
                    }
                    ui.label("P / Escape pause · R restart · Ctrl+H Arcade · F11 fullscreen");
                    if let Some(idx) = self.rebind {
                        for e in ctx.input(|i| i.events.clone()) {
                            if let egui::Event::Key {
                                key,
                                pressed: true,
                                repeat: false,
                                modifiers,
                                ..
                            } = e
                            {
                                if modifiers.is_none()
                                    && ![Key::P, Key::Escape, Key::R, Key::F11, Key::Enter, Key::S]
                                        .contains(&key)
                                {
                                    let name = key.name().to_owned();
                                    if let Some(other) =
                                        self.saved.preferences.keys.iter().position(|k| k == &name)
                                    {
                                        self.saved.preferences.keys.swap(idx, other);
                                    }
                                    self.saved.preferences.keys[idx] = name;
                                    self.rebind = None;
                                }
                            }
                        }
                    }
                    ui.add(
                        egui::Slider::new(&mut self.saved.preferences.das, 4..=24)
                            .text("Initial repeat delay (ticks)"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.saved.preferences.arr, 1..=12)
                            .text("Repeat interval (ticks)"),
                    );
                    ui.label("60 ticks = 1 second. Timing changes apply to new runs.");
                    ui.checkbox(&mut self.saved.preferences.audio, "Sound effects");
                    ui.checkbox(&mut self.saved.preferences.reduced_motion, "Reduced motion");
                    if ui.button("Done").clicked() {
                        self.settings = false;
                        self.rebind = None;
                        self.persist();
                    }
                });
        }
        if let Some(error) = self.error.clone() {
            egui::Window::new("Save notice").show(ctx, |ui| {
                ui.label(error);
                if ui.button("Dismiss").clicked() {
                    self.error = None;
                }
            });
        }
        self.online.windows(ctx, self.sim.as_ref());
        ctx.request_repaint_after(Duration::from_millis(8));
    }
    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        self.suspend();
    }
}
fn clock(ticks: u64) -> String {
    format!(
        "{}:{:02}.{:02}",
        ticks / 3600,
        (ticks / 60) % 60,
        (ticks % 60) * 100 / 60
    )
}
fn colour(k: u8) -> Color32 {
    match k {
        1 => Color32::from_rgb(72, 192, 220),
        2 => Color32::from_rgb(234, 190, 81),
        3 => Color32::from_rgb(166, 120, 219),
        4 => Color32::from_rgb(98, 184, 137),
        5 => Color32::from_rgb(222, 103, 117),
        6 => Color32::from_rgb(91, 143, 215),
        _ => Color32::from_rgb(230, 145, 88),
    }
}
fn block(p: &egui::Painter, r: Rect, k: u8, active: bool, ghost: bool) {
    let c = colour(k);
    let r = r.shrink(1.5);
    if ghost {
        p.rect_stroke(r, 2., Stroke::new(1.5f32, c), egui::StrokeKind::Inside);
        p.circle_filled(r.center(), 1.5, c);
        return;
    }
    arcade_presentation::tile(
        p,
        r,
        if active { c } else { c.gamma_multiply(0.80) },
        active,
    );
}
fn preview(ui: &mut egui::Ui, k: Option<u8>, cell: f32, used: bool) {
    let (r, _) = ui.allocate_exact_size(Vec2::new(cell * 4., cell * 2.), egui::Sense::hover());
    if let Some(k) = k {
        for (x, y) in (Piece {
            kind: k,
            rotation: 0,
            x: 0,
            y: 0,
        })
        .cells()
        {
            block(
                ui.painter(),
                Rect::from_min_size(
                    Pos2::new(r.min.x + x as f32 * cell, r.min.y + y as f32 * cell),
                    Vec2::splat(cell),
                ),
                k,
                !used,
                false,
            );
        }
    } else {
        ui.painter().text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            "—",
            egui::FontId::monospace(16.),
            ui.visuals().weak_text_color(),
        );
    }
}

#[cfg(test)]
mod input_tests {
    use super::*;
    #[test]
    fn a_tap_between_simulation_ticks_is_not_lost_or_repeated() {
        let mut latch = InputLatch::default();
        latch.frame(HARD, HARD);
        latch.frame(0, 0);
        assert_eq!(latch.tick(), HARD);
        assert_eq!(latch.tick(), 0);
    }
}

#[cfg(test)]
mod save_tests {
    use super::*;
    use eframe::App;
    use std::fs;

    fn exercise_save_paths(app: &mut StackApp) {
        // New play, periodic/settings saves, pause/shelf exit and normal close.
        app.sim = Some(Sim::new(123, Mode::Marathon, 10, 2));
        app.saved.preferences.audio = true;
        app.persist();
        app.suspend();
        app.on_exit(None);
    }

    #[test]
    fn failed_recovery_destination_cannot_destroy_the_original() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("session.json");
        let original = br#"{"version":999,"future_run":"keep me"}"#;
        fs::write(&path, original).unwrap();
        let recovery = dir.path().join("session.rejected.json");
        fs::create_dir(&recovery).unwrap();

        let mut app = StackApp::from_dir(dir.path().into());
        exercise_save_paths(&mut app);

        assert_eq!(fs::read(&path).unwrap(), original);
        assert!(recovery.is_dir());
        assert_eq!(fs::read_dir(recovery).unwrap().count(), 0);
    }

    #[test]
    fn rejected_saves_and_previous_recovery_files_survive_reopening() {
        for original in [b"broken JSON".as_slice(), br#"{"version":2}"#, b"{}"] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("session.json");
            let recovery = dir.path().join("session.rejected.json");
            fs::write(&path, original).unwrap();
            fs::write(&recovery, b"earlier recovery data").unwrap();

            for _ in 0..2 {
                let mut app = StackApp::from_dir(dir.path().into());
                exercise_save_paths(&mut app);
                assert_eq!(fs::read(&path).unwrap(), original);
                assert_eq!(fs::read(&recovery).unwrap(), b"earlier recovery data");
            }
        }
    }

    #[test]
    fn invalid_simulation_is_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("session.json");
        let mut sim = Sim::new(123, Mode::Marathon, 10, 2);
        sim.active.kind = 0;
        let original = serde_json::to_vec(&Saved {
            version: 1,
            marathon: Some(sim),
            ..Default::default()
        })
        .unwrap();
        fs::write(&path, &original).unwrap();

        let mut app = StackApp::from_dir(dir.path().into());
        exercise_save_paths(&mut app);

        assert_eq!(fs::read(&path).unwrap(), original);
    }

    #[cfg(unix)]
    #[test]
    fn read_errors_and_dangling_links_do_not_enable_saving() {
        use std::os::unix::fs::symlink;
        for target in ["session.json", "missing-save.json"] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("session.json");
            symlink(target, &path).unwrap();
            assert!(fs::read(&path).is_err());

            let mut app = StackApp::from_dir(dir.path().into());
            exercise_save_paths(&mut app);

            assert_eq!(fs::read_link(&path).unwrap(), PathBuf::from(target));
        }
    }

    #[test]
    fn valid_saves_keep_both_modes_records_and_preferences() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("session.json");
        let marathon = Sim::new(123, Mode::Marathon, 10, 2);
        let sprint = Sim::new(456, Mode::Sprint, 10, 2);
        let saved = Saved {
            version: 1,
            marathon: Some(marathon.clone()),
            sprint: Some(sprint.clone()),
            best_score: 4200,
            best_ticks: Some(9000),
            preferences: Preferences {
                audio: true,
                ..Default::default()
            },
        };
        fs::write(&path, serde_json::to_vec(&saved).unwrap()).unwrap();

        let mut app = StackApp::from_dir(dir.path().into());
        app.suspend();
        app.on_exit(None);
        let mut reopened = StackApp::from_dir(dir.path().into());
        assert_eq!(reopened.saved.marathon.as_ref(), Some(&marathon));
        assert_eq!(reopened.saved.sprint.as_ref(), Some(&sprint));
        assert_eq!(reopened.saved.best_score, 4200);
        assert_eq!(reopened.saved.best_ticks, Some(9000));
        assert!(reopened.saved.preferences.audio);
        reopened.saved.best_score = 4300;
        reopened.persist();
        let on_disk: Saved = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(on_disk.best_score, 4300);
    }

    #[test]
    fn new_players_can_save_and_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let state = dir.path().join("new-player");
        let mut app = StackApp::from_dir(state.clone());
        exercise_save_paths(&mut app);
        let reopened = StackApp::from_dir(state);
        assert_eq!(reopened.saved.marathon, app.sim);
        assert!(reopened.saved.preferences.audio);
    }

    #[test]
    fn moving_a_rejected_save_requires_reopening_before_saving() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("session.json");
        let recovery = dir.path().join("recovered.json");
        fs::write(&path, b"future save").unwrap();
        let mut app = StackApp::from_dir(dir.path().into());
        fs::rename(&path, &recovery).unwrap();

        exercise_save_paths(&mut app);
        assert!(!path.exists());
        assert_eq!(fs::read(&recovery).unwrap(), b"future save");

        let mut reopened = StackApp::from_dir(dir.path().into());
        exercise_save_paths(&mut reopened);
        assert!(path.is_file());
        assert_eq!(fs::read(&recovery).unwrap(), b"future save");
    }
}
