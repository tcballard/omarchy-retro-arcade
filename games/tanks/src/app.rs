use crate::{
    ai::{Difficulty, Search},
    audio::{Audio, Cue},
    effects::Resolution,
    rules::{Phase, Point, Rejected, Weapon, DT},
    storage::{self, Mode, Save},
};
use eframe::egui::{self, Align2, Key, Pos2, Rect, RichText, Sense, Vec2};
use std::{
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
#[path = "render.rs"]
mod render;

pub struct App {
    save: Save,
    path: PathBuf,
    blocked: bool,
    notice: Option<String>,
    paused: bool,
    settings: bool,
    restart: bool,
    enabled: bool,
    armed: bool,
    leave: bool,
    search: Option<Search>,
    last: Instant,
    accumulator: f64,
    movement_clock: f64,
    aim_clock: f64,
    visual_clock: f64,
    audio: Audio,
    audio_notice: Option<String>,
    theme: omarchy_chess::theme::Theme,
    themed: Instant,
}
impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
impl App {
    pub fn new() -> Self {
        Self::from_path(storage::path())
    }
    fn from_path(path: PathBuf) -> Self {
        let (save, notice) = match storage::load(&path) {
            Ok(s) => (s, None),
            Err(e) => (Save::default(), Some(e)),
        };
        Self {
            save,
            path,
            blocked: notice.is_some(),
            notice,
            paused: true,
            settings: false,
            restart: false,
            enabled: true,
            armed: false,
            leave: false,
            search: None,
            last: Instant::now(),
            accumulator: 0.0,
            movement_clock: 0.0,
            aim_clock: 0.0,
            visual_clock: 0.0,
            audio: Audio::default(),
            audio_notice: None,
            theme: omarchy_chess::theme::Theme::load(),
            themed: Instant::now(),
        }
    }
    fn persist(&mut self) {
        if !self.blocked {
            self.save.observe();
            if let Err(e) = storage::write(&self.path, &self.save) {
                self.notice = Some(format!("Could not save: {e}"));
            }
        }
    }
    fn clear_input(&mut self) {
        self.armed = false;
        self.accumulator = 0.0;
        self.movement_clock = 0.0;
        self.aim_clock = 0.0;
    }
    pub fn suspend(&mut self) {
        self.paused = true;
        self.clear_input();
        self.search = None;
        self.audio.stop();
        self.persist();
    }
    pub fn set_input_enabled(&mut self, enabled: bool) {
        if self.enabled && !enabled {
            self.suspend();
        }
        self.enabled = enabled;
    }
    pub fn finished(&mut self) -> bool {
        self.leave
    }
    fn ai_turn(&self) -> bool {
        self.save.mode == Mode::Solo && self.save.game.active() == 1
    }
    fn sound(&mut self, cue: Cue) {
        if self.save.sound {
            if let Err(e) = self.audio.play(cue) {
                self.audio_notice = Some(format!("Sound unavailable: {e}"));
            }
        }
    }
    fn mute(&mut self) {
        self.save.sound = !self.save.sound;
        if !self.save.sound {
            self.audio.stop();
        }
        self.persist();
    }
    fn resume(&mut self) {
        if self.save.started {
            self.paused = false;
            self.settings = false;
            self.restart = false;
            self.clear_input();
        }
    }
    fn begin_match(&mut self) {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        self.save.new_match(seed);
        self.search = None;
        self.restart = false;
        self.paused = false;
        self.settings = false;
        self.clear_input();
        self.audio.stop();
        self.notice = if self.blocked {
            self.notice.take()
        } else {
            None
        };
        self.persist();
    }
    fn choose(&mut self, mode: Mode, difficulty: Difficulty) {
        self.save.mode = mode;
        self.save.difficulty = difficulty;
        self.begin_match();
    }
    fn command(&mut self, result: Result<(), Rejected>) {
        if let Err(e) = result {
            self.notice = Some(
                match e {
                    Rejected::NoFuel => "No movement fuel left this round.",
                    Rejected::Steep => "That slope is too steep to cross.",
                    Rejected::Edge => "You have reached the battlefield edge.",
                    Rejected::Occupied => "The other tank is blocking that move.",
                    Rejected::NoAmmo => "That weapon is empty. Choose another.",
                    _ => "Wait until your turn is ready.",
                }
                .into(),
            );
        }
    }
    fn tick(&mut self) {
        if let Some(r) = &mut self.save.resolution {
            if r.advance() {
                self.save.resolution = None;
                self.clear_input();
                match self.save.game.phase() {
                    Phase::MatchOver { .. } => self.sound(Cue::Match),
                    Phase::RoundOver { winner: Some(_) } => self.sound(Cue::Round),
                    _ => {}
                }
                self.persist();
            }
            return;
        }
        let phase = self.save.game.phase();
        if let Some(impact) = self.save.game.tick_event() {
            let cue = match impact.weapon {
                Weapon::Shell => Cue::Shell,
                Weapon::Heavy => Cue::Heavy,
                Weapon::Digger => Cue::Digger,
            };
            self.save.resolution = Some(Resolution::new(impact));
            self.clear_input();
            self.sound(cue);
            self.persist();
        } else if phase == Phase::Flying && self.save.game.phase() != Phase::Flying {
            self.clear_input();
            self.notice = Some("Shot missed. Adjust using your previous trace.".into());
            self.persist();
        }
    }
    fn top(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("tanks-header").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("TANKS");
                ui.label(
                    RichText::new(format!(
                        "ROUND {:02}  /  FIRST TO TWO",
                        self.save.game.round()
                    ))
                    .small(),
                );
                if ui
                    .add_enabled(
                        self.enabled && self.save.started,
                        egui::Button::new(if self.paused { "Resume" } else { "Pause" }),
                    )
                    .clicked()
                {
                    if self.paused {
                        self.resume()
                    } else {
                        self.suspend()
                    }
                }
                if ui
                    .add_enabled(self.enabled, egui::Button::new("Settings"))
                    .clicked()
                {
                    self.suspend();
                    self.settings = true;
                }
                if ui
                    .add_enabled(
                        self.enabled,
                        egui::Button::new(if self.save.sound { "Sound on" } else { "Muted" }),
                    )
                    .clicked()
                {
                    self.mute();
                }
                if ui
                    .add_enabled(self.enabled, egui::Button::new("Back to Arcade"))
                    .clicked()
                {
                    self.suspend();
                    self.leave = true;
                }
            });
            ui.horizontal_wrapped(|ui| {
                let wins = self.save.game.wins();
                for (i, tank) in self.save.game.tanks().iter().enumerate() {
                    let health = self
                        .save
                        .resolution
                        .as_ref()
                        .filter(|r| r.progress() < 0.7 && !self.save.reduced_effects)
                        .map_or(tank.health, |r| r.impact.tanks_before[i].health);
                    ui.label(
                        RichText::new(format!(
                            "{}  ·  {} HP  ·  {} round{}",
                            if i == 0 {
                                "P1"
                            } else if self.save.mode == Mode::Solo {
                                "CPU"
                            } else {
                                "P2"
                            },
                            health,
                            wins[i],
                            if wins[i] == 1 { "" } else { "s" }
                        ))
                        .color(if i == 0 {
                            self.theme.accent
                        } else {
                            self.theme.foreground
                        }),
                    );
                    if i == 0 {
                        ui.separator();
                    }
                }
            });
            if let Some(n) = &self.notice {
                ui.label(n);
            }
        });
    }
    fn controls(&mut self, ctx: &egui::Context, human: bool, interactive: bool) -> (bool, i8) {
        let mut fire = false;
        let mut movement = 0;
        egui::TopBottomPanel::bottom("tanks-controls").show(ctx, |ui| {
            let active = self
                .save
                .resolution
                .as_ref()
                .map_or(self.save.game.active(), |r| r.impact.shooter);
            let tank = self.save.game.tanks()[active].clone();
            if let Some(resolution) = &self.save.resolution {
                ui.set_min_height(92.0);
                ui.label(RichText::new("IMPACT  /  GROUND SETTLING").strong());
                ui.label(format!(
                    "Player {} · {:?} · {}° · Power {}",
                    active + 1,
                    resolution.impact.weapon,
                    tank.angle,
                    tank.power
                ));
                ui.label("Next turn begins when the ground settles.");
                return;
            } else {
                match self.save.game.phase() {
                    Phase::Ready => {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                RichText::new(format!(
                                    "{} — your turn",
                                    if self.ai_turn() {
                                        "Computer"
                                    } else if active == 0 {
                                        "Player 1"
                                    } else {
                                        "Player 2"
                                    }
                                ))
                                .strong(),
                            );
                            if !self.ai_turn()
                                && (ui
                                    .add_enabled(interactive, egui::Button::new("Ready · Enter"))
                                    .clicked()
                                    || (interactive && ctx.input(|i| i.key_pressed(Key::Enter))))
                            {
                                let result = self.save.game.ready();
                                self.command(result);
                                self.clear_input();
                                self.notice = None;
                            }
                        });
                    }
                    Phase::Flying => {
                        ui.label("SHOT IN FLIGHT  /  WATCH YOUR ARC");
                    }
                    Phase::Aiming => {
                        ui.label(if self.ai_turn() {
                            "Computer is lining up a shot…"
                        } else {
                            "SET YOUR ANGLE. READ THE WIND."
                        });
                    }
                    Phase::RoundOver { winner } => {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                RichText::new(
                                    winner.map_or("DRAW — both tanks eliminated".into(), |i| {
                                        format!("PLAYER {} TAKES THE ROUND", i + 1)
                                    }),
                                )
                                .strong(),
                            );
                            if ui
                                .add_enabled(interactive, egui::Button::new("Next round · Enter"))
                                .clicked()
                                || (interactive && ctx.input(|i| i.key_pressed(Key::Enter)))
                            {
                                let result = self.save.game.next_round();
                                self.command(result);
                                self.clear_input();
                                self.persist();
                            }
                        });
                    }
                    Phase::MatchOver { winner } => {
                        ui.horizontal_wrapped(|ui| {
                            ui.heading(format!("PLAYER {} WINS", winner + 1));
                            if ui
                                .add_enabled(interactive, egui::Button::new("Play again · Enter"))
                                .clicked()
                                || (interactive && ctx.input(|i| i.key_pressed(Key::Enter)))
                            {
                                self.begin_match();
                            }
                        });
                    }
                }
            }
            ui.add_enabled_ui(human && !self.paused, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let mut angle = tank.angle;
                    let mut power = tank.power;
                    ui.label("ANGLE");
                    ui.add(
                        egui::DragValue::new(&mut angle)
                            .range(5..=175)
                            .suffix("°")
                            .speed(0.3),
                    );
                    ui.label("POWER");
                    let old_width = ui.spacing().slider_width;
                    ui.spacing_mut().slider_width =
                        (ui.available_width() * 0.25).clamp(85.0, 180.0);
                    let slider = ui.add(egui::Slider::new(&mut power, 1..=100));
                    ui.spacing_mut().slider_width = old_width;
                    if slider.hovered() {
                        let scroll = ctx.input_mut(|i| {
                            let d = i.smooth_scroll_delta.y;
                            i.smooth_scroll_delta = Vec2::ZERO;
                            d
                        });
                        if scroll != 0.0 {
                            power = (i32::from(power) + if scroll > 0.0 { 1 } else { -1 })
                                .clamp(1, 100) as u16;
                        }
                    }
                    if angle != tank.angle || power != tank.power {
                        let result = self.save.game.aim(angle, power);
                        self.command(result);
                    }
                    fire = ui
                        .add(
                            egui::Button::new(
                                RichText::new("FIRE · Space")
                                    .strong()
                                    .color(self.theme.accent_text()),
                            )
                            .fill(self.theme.accent)
                            .min_size(Vec2::new(130.0, 36.0)),
                        )
                        .clicked();
                });
                ui.horizontal_wrapped(|ui| {
                    let left = ui.button("A · Move left").on_hover_text("Hold A");
                    let right = ui.button("D · Move right").on_hover_text("Hold D");
                    movement = i8::from(right.is_pointer_button_down_on())
                        - i8::from(left.is_pointer_button_down_on());
                    ui.label(format!("FUEL {:.0}", tank.fuel));
                    ui.separator();
                    for (w, label) in [
                        (Weapon::Shell, "1  Shell ∞".to_string()),
                        (Weapon::Heavy, format!("2  Heavy ×{}", tank.heavy)),
                        (Weapon::Digger, format!("3  Digger ×{}", tank.diggers)),
                    ] {
                        if ui
                            .add_enabled(
                                tank.available(w),
                                egui::Button::new(label).selected(tank.weapon == w),
                            )
                            .clicked()
                        {
                            let result = self.save.game.select(w);
                            self.command(result);
                        }
                    }
                });
            });
        });
        (fire, movement)
    }
    fn menus(&mut self, ctx: &egui::Context) {
        if !self.paused {
            return;
        }
        let fresh = !self.save.started;
        let title = if self.settings {
            "Tanks settings"
        } else if fresh {
            "Choose your battle"
        } else {
            "Paused"
        };
        egui::Window::new(title)
            .enabled(self.enabled)
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_min_width(280.0);
                if self.settings {
                    let mut sound = self.save.sound;
                    if ui.checkbox(&mut sound, "Sound · Ctrl+M").changed() {
                        self.mute();
                    }
                    if ui
                        .checkbox(&mut self.save.reduced_effects, "Reduced effects")
                        .changed()
                    {
                        self.persist();
                    }
                    if let Some(n) = &self.audio_notice {
                        ui.label(n);
                    }
                    ui.label("↑↓ angle · ←→ power · A/D move · 1/2/3 weapon");
                }
                if fresh || self.restart {
                    ui.label(if fresh {
                        "Pick a mode. First to two rounds wins."
                    } else {
                        "Starting again replaces your unfinished match."
                    });
                    for (mode, difficulty, label) in [
                        (Mode::Solo, Difficulty::Easy, "1 · Solo — Easy"),
                        (Mode::Solo, Difficulty::Normal, "2 · Solo — Normal"),
                        (Mode::Local, Difficulty::Normal, "3 · Two local players"),
                    ] {
                        if ui
                            .add_sized([280.0, 38.0], egui::Button::new(label))
                            .clicked()
                        {
                            self.choose(mode, difficulty);
                        }
                    }
                    if !fresh && ui.button("Cancel").clicked() {
                        self.restart = false;
                    }
                } else {
                    ui.label("Your match is saved. Continue when ready.");
                    if ui
                        .add_sized([280.0, 38.0], egui::Button::new("Resume · Escape"))
                        .clicked()
                    {
                        self.resume();
                    }
                    if ui.button("New match…").clicked() {
                        self.restart = true;
                        self.armed = false;
                    }
                    ui.label(format!(
                        "Solo {} wins / {} losses · Local {} matches",
                        self.save.solo_wins, self.save.solo_losses, self.save.local_matches
                    ));
                }
                if self.blocked {
                    ui.label("Original save retained. Saving is disabled.");
                    if ui.button("Archive original and reset").clicked() {
                        match storage::archive(&self.path) {
                            Ok(path) => {
                                self.blocked = false;
                                self.save = Save::default();
                                self.notice =
                                    Some(format!("Original archived: {}", path.display()));
                                self.search = None;
                                self.audio.stop();
                                self.clear_input();
                                self.persist();
                            }
                            Err(e) => self.notice = Some(format!("Archive failed: {e}")),
                        }
                    }
                }
                if ui.button("Back to Arcade").clicked() {
                    self.suspend();
                    self.leave = true;
                }
            });
    }
    pub fn frame(&mut self, ctx: &egui::Context) {
        let elapsed = self.last.elapsed().as_secs_f64();
        self.last = Instant::now();
        if let Err(e) = self.audio.poll() {
            self.audio_notice = Some(e);
        }
        if self.themed.elapsed() > Duration::from_secs(2) {
            self.theme = omarchy_chess::theme::Theme::load();
            self.themed = Instant::now();
        }
        let mut v = if self.theme.light() {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };
        v.panel_fill = self.theme.background;
        v.window_fill = self.theme.background;
        v.override_text_color = Some(self.theme.foreground);
        v.selection.bg_fill = self.theme.accent;
        ctx.set_visuals(v);
        arcade_presentation::apply(ctx);
        let focused = ctx.input(|i| i.focused);
        if (!focused || elapsed > 0.25) && !self.paused {
            self.suspend();
        }
        let held = ctx.input(|i| {
            i.pointer.any_down()
                || [
                    Key::Space,
                    Key::Enter,
                    Key::A,
                    Key::D,
                    Key::ArrowUp,
                    Key::ArrowDown,
                    Key::ArrowLeft,
                    Key::ArrowRight,
                    Key::Num1,
                    Key::Num2,
                    Key::Num3,
                ]
                .iter()
                .any(|k| i.key_down(*k))
        });
        if !self.armed && !held && focused && self.enabled {
            self.armed = true;
        }
        if self.enabled && focused {
            if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::M)) {
                self.mute();
            }
            if ctx.input(|i| i.key_pressed(Key::Escape)) {
                if self.paused {
                    if self.restart {
                        self.restart = false;
                    } else {
                        self.resume();
                    }
                } else {
                    self.suspend();
                }
            }
            if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::Comma)) {
                self.suspend();
                self.settings = true;
            }
            if self.paused && self.armed && (!self.save.started || self.restart) && !self.settings {
                if ctx.input(|i| i.key_pressed(Key::Num1)) {
                    self.choose(Mode::Solo, Difficulty::Easy);
                } else if ctx.input(|i| i.key_pressed(Key::Num2)) {
                    self.choose(Mode::Solo, Difficulty::Normal);
                } else if ctx.input(|i| i.key_pressed(Key::Num3)) {
                    self.choose(Mode::Local, Difficulty::Normal);
                }
            }
        }
        let interactive =
            self.enabled && focused && !self.paused && self.armed && self.save.resolution.is_none();
        let human = interactive && !self.ai_turn() && self.save.game.phase() == Phase::Aiming;
        self.top(ctx);
        let (mut fire, mut movement) = self.controls(ctx, human, interactive);
        egui::CentralPanel::default().show(ctx, |ui| {
            let (rect, scale, response) = render::field(self, ui);
            if human && !self.paused && response.dragged() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let t = &self.save.game.tanks()[self.save.game.active()];
                    let p = Point {
                        x: f64::from((pos.x - rect.left()) / scale),
                        y: f64::from((pos.y - rect.top()) / scale),
                    };
                    let c = t.centre();
                    let a = (c.y - p.y).atan2(p.x - c.x).to_degrees().clamp(5.0, 175.0) as u16;
                    let result = self.save.game.aim(a, t.power);
                    self.command(result);
                }
            }
        });
        self.menus(ctx);
        if human && self.armed && !self.paused && !ctx.wants_keyboard_input() {
            let (da, dp, m, f, weapon, pressed) = ctx.input(|i| {
                (
                    i32::from(i.key_down(Key::ArrowUp)) - i32::from(i.key_down(Key::ArrowDown)),
                    i32::from(i.key_down(Key::ArrowRight)) - i32::from(i.key_down(Key::ArrowLeft)),
                    i8::from(i.key_down(Key::D)) - i8::from(i.key_down(Key::A)),
                    i.key_pressed(Key::Space),
                    if i.key_pressed(Key::Num1) {
                        Some(Weapon::Shell)
                    } else if i.key_pressed(Key::Num2) {
                        Some(Weapon::Heavy)
                    } else if i.key_pressed(Key::Num3) {
                        Some(Weapon::Digger)
                    } else {
                        None
                    },
                    i.events.iter().any(|e| {
                        matches!(
                            e,
                            egui::Event::Key {
                                key: Key::ArrowUp
                                    | Key::ArrowDown
                                    | Key::ArrowLeft
                                    | Key::ArrowRight,
                                pressed: true,
                                repeat: false,
                                ..
                            }
                        )
                    }),
                )
            });
            if da != 0 || dp != 0 {
                self.aim_clock += elapsed.min(0.25) * 40.0;
                let steps = if pressed {
                    self.aim_clock = 0.0;
                    1
                } else {
                    let n = self.aim_clock.floor() as i32;
                    self.aim_clock -= f64::from(n);
                    n
                };
                if steps > 0 {
                    let t = &self.save.game.tanks()[self.save.game.active()];
                    let result = self.save.game.aim(
                        (i32::from(t.angle) + da * steps).clamp(5, 175) as u16,
                        (i32::from(t.power) + dp * steps).clamp(1, 100) as u16,
                    );
                    self.command(result);
                }
            } else {
                self.aim_clock = 0.0;
            }
            if m != 0 {
                movement = m;
            }
            fire |= f;
            if let Some(w) = weapon {
                let result = self.save.game.select(w);
                self.command(result);
            }
        }
        if interactive && !self.paused && self.armed {
            if human {
                if movement != 0 {
                    self.movement_clock += elapsed.min(0.25) * 30.0;
                    while self.movement_clock >= 1.0 {
                        self.movement_clock -= 1.0;
                        let result = self.save.game.move_one(movement > 0);
                        if result.is_err() {
                            self.command(result);
                            self.movement_clock = 0.0;
                            break;
                        }
                    }
                } else {
                    self.movement_clock = 0.0;
                }
                if fire {
                    match self.save.game.fire() {
                        Ok(()) => {
                            self.clear_input();
                            self.notice = None;
                            self.sound(Cue::Launch);
                        }
                        Err(e) => self.command(Err(e)),
                    }
                }
            }
            if self.ai_turn() {
                if self.save.game.phase() == Phase::Ready {
                    let _ = self.save.game.ready();
                }
                if self.save.game.phase() == Phase::Aiming {
                    if self.search.is_none() {
                        self.search =
                            Search::new(&self.save.game, self.save.difficulty, self.save.ai_seed);
                    }
                    if let Some(search) = &mut self.search {
                        if search.advance(1024) {
                            let fired = search.apply(&mut self.save.game).is_ok();
                            if fired {
                                self.save.ai_seed = search.next_seed();
                            }
                            self.search = None;
                            if fired {
                                self.sound(Cue::Launch);
                            }
                        }
                    }
                }
            }
        }
        if !self.paused && self.enabled && focused && self.save.started {
            self.visual_clock += elapsed.min(0.25);
            self.accumulator += elapsed.min(0.25);
            while self.accumulator >= DT {
                self.accumulator -= DT;
                self.tick();
            }
        } else {
            self.accumulator = 0.0;
        }
        ctx.request_repaint_after(Duration::from_millis(8));
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.frame(ctx);
    }
    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        self.suspend();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn frame(app: &mut App, ctx: &egui::Context, events: Vec<egui::Event>) {
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000.0, 800.0))),
            focused: true,
            events,
            ..Default::default()
        };
        let _ = ctx.run(input, |ctx| app.frame(ctx));
    }
    fn key(app: &mut App, ctx: &egui::Context, key: Key) {
        frame(
            app,
            ctx,
            vec![egui::Event::Key {
                key,
                physical_key: Some(key),
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        frame(
            app,
            ctx,
            vec![egui::Event::Key {
                key,
                physical_key: Some(key),
                pressed: false,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
    }
    #[test]
    fn keyboard_handover_pause_host_blocking_and_exact_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tanks.json");
        let mut app = App::from_path(path.clone());
        app.save.mode = Mode::Local;
        app.save.started = true;
        app.save.sound = false;
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]);
        key(&mut app, &ctx, Key::Space);
        assert_eq!(app.save.game.phase(), Phase::Ready);
        key(&mut app, &ctx, Key::Escape);
        key(&mut app, &ctx, Key::Enter);
        assert_eq!(app.save.game.phase(), Phase::Aiming);
        key(&mut app, &ctx, Key::Space);
        assert_eq!(app.save.game.phase(), Phase::Flying);
        key(&mut app, &ctx, Key::Escape);
        let snapshot = app.save.clone();
        key(&mut app, &ctx, Key::Space);
        assert_eq!(app.save, snapshot);
        app.set_input_enabled(false);
        key(&mut app, &ctx, Key::Escape);
        assert!(app.paused);
        assert_eq!(app.save, snapshot);
        app.suspend();
        let reopened = App::from_path(path);
        assert!(reopened.paused);
        assert_eq!(reopened.save, snapshot);
    }
    #[test]
    fn mouse_fire_commits_once_and_paused_clicks_cannot_fire() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = App::from_path(dir.path().join("tanks.json"));
        app.save.mode = Mode::Local;
        app.save.started = true;
        app.save.sound = false;
        app.save.game.ready().unwrap();
        app.paused = false;
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000.0, 800.0))),
            focused: true,
            ..Default::default()
        };
        let output = ctx.run(input, |ctx| app.frame(ctx));
        let pos = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.text().contains("FIRE · Space") => {
                    Some(text.pos + Vec2::splat(4.0))
                }
                _ => None,
            })
            .expect("visible fire control");
        for pressed in [true, false] {
            frame(
                &mut app,
                &ctx,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
        assert_eq!(app.save.game.phase(), Phase::Flying);
        app.suspend();
        let before = app.save.clone();
        for pressed in [true, false] {
            frame(
                &mut app,
                &ctx,
                vec![egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
        }
        assert_eq!(app.save, before);
    }
    #[test]
    fn rejected_save_stays_untouched_through_play_and_exit() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tanks.json");
        std::fs::write(&path, b"future-save").unwrap();
        let mut app = App::from_path(path.clone());
        assert!(app.blocked);
        app.begin_match();
        app.suspend();
        assert_eq!(std::fs::read(path).unwrap(), b"future-save");
    }
    #[test]
    fn first_run_choice_does_not_fire_or_select_a_weapon() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = App::from_path(dir.path().join("tanks.json"));
        app.save.sound = false;
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]);
        key(&mut app, &ctx, Key::Space);
        assert!(!app.save.started);
        key(&mut app, &ctx, Key::Num3);
        assert!(app.save.started);
        assert_eq!(app.save.mode, Mode::Local);
        assert_eq!(app.save.game.phase(), Phase::Ready);
        assert_eq!(
            app.save.game.tanks()[app.save.game.active()].weapon,
            Weapon::Shell
        );
    }
    #[test]
    fn impact_pause_reopen_and_reduced_effects_preserve_exact_outcome() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tanks.json");
        let mut app = App::from_path(path.clone());
        app.save.started = true;
        app.save.mode = Mode::Local;
        app.save.sound = false;
        app.save.game.ready().unwrap();
        app.save.game.aim(90, 1).unwrap();
        app.save.game.fire().unwrap();
        for _ in 0..2400 {
            app.tick();
            if app.save.resolution.is_some() {
                break;
            }
        }
        assert!(app.save.resolution.is_some());
        for _ in 0..43 {
            app.tick();
        }
        app.suspend();
        let mut resumed = App::from_path(path);
        assert_eq!(resumed.save, app.save);
        assert!(!resumed.blocked);
        let ctx = egui::Context::default();
        let snapshot = resumed.save.clone();
        frame(&mut resumed, &ctx, vec![]);
        key(&mut resumed, &ctx, Key::Enter);
        key(&mut resumed, &ctx, Key::Space);
        assert_eq!(resumed.save, snapshot);
        resumed.save.reduced_effects = true;
        for _ in 0..108 {
            app.tick();
            resumed.tick();
        }
        assert!(app.save.resolution.is_none());
        assert!(resumed.save.resolution.is_none());
        assert_eq!(app.save.game, resumed.save.game);
        assert_eq!(app.save.game, snapshot.game);
    }
}
