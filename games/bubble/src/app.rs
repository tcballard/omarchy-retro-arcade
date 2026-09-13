use crate::{
    audio::Audio,
    rules::{self, Board, Point, Shot, Status, DANGER, LAUNCH, R},
    storage::{self, Progress},
};
use eframe::egui::{self, Align2, Color32, FontId, Key, Pos2, Rect, Sense, Stroke, Vec2};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
const COLORS: [Color32; 6] = [
    Color32::from_rgb(247, 127, 104),
    Color32::from_rgb(100, 202, 200),
    Color32::from_rgb(239, 201, 105),
    Color32::from_rgb(165, 150, 236),
    Color32::from_rgb(134, 200, 126),
    Color32::from_rgb(233, 150, 199),
];
struct Flight {
    shot: Shot,
    distance: f64,
}
struct Particle {
    point: Point,
    color: u8,
    age: f32,
    fall: bool,
}
pub struct BubbleApp {
    pub board: Board,
    levels: Vec<rules::Level>,
    progress: Progress,
    path: PathBuf,
    readonly: bool,
    error: Option<String>,
    angle: f64,
    flight: Option<Flight>,
    particles: Vec<Particle>,
    paused: bool,
    intro: bool,
    finished: bool,
    audio: Audio,
    theme: omarchy_chess::theme::Theme,
    themed: Instant,
}
impl Default for BubbleApp {
    fn default() -> Self {
        Self::new()
    }
}
impl BubbleApp {
    pub fn prepare_to_leave(&mut self) -> Result<(), String> {
        self.finished = false;
        self.suspend();
        if self.readonly {
            return Err("The original Bubble save needs recovery before saving is allowed.".into());
        }
        storage::save(&self.path, &self.progress).map_err(|e| e.to_string())
    }
    pub fn new() -> Self {
        Self::with_path(storage::state_path())
    }
    pub fn with_path(path: PathBuf) -> Self {
        let levels = rules::levels();
        let (progress, error) = match storage::load(&path, levels.len()) {
            Ok(p) => (p, None),
            Err(e) => (Progress::default(), Some(e)),
        };
        Self {
            board: Board::new(&levels[progress.selected]).expect("verified level"),
            levels,
            intro: !progress.introduced,
            progress,
            path,
            readonly: error.is_some(),
            error,
            angle: 0.,
            flight: None,
            particles: vec![],
            paused: false,
            finished: false,
            audio: Audio::default(),
            theme: omarchy_chess::theme::Theme::load(),
            themed: Instant::now(),
        }
    }
    pub fn finished(&self) -> bool {
        self.finished
    }
    pub fn suspend(&mut self) {
        self.paused = true;
        self.audio.stop();
    }
    fn persist(&mut self) {
        if !self.readonly {
            self.error = storage::save(&self.path, &self.progress).err();
        }
    }
    fn start(&mut self, level: usize) {
        self.audio.stop();
        self.progress.selected = level.min(self.progress.unlocked);
        self.board = Board::new(&self.levels[self.progress.selected]).expect("verified level");
        self.flight = None;
        self.particles.clear();
        self.paused = false;
        self.angle = 0.;
        self.persist();
    }
    fn begin_shot(&mut self) {
        if self.flight.is_none() && self.board.status == Status::Playing {
            self.flight = self
                .board
                .trace(self.angle)
                .map(|shot| Flight { shot, distance: 0. });
            if self.progress.sound {
                self.audio.play(0);
            }
        }
    }
    fn advance(&mut self, dt: f32) {
        if self.paused || self.intro {
            return;
        }
        for p in &mut self.particles {
            p.age += dt;
        }
        self.particles.retain(|p| p.age < 0.65);
        let Some(flight) = self.flight.as_mut() else {
            return;
        };
        flight.distance += dt as f64 * 900.;
        let total: f64 = flight
            .shot
            .points
            .windows(2)
            .map(|w| w[0].distance(w[1]))
            .sum();
        if flight.distance < total {
            return;
        }
        let shot = self.flight.take().unwrap().shot;
        let out = self.board.attach(&shot);
        if self.progress.sound && (!out.popped.is_empty() || self.board.status != Status::Playing) {
            self.audio.play(if self.board.status == Status::Won {
                2
            } else {
                1
            });
        }
        self.particles
            .extend(out.popped.into_iter().map(|(point, color)| Particle {
                point,
                color,
                age: 0.,
                fall: false,
            }));
        self.particles
            .extend(out.fallen.into_iter().map(|(point, color)| Particle {
                point,
                color,
                age: 0.,
                fall: true,
            }));
        let selected = self.progress.selected;
        self.progress.best[selected] = self.progress.best[selected].max(self.board.score);
        if self.board.status == Status::Won {
            self.progress.unlocked = self
                .progress
                .unlocked
                .max((selected + 1).min(self.levels.len() - 1));
        }
        self.persist();
    }
    fn canvas(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, dt: f32, enabled: bool) {
        let space = ui.available_rect_before_wrap();
        let scale = (space.width() / 1000.).min(space.height() / 730.);
        let area = Rect::from_center_size(space.center(), Vec2::new(1000., 730.) * scale);
        let painter = ui.painter_at(area);
        let at = |x: f32, y: f32| area.min + Vec2::new(x, y) * scale;
        let world = |p: Point| at(322. + p.x as f32, 30. + p.y as f32);
        let text = |x: f32, y: f32, s: &str, size: f32, color: Color32| {
            painter.text(
                at(x, y),
                Align2::LEFT_TOP,
                s,
                FontId::proportional(size * scale),
                color,
            );
        };
        let muted = self.theme.foreground.gamma_multiply(0.58);
        text(28., 36., "OMARCHY ARCADE / BUBBLE", 12., self.theme.accent);
        text(25., 69., "Bubble.", 58., self.theme.foreground);
        text(29., 144., "A little room to clear your head.", 14., muted);
        painter.line_segment(
            [at(29., 185.), at(264., 185.)],
            Stroke::new(scale, self.theme.foreground.gamma_multiply(0.16)),
        );
        text(
            29.,
            216.,
            &format!(
                "LEVEL {:02} / {:02}",
                self.progress.selected + 1,
                self.levels.len()
            ),
            13.,
            self.theme.accent,
        );
        text(
            29.,
            248.,
            &self.levels[self.progress.selected].name,
            23.,
            self.theme.foreground,
        );
        text(29., 306., "SCORE", 11., muted);
        text(
            29.,
            330.,
            &format!("{:06}", self.board.score),
            38.,
            self.theme.foreground,
        );
        text(29., 399., "PERSONAL BEST", 11., muted);
        text(
            29.,
            422.,
            &format!("{:06}", self.progress.best[self.progress.selected]),
            24.,
            self.theme.foreground,
        );
        text(29., 518., "MAKE THREE. LET GO.", 12., self.theme.accent);
        text(29., 548., "Match a colour to pop a cluster.", 14., muted);
        text(29., 572., "Unhook it to drop everything below.", 14., muted);
        text(29., 640., "MOUSE  Aim & fire", 12., muted);
        text(29., 661., "LEFT / RIGHT  Aim     SPACE  Fire", 12., muted);
        text(29., 682., "SHIFT  Fine aim     ESC  Pause", 12., muted);
        // A fixed high-contrast playfield nested inside the current Omarchy palette.
        let field = Rect::from_min_max(at(306., 32.), at(738., 716.));
        arcade_presentation::bezel(&painter, field, self.theme.accent);
        painter.rect_filled(field, 0., Color32::from_rgb(12, 23, 26));
        painter.rect_stroke(
            field,
            16. * scale,
            Stroke::new(scale, Color32::from_rgb(62, 78, 87)),
            egui::StrokeKind::Inside,
        );
        for row in 0..18 {
            for col in 0..11 {
                painter.circle_filled(
                    at(322. + col as f32 * 40., 56. + row as f32 * 34.6),
                    0.8 * scale,
                    Color32::from_rgb(36, 50, 61),
                );
            }
        }
        let ceiling = self.board.center(0).y as f32 - 20.;
        painter.rect_filled(
            Rect::from_min_max(at(321., 44.), at(723., 30. + ceiling)),
            3. * scale,
            Color32::from_rgb(47, 64, 72),
        );
        painter.line_segment(
            [at(322., 30. + ceiling), at(722., 30. + ceiling)],
            Stroke::new(2. * scale, Color32::from_rgb(130, 157, 161)),
        );
        let danger = 30. + DANGER as f32;
        for x in (326..719).step_by(15) {
            painter.line_segment(
                [at(x as f32, danger), at(x as f32 + 7., danger)],
                Stroke::new(scale, Color32::from_rgb(165, 104, 99)),
            );
        }
        painter.text(
            at(522., danger + 8.),
            Align2::CENTER_TOP,
            "DANGER LINE",
            FontId::monospace(9. * scale),
            Color32::from_rgb(186, 126, 119),
        );
        let response = ui.interact(
            Rect::from_min_max(at(322., 50.), at(722., 688.)),
            ui.id().with("bubble-playfield"),
            Sense::click(),
        );
        if enabled && self.flight.is_none() {
            if let Some(pos) = response.hover_pos() {
                if ctx.input(|i| i.pointer.delta().length_sq() > 0.) || response.clicked() {
                    let target = Point {
                        x: ((pos.x - area.min.x) / scale - 322.) as f64,
                        y: ((pos.y - area.min.y) / scale - 30.) as f64,
                    };
                    if target.y < LAUNCH.y - 5. {
                        self.angle = (target.x - LAUNCH.x)
                            .atan2(LAUNCH.y - target.y)
                            .to_degrees()
                            .clamp(-78., 78.);
                    }
                }
            }
            if response.clicked() {
                self.begin_shot();
            }
        }
        if self.flight.is_none() && self.board.status == Status::Playing {
            if let Some(shot) = self.board.trace(self.angle) {
                // Stop after the first reflected segment, and limit the total guide length.
                let mut budget = 440.;
                for segment in shot.points.windows(2).take(2) {
                    let len = segment[0].distance(segment[1]);
                    let shown = len.min(budget);
                    let mut d = 12.;
                    while d < shown {
                        let f = d / len;
                        let p = Point {
                            x: segment[0].x + (segment[1].x - segment[0].x) * f,
                            y: segment[0].y + (segment[1].y - segment[0].y) * f,
                        };
                        painter.circle_filled(
                            world(p),
                            1.8 * scale,
                            COLORS[shot.color as usize].gamma_multiply(0.55),
                        );
                        d += 15.;
                    }
                    budget -= shown;
                    if budget <= 0. {
                        break;
                    }
                }
            }
        }
        for (i, c) in self.board.cells.iter().enumerate() {
            if let Some(c) = c {
                bubble(
                    &painter,
                    world(self.board.center(i)),
                    R as f32 * scale,
                    *c,
                    1.,
                );
            }
        }
        for p in &self.particles {
            let mut point = p.point;
            if p.fall {
                point.y += 350. * (p.age as f64).powi(2);
                point.x += (point.x - 200.) * p.age as f64 * 0.12;
            } else {
                point.y -= p.age as f64 * 28.;
            }
            bubble(
                &painter,
                world(point),
                R as f32 * scale * (if p.fall { 1. } else { 1. + p.age * 0.35 }),
                p.color,
                1. - p.age / 0.65,
            );
        }
        let launcher = world(LAUNCH);
        painter.circle_filled(
            launcher + Vec2::new(0., 6. * scale),
            38. * scale,
            Color32::from_rgb(9, 17, 23),
        );
        painter.circle_stroke(
            launcher,
            34. * scale,
            Stroke::new(3. * scale, Color32::from_rgb(110, 131, 139)),
        );
        let aim = Vec2::new(
            self.angle.to_radians().sin() as f32,
            -self.angle.to_radians().cos() as f32,
        );
        painter.line_segment(
            [launcher + aim * 24. * scale, launcher + aim * 46. * scale],
            Stroke::new(13. * scale, Color32::from_rgb(113, 137, 146)),
        );
        painter.line_segment(
            [launcher + aim * 25. * scale, launcher + aim * 45. * scale],
            Stroke::new(6. * scale, Color32::from_rgb(25, 41, 53)),
        );
        if let Some(flight) = &self.flight {
            let mut remaining = flight.distance;
            for s in flight.shot.points.windows(2) {
                let len = s[0].distance(s[1]);
                if remaining <= len {
                    let f = remaining / len.max(1e-9);
                    bubble(
                        &painter,
                        world(Point {
                            x: s[0].x + (s[1].x - s[0].x) * f,
                            y: s[0].y + (s[1].y - s[0].y) * f,
                        }),
                        R as f32 * scale,
                        flight.shot.color,
                        1.,
                    );
                    break;
                }
                remaining -= len;
            }
        } else if let Some(c) = self.board.color(0) {
            bubble(&painter, launcher, 20. * scale, c, 1.);
        }
        painter.text(
            at(522., 697.),
            Align2::CENTER_CENTER,
            "ONE GOOD SHOT AT A TIME",
            FontId::monospace(9. * scale),
            Color32::from_rgb(136, 158, 166),
        );
        text(780., 128., "UP NEXT", 11., muted);
        if let Some(c) = self.board.color(1) {
            bubble(&painter, at(805., 184.), 23. * scale, c, 1.);
        }
        text(780., 259., "CEILING DROPS IN", 11., muted);
        text(
            780.,
            287.,
            &format!("{} shots", self.board.pressure_in()),
            27.,
            if self.board.pressure_in() <= 2 {
                COLORS[0]
            } else {
                self.theme.foreground
            },
        );
        for i in 0..self.board.pressure_every {
            let x = 780. + (i % 6) as f32 * 24.;
            let y = 335. + (i / 6) as f32 * 14.;
            painter.rect_filled(
                Rect::from_min_size(at(x, y), Vec2::new(17., 5.) * scale),
                1.,
                if i < self.board.pressure_in() {
                    self.theme.accent
                } else {
                    self.theme.foreground.gamma_multiply(0.12)
                },
            );
        }
        text(780., 399., "ON THE BOARD", 11., muted);
        text(
            780.,
            427.,
            &format!("{} bubbles", self.board.count()),
            23.,
            self.theme.foreground,
        );
        let button = |ui: &mut egui::Ui, x: f32, y: f32, w: f32, label: &str| {
            ui.put(
                Rect::from_min_size(at(x, y), Vec2::new(w, 36.) * scale),
                egui::Button::new(egui::RichText::new(label).size(14. * scale)),
            )
            .clicked()
        };
        if button(ui, 780., 36., 176., "Pause · Esc") {
            self.suspend();
        }
        if button(
            ui,
            780.,
            532.,
            176.,
            if self.progress.sound {
                "Sound: on"
            } else {
                "Sound: off"
            },
        ) {
            self.progress.sound = !self.progress.sound;
            self.audio.stop();
            self.persist();
        }
        if button(ui, 780., 579., 176., "How to play") {
            self.intro = true;
            self.audio.stop();
        }
        if button(ui, 780., 626., 176., "Choose level") {
            self.suspend();
        }
        if enabled && !self.paused && !self.intro {
            self.advance(dt);
        }
    }
}
fn bubble(p: &egui::Painter, c: Pos2, r: f32, color: u8, alpha: f32) {
    let base = COLORS[color as usize];
    let tint = |v: Color32| v.gamma_multiply(alpha.clamp(0., 1.));
    arcade_presentation::glass_ball(p, c, r, tint(base));
    let ink = tint(Color32::from_rgb(34, 46, 55));
    let stroke = Stroke::new(r * 0.115, ink);
    let vertices = |n: usize, rotation: f32, size: f32| {
        (0..n)
            .map(|i| {
                let a = rotation + i as f32 * std::f32::consts::TAU / n as f32;
                c + Vec2::new(a.cos(), a.sin()) * r * size
            })
            .collect::<Vec<_>>()
    };
    match color {
        0 => {
            p.circle_stroke(c + Vec2::new(0., 0.08) * r, r * 0.28, stroke);
        }
        1 => {
            p.add(egui::Shape::closed_line(vertices(4, 0., 0.38), stroke));
        }
        2 => {
            p.add(egui::Shape::closed_line(
                vertices(3, -std::f32::consts::FRAC_PI_2, 0.38),
                stroke,
            ));
        }
        3 => {
            for a in [-1., 1.] {
                p.line_segment(
                    [
                        c + Vec2::new(-0.24, a * 0.24) * r,
                        c + Vec2::new(0.24, -a * 0.24) * r,
                    ],
                    stroke,
                );
            }
        }
        4 => {
            p.add(egui::Shape::closed_line(
                vertices(4, std::f32::consts::FRAC_PI_4, 0.36),
                stroke,
            ));
        }
        _ => {
            for dx in [-0.16, 0.16] {
                p.line_segment(
                    [c + Vec2::new(dx, -0.27) * r, c + Vec2::new(dx, 0.27) * r],
                    stroke,
                );
            }
        }
    }
}
impl eframe::App for BubbleApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        arcade_presentation::apply(ctx);
        if self.themed.elapsed() > Duration::from_secs(2) {
            self.theme = omarchy_chess::theme::Theme::load();
            self.themed = Instant::now();
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
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL, Key::M))
        }) {
            self.progress.sound = !self.progress.sound;
            self.audio.stop();
            self.persist();
        }
        let focused = ctx.input(|i| i.focused);
        if !focused {
            self.suspend();
        }
        let dt = ctx.input(|i| i.stable_dt).min(0.05);
        let escape = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape));
        if escape && !self.intro {
            self.paused = !self.paused;
            self.audio.stop();
        }
        let enabled =
            focused && !self.paused && !self.intro && self.board.status == Status::Playing;
        if enabled {
            ctx.input(|i| {
                let speed = if i.modifiers.shift { 20. } else { 75. };
                if i.key_down(Key::ArrowLeft) {
                    self.angle -= dt as f64 * speed;
                }
                if i.key_down(Key::ArrowRight) {
                    self.angle += dt as f64 * speed;
                }
            });
            self.angle = self.angle.clamp(-78., 78.);
            if ctx.input_mut(|i| {
                i.consume_key(egui::Modifiers::NONE, Key::Space)
                    || i.consume_key(egui::Modifiers::NONE, Key::Enter)
            }) {
                self.begin_shot();
            }
        }
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(self.theme.background)
                    .inner_margin(10.),
            )
            .show(ctx, |ui| {
                arcade_presentation::backdrop(ui);
                self.canvas(ui, ctx, dt, enabled);
            });
        if self.intro {
            egui::Window::new("A little introduction").collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER,Vec2::ZERO).default_width(380.).show(ctx,|ui| {
                ui.heading("Make three. Clear the ceiling.");ui.add_space(8.);
                ui.label("Aim with the mouse and click to shoot, or use Left/Right and Space. Hold Shift for fine keyboard aiming.");ui.add_space(8.);
                ui.label("Three touching bubbles of one colour pop. Anything no longer attached to the ceiling falls away. Every colour also has its own symbol.");ui.add_space(8.);
                ui.label("Bounce off the walls. Watch the shot counter: the ceiling moves down when it reaches zero. Keep bubbles above the danger line.");ui.add_space(8.);
                ui.label("Escape pauses. Progress and personal bests save automatically. Reopening a level starts a fresh attempt.");ui.add_space(12.);
                if ui.button("Let's play · Enter").clicked() || ctx.input_mut(|i|i.consume_key(egui::Modifiers::NONE,Key::Enter)) {self.intro=false;self.paused=false;self.progress.introduced=true;self.persist();}
            });
        } else if self.paused {
            egui::Window::new("Take your time.")
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                .default_width(320.)
                .show(ctx, |ui| {
                    ui.label("Paused. Your shot will wait.");
                    ui.add_space(8.);
                    if ui.button("Resume · Esc").clicked() {
                        self.paused = false;
                    }
                    if ui.button("Restart level").clicked() {
                        self.start(self.progress.selected);
                    }
                    let mut selected = self.progress.selected;
                    egui::ComboBox::from_label("Level")
                        .selected_text(format!(
                            "{:02} · {}",
                            selected + 1,
                            self.levels[selected].name
                        ))
                        .show_ui(ui, |ui| {
                            for (i, l) in self.levels.iter().enumerate() {
                                ui.add_enabled_ui(i <= self.progress.unlocked, |ui| {
                                    ui.selectable_value(
                                        &mut selected,
                                        i,
                                        format!(
                                            "{:02} · {}{}",
                                            i + 1,
                                            l.name,
                                            if i > self.progress.unlocked {
                                                " (locked)"
                                            } else {
                                                ""
                                            }
                                        ),
                                    );
                                });
                            }
                        });
                    if selected != self.progress.selected {
                        self.start(selected);
                    }
                    ui.add_space(10.);
                    if ui.button("Return to Arcade").clicked() {
                        self.audio.stop();
                        self.finished = true;
                    }
                });
        } else if self.board.status != Status::Playing {
            let won = self.board.status == Status::Won;
            egui::Window::new(if won {
                "Room to breathe."
            } else {
                "One more go?"
            })
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .default_width(300.)
            .show(ctx, |ui| {
                ui.heading(if won {
                    "Board cleared!"
                } else {
                    "The bubbles reached the line."
                });
                ui.label(format!(
                    "{} points · {} shots",
                    self.board.score, self.board.shots
                ));
                if won
                    && self.progress.selected + 1 < self.levels.len()
                    && ui.button("Next level · Enter").clicked()
                {
                    self.start(self.progress.selected + 1);
                }
                if won && self.progress.selected + 1 == self.levels.len() {
                    ui.label("Every level cleared. Nicely done.");
                }
                if ui.button("Play this level again").clicked() {
                    self.start(self.progress.selected);
                }
                if ui.button("Return to Arcade").clicked() {
                    self.audio.stop();
                    self.finished = true;
                }
            });
            if won
                && self.progress.selected + 1 < self.levels.len()
                && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter))
            {
                self.start(self.progress.selected + 1);
            }
        }
        if let Some(error) = &self.error {
            egui::Window::new("Progress could not be saved")
                .anchor(Align2::CENTER_BOTTOM, Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label(error);
                    ui.label("Your existing save has been kept. This attempt can still be played.");
                });
        }
        if enabled || !self.particles.is_empty() {
            ctx.request_repaint_after(Duration::from_millis(16));
        } else {
            ctx.request_repaint_after(Duration::from_millis(250));
        }
    }
    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        self.audio.stop();
        self.persist();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pause_freezes_flight_then_resume_commits_once_and_restart_resets() {
        let tmp = tempfile::tempdir().unwrap();
        let mut app = BubbleApp::with_path(tmp.path().join("bubble.json"));
        app.intro = false;
        app.progress.sound = false;
        app.begin_shot();
        app.advance(0.1);
        let before = app.flight.as_ref().unwrap().distance;
        app.suspend();
        for _ in 0..50 {
            app.advance(0.05);
        }
        assert_eq!(before, app.flight.as_ref().unwrap().distance);
        assert_eq!(app.board.shots, 0);
        app.paused = false;
        for _ in 0..100 {
            app.advance(0.05);
        }
        assert_eq!(app.board.status, Status::Won);
        assert_eq!(app.board.shots, 1);
        assert_eq!(app.progress.unlocked, 1);
        let score = app.board.score;
        app.advance(10.);
        assert_eq!(score, app.board.score);
        app.start(0);
        assert_eq!(app.board.shots, 0);
        assert_eq!(app.board.score, 0);
        assert_eq!(app.board.status, Status::Playing);
        assert!(app.flight.is_none());
        assert!(app.particles.is_empty());
        let read = storage::load(&app.path, 20).unwrap();
        assert_eq!(read.best[0], 1300);
        assert_eq!(read.unlocked, 1);
    }
    #[test]
    fn first_play_blocks_simulation_and_corrupt_save_survives_exit() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("bubble.json");
        std::fs::write(&path, "bad data").unwrap();
        let mut app = BubbleApp::with_path(path.clone());
        app.progress.sound = false;
        app.begin_shot();
        app.advance(5.);
        assert_eq!(app.board.shots, 0);
        app.persist();
        assert_eq!(std::fs::read_to_string(path).unwrap(), "bad data");
    }
}
