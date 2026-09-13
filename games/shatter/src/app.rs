use crate::{
    audio::Audio,
    levels::*,
    rules::*,
    storage::{self, Save},
};
use eframe::egui::{self, Align2, FontId, Key, Pos2, Rect, Stroke, Vec2};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
#[derive(Clone, Copy, PartialEq)]
enum Panel {
    Play,
    Pause,
    Settings,
    Help,
    Practice,
    Restart,
    Recovery,
}
#[derive(Default)]
struct Controls {
    pointer: Option<Pos2>,
    mouse: bool,
    armed: bool,
}
impl Controls {
    fn clear(&mut self) {
        self.armed = false;
        self.pointer = None;
        self.mouse = false;
    }
    fn input(&mut self, ctx: &egui::Context, rect: Rect, scale: f32) -> Input {
        ctx.input(|i| {
            let held = i.key_down(Key::Space)
                || i.pointer.primary_down()
                || [Key::ArrowLeft, Key::ArrowRight, Key::A, Key::D]
                    .iter()
                    .any(|k| i.key_down(*k));
            if !self.armed {
                self.pointer = i.pointer.hover_pos();
                if !held {
                    self.armed = true;
                }
                return Input::default();
            }
            let directional = [Key::ArrowLeft, Key::ArrowRight, Key::A, Key::D]
                .iter()
                .any(|k| i.key_pressed(*k));
            let pos = i.pointer.hover_pos();
            if directional {
                self.mouse = false;
            } else if pos != self.pointer && pos.is_some_and(|p| rect.contains(p)) {
                self.mouse = true;
            }
            self.pointer = pos;
            let left = i.key_down(Key::ArrowLeft) || i.key_down(Key::A);
            let right = i.key_down(Key::ArrowRight) || i.key_down(Key::D);
            Input {
                direction: i8::from(right) - i8::from(left),
                target: if self.mouse {
                    pos.filter(|p| rect.contains(*p))
                        .map(|p| f64::from((p.x - rect.left()) / scale))
                } else {
                    None
                },
                launch: i.key_pressed(Key::Space)
                    || (i.pointer.primary_pressed() && pos.is_some_and(|p| rect.contains(p))),
                fire: i.key_down(Key::Space)
                    || (i.pointer.primary_down() && pos.is_some_and(|p| rect.contains(p))),
            }
        })
    }
}
pub struct App {
    save: Save,
    practice: Option<Run>,
    path: PathBuf,
    blocked: bool,
    error: Option<String>,
    panel: Panel,
    controls: Controls,
    clock: Clock,
    last: Instant,
    checkpoint: Instant,
    theme: omarchy_chess::theme::Theme,
    themed: Instant,
    audio: Audio,
    leave: bool,
    enabled: bool,
    cue: Option<(String, Instant)>,
    fragments: Vec<(V, Instant)>,
    trails: Vec<V>,
    pending_launch: bool,
}
impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
impl App {
    pub fn new() -> Self {
        let path = storage::path();
        let exists = path.exists();
        let (save, error) = match storage::load(&path) {
            Ok(s) => (s, None),
            Err(e) => (Save::default(), Some(e)),
        };
        let blocked = error.is_some();
        Self {
            save,
            practice: None,
            path,
            blocked,
            error,
            panel: if exists { Panel::Pause } else { Panel::Play },
            controls: Controls::default(),
            clock: Clock::default(),
            last: Instant::now(),
            checkpoint: Instant::now(),
            theme: omarchy_chess::theme::Theme::load(),
            themed: Instant::now(),
            audio: Audio::default(),
            leave: false,
            enabled: true,
            cue: None,
            fragments: vec![],
            trails: vec![],
            pending_launch: false,
        }
    }
    fn run(&self) -> &Run {
        self.practice.as_ref().unwrap_or(&self.save.campaign)
    }
    fn run_mut(&mut self) -> &mut Run {
        self.practice.as_mut().unwrap_or(&mut self.save.campaign)
    }
    fn persist(&mut self) {
        if !self.blocked {
            self.save.observe();
            if let Err(e) = storage::write(&self.path, &self.save) {
                self.error = Some(format!("Progress could not be saved: {e}"));
            }
        }
        self.checkpoint = Instant::now();
    }
    pub fn suspend(&mut self) {
        self.panel = Panel::Pause;
        self.controls.clear();
        self.pending_launch = false;
        self.clock.remainder = 0.;
        self.audio.stop();
        self.persist();
    }
    pub fn set_input_enabled(&mut self, enabled: bool) {
        if self.enabled && !enabled {
            self.suspend();
        }
        self.enabled = enabled;
    }
    pub fn finished(&self) -> bool {
        self.leave
    }
    fn open(&mut self, panel: Panel) {
        self.suspend();
        self.panel = panel;
    }
    fn resume(&mut self) {
        self.panel = Panel::Play;
        self.controls.clear();
        self.pending_launch = false;
        self.clock.remainder = 0.;
        self.last = Instant::now();
    }
    fn restart(&mut self) {
        if self.practice.is_some() {
            self.practice = Some(Run::new(self.run().level, true, self.run().rng));
        } else {
            self.save.campaign = Run::new(0, false, self.save.campaign.rng);
        }
        self.fragments.clear();
        self.trails.clear();
        self.persist();
        self.resume();
    }
    fn render(&self, ui: &egui::Ui, rect: Rect, scale: f32) {
        let p = ui.painter_at(rect.expand(14.));
        let run = self.run();
        let fg = self.theme.foreground;
        let accent = self.theme.accent;
        arcade_presentation::bezel(&p, rect, accent);
        p.rect_filled(rect, 0., self.theme.background);
        let pt = |v: V| {
            Pos2::new(
                rect.left() + v.x as f32 * scale,
                rect.top() + v.y as f32 * scale,
            )
        };
        let box_at = |x: f64, y: f64, w: f64, h: f64| {
            Rect::from_min_size(
                pt(V::new(x, y)),
                Vec2::new(w as f32 * scale, h as f32 * scale),
            )
        };
        for b in run.bricks.iter().filter(|b| b.alive()) {
            let r = box_at(b.x, b.y, BRICK_W, BRICK_H);
            let colour = if b.kind == 3 {
                fg.gamma_multiply(0.30)
            } else {
                accent.gamma_multiply(if b.kind == 2 { 0.75 } else { 0.95 })
            };
            p.rect_filled(r, 0., colour);
            p.line_segment([r.left_top(), r.right_top()], Stroke::new(scale, fg));
            if b.kind == 2 {
                p.rect_stroke(
                    r.shrink(4. * scale),
                    0.,
                    Stroke::new(scale, fg),
                    egui::StrokeKind::Inside,
                );
                if b.hp == 1 {
                    p.line_segment(
                        [r.left_top() + Vec2::new(20. * scale, 0.), r.center()],
                        Stroke::new(2. * scale, self.theme.background),
                    );
                    p.line_segment(
                        [r.center(), r.right_bottom() - Vec2::new(18. * scale, 0.)],
                        Stroke::new(2. * scale, self.theme.background),
                    );
                }
            }
            if b.kind == 3 {
                p.line_segment(
                    [
                        r.left_top() + Vec2::splat(3. * scale),
                        r.right_bottom() - Vec2::splat(3. * scale),
                    ],
                    Stroke::new(2. * scale, fg),
                );
                p.line_segment(
                    [
                        r.left_bottom() + Vec2::new(3. * scale, -3. * scale),
                        r.right_top() + Vec2::new(-3. * scale, 3. * scale),
                    ],
                    Stroke::new(2. * scale, fg),
                );
            }
        }
        let width = run.width();
        p.rect_filled(
            box_at(run.paddle - width / 2., PADDLE_Y + 4., width, 10.),
            0.,
            accent,
        );
        p.rect_filled(
            box_at(run.paddle - width / 2. + 8., PADDLE_Y, width - 16., 8.),
            0.,
            fg,
        );
        p.rect_filled(
            box_at(run.paddle - 18., PADDLE_Y + 5., 36., 5.),
            0.,
            arcade_presentation::BRASS,
        );
        if run.laser > 0 {
            for dx in [-width / 2. + 10., width / 2. - 10.] {
                p.rect_filled(
                    box_at(run.paddle + dx - 2., PADDLE_Y - 6., 4., 10.),
                    0.,
                    accent,
                );
            }
        }
        if !self.save.reduced_effects {
            for (i, v) in self.trails.iter().enumerate() {
                p.rect_filled(
                    Rect::from_center_size(pt(*v), Vec2::splat(3. * scale)),
                    0.,
                    fg.gamma_multiply(i as f32 / self.trails.len().max(1) as f32 * 0.5),
                );
            }
            for (v, time) in &self.fragments {
                let age = time.elapsed().as_secs_f32();
                for (dx, dy) in [(-1., -1.), (1., -1.), (-1., 1.), (1., 1.)] {
                    let at = pt(*v) + Vec2::new(dx, dy) * age * 65. * scale;
                    p.rect_filled(
                        Rect::from_center_size(at, Vec2::splat(3. * scale)),
                        0.,
                        accent.gamma_multiply((1. - age * 3.).max(0.)),
                    );
                }
            }
        }
        let balls: Vec<_> = if run.phase == Phase::Serve {
            vec![V::new(run.paddle, PADDLE_Y - RADIUS - 1.)]
        } else {
            run.balls.iter().map(|b| b.p).collect()
        };
        for v in balls {
            p.circle_filled(pt(v), (RADIUS as f32 + 1.) * scale, self.theme.background);
            p.circle_filled(pt(v), RADIUS as f32 * scale, fg);
        }
        for shot in &run.shots {
            p.rect_filled(box_at(shot.x - 1., shot.y, 2., 10.), 0., accent);
        }
        for cap in &run.capsules {
            let r = box_at(cap.p.x - 10., cap.p.y - 8., 20., 16.);
            p.rect_filled(r, 0., accent);
            p.text(
                r.center(),
                Align2::CENTER_CENTER,
                match cap.kind {
                    Power::Expand => "E",
                    Power::Multiball => "M",
                    Power::Laser => "L",
                },
                FontId::monospace(12. * scale),
                self.theme.background,
            );
        }
        p.text(
            pt(V::new(24., 22.)),
            Align2::LEFT_CENTER,
            format!(
                "{:07}   /   {:02}  {}",
                run.score,
                run.level + 1,
                LEVELS[run.level].name.to_uppercase()
            ),
            FontId::monospace(15. * scale),
            fg,
        );
        p.text(
            pt(V::new(776., 22.)),
            Align2::RIGHT_CENTER,
            format!("{} LIVES · {} BALLS", run.lives, run.balls.len()),
            FontId::monospace(13. * scale),
            fg,
        );
        let effects = format!(
            "{}{}",
            if run.expand > 0 {
                format!("EXPAND {:.1}s   ", run.expand as f64 / 120.)
            } else {
                String::new()
            },
            if run.laser > 0 {
                format!("LASER {:.1}s", run.laser as f64 / 120.)
            } else {
                String::new()
            }
        );
        p.text(
            pt(V::new(400., 580.)),
            Align2::CENTER_CENTER,
            effects,
            FontId::monospace(12. * scale),
            accent,
        );
        if run.phase == Phase::Serve && self.panel == Panel::Play {
            p.text(
                pt(V::new(400., 420.)),
                Align2::CENTER_CENTER,
                "MOVE: mouse / A D / ← →     LAUNCH: click / Space",
                FontId::monospace(14. * scale),
                fg,
            );
        }
        if let Some((cue, time)) = &self.cue {
            if time.elapsed() < Duration::from_secs(2) {
                p.text(
                    pt(V::new(400., 475.)),
                    Align2::CENTER_CENTER,
                    cue,
                    FontId::monospace(14. * scale),
                    accent,
                );
            }
        }
    }
    fn overlay(&mut self, ctx: &egui::Context) {
        if self.panel == Panel::Play && matches!(self.run().phase, Phase::Serve | Phase::Playing) {
            return;
        }
        let title = match self.panel {
            Panel::Settings => "Settings",
            Panel::Help => "How to Shatter",
            Panel::Practice => "Practice",
            Panel::Restart => "Restart?",
            Panel::Recovery => "Recover saved campaign",
            _ => match self.run().phase {
                Phase::Clear => "Level clear",
                Phase::Over => "Game over",
                Phase::Complete => "Campaign complete",
                _ => "Shatter · Paused",
            },
        };
        let window = egui::Window::new(title)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .default_width(350.);
        window.show(ctx, |ui| {
            match self.panel {
                Panel::Settings => {
                    ui.checkbox(&mut self.save.sound, "Sound · Ctrl+M");
                    ui.checkbox(&mut self.save.reduced_effects, "Reduced effects");
                    if !self.save.sound { self.audio.stop(); }
                    if ui.button("Back").clicked() { self.persist(); self.panel = Panel::Pause; }
                }
                Panel::Help => {
                    ui.label("Move with the mouse, arrows or A/D. Click or Space launches. Hold to fire when Laser is active.");
                    ui.label("Steer with the paddle's edges. Solid bricks: 100 points. Inset armour: two hits, 200 points. Crossed steel cannot break.");
                    ui.label("E expands for 15 seconds. M splits to three balls. L fires for 12 seconds. Lose a life only when the last ball falls.");
                    ui.label("Escape pauses. Ctrl+, settings. Ctrl+M sound. Ctrl+H Arcade. Ctrl+Q quits.");
                    ui.separator();
                    ui.label("Original Shatter geometry, layouts and synthesized audio. Built for Omarchy Arcade. Arkanoid is a gameplay reference; no assets copied.");
                    if ui.button("Back").clicked() { self.panel = Panel::Pause; }
                }
                Panel::Practice => {
                    ui.label("Separate from your campaign. Only unlocked levels.");
                    egui::Grid::new("practice-levels").num_columns(4).show(ui, |ui| {
                        for (level, data) in LEVELS.iter().enumerate() {
                            let button = egui::Button::new(format!("{:02}", level + 1));
                            if ui.add_enabled(level <= self.save.unlocked, button)
                                .on_hover_text(format!("{} · best {}", data.name, self.save.practice_best[level]))
                                .clicked() {
                                self.practice = Some(Run::new(level, true, 0x5052414354494345 + level as u64));
                                self.resume();
                            }
                            if level % 4 == 3 { ui.end_row(); }
                        }
                    });
                    if ui.button("Back").clicked() { self.panel = Panel::Pause; }
                }
                Panel::Restart => {
                    ui.label("Discard this unfinished run? Records and unlocks are kept.");
                    if ui.button("Restart run").clicked() { self.restart(); }
                    if ui.button("Keep playing").clicked() { self.resume(); }
                }
                Panel::Recovery => {
                    ui.label("Archive the original save to a unique recovery file, then start fresh? It will remain available for manual recovery.");
                    if ui.button("Archive and reset").clicked() {
                        match storage::archive(&self.path) {
                            Ok(path) => {
                                self.blocked = false;
                                self.save = Save::default();
                                self.error = Some(format!("Original archived at {}", path.display()));
                                self.persist();
                                self.resume();
                            }
                            Err(e) => self.error = Some(format!("Archive failed: {e}")),
                        }
                    }
                    if ui.button("Cancel").clicked() { self.panel = Panel::Pause; }
                }
                _ => self.run_menu(ui),
            }
        });
    }
    fn run_menu(&mut self, ui: &mut egui::Ui) {
        let mode = if self.practice.is_some() {
            "Practice"
        } else {
            "Campaign"
        };
        ui.label(format!(
            "{} · Level {} · Score {}",
            mode,
            self.run().level + 1,
            self.run().score
        ));
        ui.label(LEVELS[self.run().level].note);
        if self.blocked {
            ui.label("Continue is unavailable. The original save is protected.");
            if ui.button("Recover saved campaign…").clicked() {
                self.panel = Panel::Recovery;
            }
        } else {
            match self.run().phase {
                Phase::Serve | Phase::Playing => {
                    if ui.button("Continue").clicked() {
                        self.resume();
                    }
                }
                Phase::Clear if self.practice.is_none() => {
                    if ui.button("Next Level").clicked() {
                        self.save.campaign.next();
                        self.persist();
                        self.resume();
                    }
                }
                _ if self.practice.is_some() => {
                    if ui.button("Practice again").clicked() {
                        self.restart();
                    }
                }
                _ => {
                    if ui.button("New Campaign").clicked() {
                        self.restart();
                    }
                }
            }
            if ui.button("Practice…").clicked() {
                self.panel = Panel::Practice;
            }
            if self.practice.is_some() && ui.button("Return to campaign").clicked() {
                self.practice = None;
                self.panel = Panel::Pause;
            }
            if ui.button("Restart…").clicked() {
                self.panel = Panel::Restart;
            }
        }
        ui.horizontal(|ui| {
            if ui.button("Settings").clicked() {
                self.panel = Panel::Settings;
            }
            if ui.button("Help / About").clicked() {
                self.panel = Panel::Help;
            }
        });
        if !self.save.records.is_empty() {
            ui.separator();
            ui.label("Campaign records");
            for record in self.save.records.iter().take(3) {
                ui.label(format!(
                    "{} · level {}{}",
                    record.score,
                    record.level + 1,
                    if record.complete { " · complete" } else { "" }
                ));
            }
        }
        if ui.button("Back to Arcade").clicked() {
            self.suspend();
            self.leave = true;
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last).as_secs_f64();
        self.last = now;
        if self.themed.elapsed() > Duration::from_secs(2) {
            self.theme = omarchy_chess::theme::Theme::load();
            self.themed = now;
        }
        let mut visuals = if self.theme.background.r() > 150 {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };
        visuals.override_text_color = Some(self.theme.foreground);
        visuals.panel_fill = self.theme.background;
        ctx.set_visuals(visuals);
        arcade_presentation::apply(ctx);
        if !self.enabled {
            return;
        }
        let focus = ctx.input(|i| i.focused);
        let gone = ctx.input(|i| {
            i.events
                .iter()
                .any(|e| matches!(e, egui::Event::PointerGone))
        });
        if (!focus || gone) && self.panel == Panel::Play {
            self.suspend();
        }
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            if self.panel == Panel::Play {
                self.suspend();
            } else if self.panel == Panel::Pause && !self.blocked {
                self.resume();
            } else {
                self.panel = Panel::Pause;
            }
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, Key::Comma)) {
            self.open(Panel::Settings);
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, Key::M)) {
            self.save.sound = !self.save.sound;
            self.audio.stop();
            self.persist();
        }
        if ctx.input(|i| i.key_pressed(Key::F1)) {
            self.open(Panel::Help);
        }
        let was_play = self.panel == Panel::Play;
        egui::TopBottomPanel::top("shatter-controls").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong("SHATTER");
                if ui.button("Pause / Resume").clicked() {
                    if self.panel == Panel::Play {
                        self.suspend();
                    } else if self.panel == Panel::Pause && !self.blocked {
                        self.resume();
                    }
                }
                if ui
                    .add_enabled(!self.blocked, egui::Button::new("Restart"))
                    .clicked()
                {
                    self.open(Panel::Restart);
                }
                if ui.button("Settings").clicked() {
                    self.open(Panel::Settings);
                }
                if ui.button("Help").clicked() {
                    self.open(Panel::Help);
                }
                if ui.button("Back to Arcade").clicked() {
                    self.suspend();
                    self.leave = true;
                }
            });
            if let Some(e) = &self.error {
                ui.label(e);
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            arcade_presentation::backdrop(ui);
            let avail = ui.available_rect_before_wrap().shrink(20.);
            let scale = (avail.width() / 800.).min(avail.height() / 600.).max(0.05);
            let rect = Rect::from_center_size(avail.center(), Vec2::new(800., 600.) * scale);
            if self.panel == Panel::Play && was_play && !self.blocked && focus {
                let mut input = self.controls.input(ctx, rect, scale);
                self.pending_launch |= input.launch;
                match self.clock.advance(elapsed) {
                    Err(reason) => {
                        self.cue = Some((reason.into(), now));
                        self.suspend();
                    }
                    Ok(ticks) => {
                        for _ in 0..ticks {
                            input.launch = self.pending_launch;
                            self.pending_launch = false;
                            let events = self.run_mut().step(input);
                            if !self.save.reduced_effects {
                                let positions: Vec<_> =
                                    self.run().balls.iter().map(|b| b.p).collect();
                                self.trails.extend(positions);
                                if self.trails.len() > 18 {
                                    self.trails.drain(..self.trails.len() - 18);
                                }
                            }
                            for event in &events {
                                if let Event::Destroy(p) | Event::Damage(p) = event {
                                    self.fragments.push((*p, now));
                                }
                                if *event == Event::Redirect {
                                    self.cue = Some(("New angle".into(), now));
                                }
                            }
                            if self.save.sound {
                                if let Some(event) = events.last() {
                                    self.audio.play(match event {
                                        Event::Paddle => 0,
                                        Event::Damage(_) => 1,
                                        Event::Destroy(_) => 5,
                                        Event::Pickup => 3,
                                        Event::Lost => 4,
                                        Event::Clear => 2,
                                        Event::Redirect => 0,
                                    });
                                }
                            }
                            if matches!(
                                self.run().phase,
                                Phase::Clear | Phase::Over | Phase::Complete
                            ) {
                                if let Some(r) = &self.practice {
                                    self.save.practice_best[r.level] =
                                        self.save.practice_best[r.level].max(r.score);
                                }
                                let clear = self.run().phase != Phase::Over;
                                self.suspend();
                                self.trails.clear();
                                if self.save.sound {
                                    self.audio.play(if clear { 2 } else { 4 });
                                }
                                break;
                            }
                        }
                    }
                }
            }
            self.fragments
                .retain(|(_, t)| t.elapsed() < Duration::from_millis(330));
            self.render(ui, rect, scale);
        });
        self.overlay(ctx);
        if self.checkpoint.elapsed() > Duration::from_secs(15) {
            self.persist();
        }
        ctx.request_repaint_after(Duration::from_millis(8));
    }
    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        self.suspend();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn key(key: Key, pressed: bool) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }
    }
    fn input(ctx: &egui::Context, c: &mut Controls, events: Vec<egui::Event>) -> Input {
        let mut result = Input::default();
        let _ = ctx.run(
            egui::RawInput {
                events,
                focused: true,
                ..Default::default()
            },
            |ctx| {
                result = c.input(
                    ctx,
                    Rect::from_min_size(Pos2::new(100., 100.), Vec2::new(800., 600.)),
                    1.,
                );
            },
        );
        result
    }
    #[test]
    fn resume_requires_release_then_fresh_launch() {
        let ctx = egui::Context::default();
        let mut c = Controls::default();
        assert!(!input(&ctx, &mut c, vec![key(Key::Space, true)]).launch);
        assert!(!input(&ctx, &mut c, vec![]).fire);
        input(&ctx, &mut c, vec![key(Key::Space, false)]);
        assert!(input(&ctx, &mut c, vec![key(Key::Space, true)]).launch);
        c.clear();
        assert!(!input(&ctx, &mut c, vec![]).fire);
    }
    #[test]
    fn latest_source_stationary_pointer_opposites_and_letterbox() {
        let ctx = egui::Context::default();
        let mut c = Controls::default();
        input(&ctx, &mut c, vec![]);
        let mouse = input(
            &ctx,
            &mut c,
            vec![egui::Event::PointerMoved(Pos2::new(300., 300.))],
        );
        assert_eq!(mouse.target, Some(200.));
        let keyboard = input(&ctx, &mut c, vec![key(Key::ArrowLeft, true)]);
        assert_eq!(keyboard.target, None);
        assert_eq!(keyboard.direction, -1);
        let both = input(&ctx, &mut c, vec![key(Key::ArrowRight, true)]);
        assert_eq!(both.direction, 0);
        assert_eq!(both.target, None);
        let outside = input(
            &ctx,
            &mut c,
            vec![egui::Event::PointerMoved(Pos2::new(50., 50.))],
        );
        assert_eq!(outside.target, None);
        let mouse = input(
            &ctx,
            &mut c,
            vec![egui::Event::PointerMoved(Pos2::new(400., 300.))],
        );
        assert_eq!(mouse.target, Some(300.));
    }
}
