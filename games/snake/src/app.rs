use crate::{
    audio::Audio,
    engine::*,
    replay::{Action, Replay},
    storage::{self, Records},
    timing::{Clock, Gate},
};
use eframe::egui::{self, Color32, Key, Pos2, Rect, RichText, Stroke, Vec2};
use omarchy_chess::theme::Theme;
use std::{
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub struct SnakeApp {
    pub sim: Sim,
    records: Records,
    dir: PathBuf,
    gate: Gate,
    restored: bool,
    trace: Option<Replay>,
    clock: Clock,
    last: Instant,
    save_at: Instant,
    theme_at: Instant,
    theme: Theme,
    audio: Audio,
    finished: bool,
    settings: bool,
    rebind: Option<usize>,
    restart: bool,
    error: Option<String>,
    writable: bool,
    feedback: Duration,
}
impl Default for SnakeApp {
    fn default() -> Self {
        Self::new()
    }
}
impl SnakeApp {
    pub fn prepare_to_leave(&mut self) -> Result<(), String> {
        self.finished = false;
        if self.writable {
            self.error = None;
        }
        self.suspend();
        if !self.writable {
            return Err("Snake could not preserve the original save. Reopen after recovery to enable saving.".into());
        }
        self.error.clone().map_or(Ok(()), Err)
    }
    pub fn new() -> Self {
        Self::from_dir(storage::state_dir())
    }
    fn from_dir(dir: PathBuf) -> Self {
        let (records, saved, error, writable) = storage::load(&dir);
        let restored = saved.is_some();
        let seed = Self::seed();
        let sim = saved.unwrap_or_else(|| Sim::new(seed, records.preferences.speed));
        let trace = if restored {
            None
        } else {
            Some(Replay::new(seed, sim.speed))
        };
        Self {
            sim,
            records,
            dir,
            gate: if restored { Gate::Paused } else { Gate::Ready },
            restored,
            trace,
            clock: Clock::default(),
            last: Instant::now(),
            save_at: Instant::now(),
            theme_at: Instant::now() - Duration::from_secs(3),
            theme: Theme::load(),
            audio: Audio::default(),
            finished: false,
            settings: false,
            rebind: None,
            restart: false,
            error,
            writable,
            feedback: Duration::ZERO,
        }
    }
    fn seed() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
    pub fn finished(&self) -> bool {
        self.finished
    }
    pub fn suspend(&mut self) {
        self.pause();
    }
    fn record(&mut self, action: Action) {
        if let Some(r) = &mut self.trace {
            if !r.record(self.sim.tick, action) {
                self.trace = None;
            }
        }
    }
    fn persist(&mut self) {
        if !self.writable {
            return;
        }
        self.records.record(&self.sim);
        if let Err(e) = storage::save(&self.dir, &mut self.records, Some(&self.sim)) {
            self.error = Some(format!("Snake could not save: {e}"));
        }
        self.save_at = Instant::now();
    }
    fn fresh(&mut self, start: bool) {
        let seed = Self::seed();
        self.sim = Sim::new(seed, self.records.preferences.speed);
        self.trace = Some(Replay::new(seed, self.sim.speed));
        self.restored = false;
        self.gate = if start { Gate::Running } else { Gate::Ready };
        self.restart = false;
        self.feedback = Duration::ZERO;
        self.clock.reset();
        self.last = Instant::now();
        self.audio.stop();
        self.persist();
    }
    fn pause(&mut self) {
        if self.gate == Gate::Running {
            self.record(Action::Pause);
        }
        self.gate.pause();
        self.sim.buffered.clear();
        self.clock.reset();
        self.audio.stop();
        self.persist();
    }
    fn enter(&mut self) {
        match self.gate {
            Gate::Ready => {
                self.gate = Gate::Running;
                self.last = Instant::now();
                self.clock.reset();
            }
            Gate::Paused => {
                self.sim.buffered.clear();
                self.gate.resume();
                self.clock.reset();
                self.last = Instant::now();
            }
            Gate::Result => self.fresh(true),
            _ => (),
        }
    }
    fn change_speed(&mut self, speed: Speed) {
        if self.gate == Gate::Ready && self.sim.speed != speed {
            self.records.preferences.speed = speed;
            self.fresh(false);
        }
    }
    fn inputs(&mut self, ctx: &egui::Context) {
        if !ctx.input(|i| i.focused) {
            if matches!(self.gate, Gate::Running | Gate::Countdown(_)) {
                self.pause();
            }
            return;
        }
        for event in ctx.input(|i| i.events.clone()) {
            let egui::Event::Key {
                key,
                pressed: true,
                repeat: false,
                modifiers,
                ..
            } = event
            else {
                continue;
            };
            if modifiers.ctrl || modifiers.alt || modifiers.command {
                continue;
            }
            if key == Key::Escape {
                if self.settings {
                    self.settings = false;
                    self.rebind = None;
                } else if self.restart {
                    self.restart = false;
                } else {
                    self.pause();
                }
                continue;
            }
            if let Some(index) = self.rebind {
                if !matches!(
                    key,
                    Key::Enter
                        | Key::F11
                        | Key::N
                        | Key::R
                        | Key::Num1
                        | Key::Num2
                        | Key::Num3
                        | Key::W
                        | Key::A
                        | Key::S
                        | Key::D
                ) {
                    let name = key.name().to_owned();
                    if !self.records.preferences.keys.contains(&name) {
                        self.records.preferences.keys[index] = name;
                        self.rebind = None;
                        self.persist();
                    }
                }
                continue;
            }
            if self.settings || self.restart {
                continue;
            }
            if key == Key::Enter {
                self.enter();
                continue;
            }
            if key == Key::N && matches!(self.gate, Gate::Paused | Gate::Result) {
                self.restart = true;
                continue;
            }
            if key == Key::R {
                if self.gate == Gate::Ready || self.gate == Gate::Result {
                    self.fresh(true);
                } else {
                    self.pause();
                    self.restart = true;
                }
                continue;
            }
            for (k, speed) in [Key::Num1, Key::Num2, Key::Num3]
                .into_iter()
                .zip(Speed::ALL)
            {
                if key == k {
                    self.change_speed(speed);
                }
            }
            if self.gate != Gate::Running {
                continue;
            }
            for (i, (direction, alternate)) in [
                (Direction::Up, Key::W),
                (Direction::Right, Key::D),
                (Direction::Down, Key::S),
                (Direction::Left, Key::A),
            ]
            .into_iter()
            .enumerate()
            {
                if (key == alternate
                    || Key::from_name(&self.records.preferences.keys[i]) == Some(key))
                    && self.sim.turn(direction, false)
                {
                    self.record(Action::Turn(direction));
                    break;
                }
            }
        }
    }
    fn theme(&mut self, ctx: &egui::Context) {
        if self.theme_at.elapsed() < Duration::from_secs(2) {
            return;
        }
        self.theme = Theme::load();
        self.theme_at = Instant::now();
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
    }
    fn board(&self, ui: &mut egui::Ui) -> Rect {
        let available = ui.available_size();
        let cell = ((available.x - 32.) / WIDTH as f32)
            .min((available.y - 48.) / HEIGHT as f32)
            .max(2.);
        let size = Vec2::new(cell * WIDTH as f32, cell * HEIGHT as f32);
        let origin = ui.cursor().min + Vec2::new((available.x - size.x) * 0.5, 14.);
        let board = Rect::from_min_size(origin, size);
        let painter = ui.painter();
        let light = self.theme.light();
        let field = if light {
            Color32::from_rgb(241, 245, 235)
        } else {
            Color32::from_rgb(18, 27, 24)
        };
        let body = if light {
            Color32::from_rgb(44, 103, 65)
        } else {
            Color32::from_rgb(136, 183, 109)
        };
        let head = if light {
            Color32::from_rgb(23, 63, 41)
        } else {
            Color32::from_rgb(206, 231, 174)
        };
        let grid = if light {
            Color32::from_rgb(218, 225, 211)
        } else {
            Color32::from_rgb(29, 41, 34)
        };
        let edge = if light {
            Color32::from_rgb(105, 125, 96)
        } else {
            Color32::from_rgb(88, 116, 77)
        };
        arcade_presentation::bezel(painter, board, edge);
        painter.rect_filled(board, 0., field);
        for x in 1..WIDTH {
            let x = origin.x + x as f32 * cell;
            painter.line_segment(
                [Pos2::new(x, origin.y), Pos2::new(x, board.bottom())],
                Stroke::new(0.5_f32, grid),
            );
        }
        for y in 1..HEIGHT {
            let y = origin.y + y as f32 * cell;
            painter.line_segment(
                [Pos2::new(origin.x, y), Pos2::new(board.right(), y)],
                Stroke::new(0.5_f32, grid),
            );
        }
        let center =
            |c: u16| origin + Vec2::new((c % WIDTH) as f32 + 0.5, (c / WIDTH) as f32 + 0.5) * cell;
        for (a, b) in self.sim.body.iter().zip(self.sim.body.iter().skip(1)) {
            painter.line_segment(
                [
                    center(*a) + Vec2::new(0., cell * 0.10),
                    center(*b) + Vec2::new(0., cell * 0.10),
                ],
                Stroke::new(cell * 0.90, Color32::from_black_alpha(100)),
            );
            painter.line_segment([center(*a), center(*b)], Stroke::new(cell * 0.74, body));
        }
        for (i, c) in self.sim.body.iter().enumerate().rev() {
            painter.rect_filled(
                Rect::from_center_size(center(*c), Vec2::splat(cell * 0.74)),
                cell * 0.22,
                if i == 0 { head } else { body },
            );
            let tile = Rect::from_center_size(center(*c), Vec2::splat(cell * 0.66));
            painter.line_segment(
                [
                    tile.left_top() + Vec2::new(cell * 0.10, cell * 0.1),
                    tile.right_top() + Vec2::new(-cell * 0.10, cell * 0.1),
                ],
                Stroke::new(cell * 0.045, Color32::from_white_alpha(90)),
            );
            if i > 0 && i % 2 == 0 {
                painter.circle_filled(center(*c), cell * 0.06, head.gamma_multiply(0.45));
            }
        }
        let h = center(self.sim.body[0]);
        let (dx, dy) = self.sim.direction.delta();
        let forward = Vec2::new(dx as f32, dy as f32);
        let side = Vec2::new(-dy as f32, dx as f32);
        for sign in [-1., 1.] {
            painter.circle_filled(
                h + forward * cell * 0.20 + side * cell * 0.19 * sign,
                cell * 0.065,
                field,
            );
        }
        if let Some(food) = self.sim.food {
            let p = center(food);
            let r = cell * 0.32;
            let color = if light {
                Color32::from_rgb(175, 61, 29)
            } else {
                Color32::from_rgb(246, 156, 90)
            };
            painter.circle_filled(
                p + Vec2::new(0., cell * 0.12),
                r,
                Color32::from_black_alpha(110),
            );
            painter.circle_stroke(p, r * 1.30, Stroke::new(1_f32, color.gamma_multiply(0.35)));
            painter.add(egui::Shape::convex_polygon(
                vec![
                    p + Vec2::new(0., -r),
                    p + Vec2::new(r, 0.),
                    p + Vec2::new(0., r),
                    p + Vec2::new(-r, 0.),
                ],
                color,
                Stroke::NONE,
            ));
            painter.line_segment(
                [p + Vec2::new(-r * 0.45, 0.), p + Vec2::new(0., -r * 0.45)],
                Stroke::new(cell * 0.055, Color32::from_white_alpha(180)),
            );
            painter.line_segment(
                [p + Vec2::new(0., -r), p + Vec2::new(r * 0.45, -r * 1.45)],
                Stroke::new(cell * 0.08, head),
            );
        }
        painter.text(
            Pos2::new(board.left(), board.bottom() + 10.),
            egui::Align2::LEFT_TOP,
            "24 × 20   /   CLASSIC BOUNDARY",
            egui::FontId::monospace(10.),
            self.theme.foreground.gamma_multiply(0.65),
        );
        painter.text(
            Pos2::new(board.right(), board.bottom() + 10.),
            egui::Align2::RIGHT_TOP,
            format!("{:03} / 480 CELLS", self.sim.body.len()),
            egui::FontId::monospace(10.),
            self.theme.foreground.gamma_multiply(0.65),
        );
        ui.allocate_space(Vec2::new(available.x, size.y + 32.));
        board
    }
    fn overlay(&mut self, ctx: &egui::Context, board: Rect) {
        if self.gate == Gate::Running {
            return;
        }
        egui::Window::new("Snake run")
            .id(egui::Id::new("snake-run"))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .fixed_pos(board.center() - Vec2::new(170., 115.))
            .fixed_size([340., 230.])
            .show(ctx, |ui| {
                ui.add_space(8.);
                ui.vertical_centered(|ui| {
                    let title = match self.gate {
                        Gate::Ready => "A little room to grow.",
                        Gate::Paused => {
                            if self.restored {
                                "Your run is waiting."
                            } else {
                                "Take a breath."
                            }
                        }
                        Gate::Countdown(_) => "Ready when you are.",
                        Gate::Result => {
                            if self.sim.outcome == Outcome::Won {
                                "Every square. Yours."
                            } else {
                                "End of the line."
                            }
                        }
                        _ => "",
                    };
                    ui.heading(title);
                    ui.add_space(9.);
                    match self.gate {
                        Gate::Ready => {
                            ui.label("Eat. Grow. Leave yourself a way out.");
                            ui.add_space(10.);
                            ui.horizontal(|ui| {
                                for speed in Speed::ALL {
                                    if ui
                                        .selectable_label(
                                            self.sim.speed == speed,
                                            format!("{} · {}", speed.label(), 60 / speed.period()),
                                        )
                                        .clicked()
                                    {
                                        self.change_speed(speed);
                                    }
                                }
                            });
                            ui.add_space(10.);
                            if ui
                                .add_sized([240., 36.], egui::Button::new("Start game    Enter"))
                                .clicked()
                            {
                                self.enter();
                            }
                            ui.label(RichText::new("1 / 2 / 3 to choose speed").small().weak());
                        }
                        Gate::Paused => {
                            ui.label(if self.restored {
                                "Restored run · local records only"
                            } else {
                                "The clock is stopped."
                            });
                            ui.add_space(10.);
                            if ui
                                .add_sized([240., 36.], egui::Button::new("Continue    Enter"))
                                .clicked()
                            {
                                self.enter();
                            }
                            if ui.button("New Game    N").clicked() {
                                self.restart = true;
                            }
                        }
                        Gate::Countdown(left) => {
                            ui.heading(
                                RichText::new(format!(
                                    "{}",
                                    left.as_secs() + u64::from(left.subsec_nanos() > 0)
                                ))
                                .size(44.),
                            );
                            ui.label("Fresh direction keys after the countdown.");
                            if ui.button("Cancel    Esc").clicked() {
                                self.pause();
                            }
                        }
                        Gate::Result => {
                            ui.heading(
                                RichText::new(format!("{} points", self.sim.score)).size(32.),
                            );
                            ui.label(format!(
                                "{} · Best {}",
                                self.sim.speed.label(),
                                self.records.best[self.sim.speed.index()]
                            ));
                            if ui
                                .add_sized([240., 34.], egui::Button::new("Play Again    Enter"))
                                .clicked()
                            {
                                self.fresh(true);
                            }
                            if ui.button("Change Speed").clicked() {
                                self.fresh(false);
                            }
                        }
                        _ => (),
                    }
                    ui.add_space(6.);
                    if ui.button("Return to Arcade").clicked() {
                        self.finished = true;
                    }
                });
                ui.add_space(6.);
            });
    }
}
impl eframe::App for SnakeApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        arcade_presentation::apply(ctx);
        self.theme(ctx);
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.last);
        self.last = now;
        let previous = self.gate;
        self.inputs(ctx);
        if previous == Gate::Running && self.gate == Gate::Running {
            for _ in 0..self.clock.ticks(elapsed) {
                let score = self.sim.score;
                self.sim.advance();
                if self.sim.score > score {
                    self.feedback = Duration::from_millis(180);
                    if self.records.preferences.audio {
                        self.audio.play(false);
                    }
                    self.records.record(&self.sim);
                }
                if self.sim.outcome != Outcome::Playing {
                    self.gate = Gate::Result;
                    if let Some(r) = &mut self.trace {
                        r.end_tick = self.sim.tick;
                    }
                    if self.records.preferences.audio {
                        self.audio.play(self.sim.outcome == Outcome::Collision);
                    }
                    self.persist();
                    break;
                }
            }
        } else if matches!(previous, Gate::Countdown(_)) && self.gate.elapse(elapsed) {
            self.record(Action::Resume);
            self.clock.reset();
        }
        self.feedback = self.feedback.saturating_sub(elapsed);
        let mut board = Rect::NOTHING;
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(self.theme.background)
                    .inner_margin(24.),
            )
            .show(ctx, |ui| {
                arcade_presentation::backdrop(ui);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("ARCADE /  SNAKE")
                                .monospace()
                                .small()
                                .color(self.theme.accent),
                        );
                        ui.heading(RichText::new("Room to grow.").size(34.));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        for (label, value) in [
                            (
                                "LOCAL BEST",
                                self.records.best[self.sim.speed.index()].to_string(),
                            ),
                            ("SCORE", self.sim.score.to_string()),
                            ("SPEED", self.sim.speed.label().to_string()),
                        ] {
                            ui.allocate_ui(Vec2::new(130., 62.), |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(RichText::new(label).monospace().small().weak());
                                    ui.label(RichText::new(value).size(25.).strong());
                                });
                            });
                        }
                    });
                });
                ui.add_space(10.);
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(self.gate == Gate::Running, egui::Button::new("Pause   Esc"))
                        .clicked()
                    {
                        self.pause();
                    }
                    if ui.button("Restart   R").clicked() {
                        self.pause();
                        self.restart = true;
                    }
                    if ui.button("Settings").clicked() {
                        self.pause();
                        self.settings = true;
                    }
                    ui.separator();
                    ui.label("Arrows / WASD");
                    ui.label(
                        RichText::new("Community leaderboard: not available for Snake yet")
                            .small()
                            .weak(),
                    );
                    if !self.records.preferences.reduced_motion && !self.feedback.is_zero() {
                        ui.label(RichText::new("+10").strong().color(self.theme.accent));
                    }
                });
                ui.add_space(16.);
                board = self.board(ui);
            });
        if !self.settings && !self.restart {
            self.overlay(ctx, board);
        }
        if self.settings {
            egui::Window::new("Snake settings")
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.checkbox(&mut self.records.preferences.audio, "Short sound cues");
                    if !self.records.preferences.audio {
                        self.audio.stop();
                    }
                    ui.checkbox(
                        &mut self.records.preferences.reduced_motion,
                        "Reduce motion and score feedback",
                    );
                    ui.separator();
                    ui.label("Direction keys (WASD always works)");
                    for (i, label) in ["Up", "Right", "Down", "Left"].into_iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(label);
                            if ui
                                .button(if self.rebind == Some(i) {
                                    "Press a key…"
                                } else {
                                    &self.records.preferences.keys[i]
                                })
                                .clicked()
                            {
                                self.rebind = Some(i);
                            }
                        });
                    }
                    ui.label("Enter, Esc, F11, N, R, 1–3 and WASD are reserved.");
                    if ui.button("Reset direction keys").clicked() {
                        self.records.preferences.keys =
                            ["ArrowUp", "ArrowRight", "ArrowDown", "ArrowLeft"].map(str::to_owned);
                        self.rebind = None;
                    }
                    ui.separator();
                    ui.label("Local play works offline. Sounds use the system audio output.");
                    if ui.button("Done").clicked() {
                        self.settings = false;
                        self.rebind = None;
                        self.persist();
                    }
                });
        }
        if self.restart {
            egui::Window::new("Start a new run?")
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("This replaces the current run. Your local best is kept.");
                    if ui.button("New Game").clicked() {
                        self.fresh(false);
                    }
                    if ui.button("Keep this run").clicked() {
                        self.restart = false;
                    }
                });
        }
        if let Some(error) = self.error.clone() {
            egui::Window::new("Snake save notice")
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label(error);
                    if ui.button("OK").clicked() {
                        self.error = None;
                    }
                });
        }
        if self.save_at.elapsed() > Duration::from_secs(2) {
            self.persist();
        }
        ctx.request_repaint_after(Duration::from_millis(16));
    }
    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        self.audio.stop();
        self.persist();
    }
}

#[cfg(test)]
mod save_protection_tests {
    use super::*;
    use eframe::App as _;

    #[test]
    fn failed_archives_block_restart_save_and_exit() {
        for filename in ["session.json", "records.json"] {
            for original in [
                b"broken".to_vec(),
                br#"{"version":99}"#.to_vec(),
                vec![b' '; 16385],
            ] {
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join(filename);
                std::fs::write(&path, &original).unwrap();
                std::fs::write(dir.path().join("archive"), b"archive blocker").unwrap();
                let mut app = SnakeApp::from_dir(dir.path().to_owned());
                assert!(!app.writable);
                app.records.preferences.audio = true;
                app.fresh(false);
                app.persist();
                assert!(app.prepare_to_leave().is_err());
                assert!(app.error.is_some(), "Keep recovery instructions after Stay");
                app.on_exit(None);
                assert_eq!(std::fs::read(&path).unwrap(), original);
                std::fs::remove_file(dir.path().join("archive")).unwrap();
                app.persist();
                assert_eq!(std::fs::read(&path).unwrap(), original);
                let mut reopened = SnakeApp::from_dir(dir.path().to_owned());
                assert!(reopened.writable);
                reopened.on_exit(None);
                let copies: Vec<_> = std::fs::read_dir(dir.path().join("archive"))
                    .unwrap()
                    .collect();
                assert_eq!(copies.len(), 1);
                assert_eq!(
                    std::fs::read(copies[0].as_ref().unwrap().path()).unwrap(),
                    original
                );
            }
        }
    }

    #[test]
    fn new_and_valid_saves_remain_writable() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = SnakeApp::from_dir(dir.path().to_owned());
        app.records.preferences.audio = true;
        app.on_exit(None);
        let reopened = SnakeApp::from_dir(dir.path().to_owned());
        assert!(reopened.writable);
        assert!(reopened.records.preferences.audio);
        assert!(reopened.restored);
    }
}
