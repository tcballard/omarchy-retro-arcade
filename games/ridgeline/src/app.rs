//! Native Ridgeline frontend: input, panels, rendering and lifecycle.
use crate::{
    art::{self, Palette, View},
    audio::{Audio, Cue},
    battle::{Battle, Clock, Command, Event, Phase, Reject, BASE_HEALTH},
    data::*,
    storage::{self, Award, Medal, Save},
    ui::{self, Tone},
};
use arcade_presentation::{BRASS, IVORY};
use eframe::egui::{self, Align2, Color32, Key, Pos2, Rect, Sense, Shape, Stroke, Vec2};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Panel {
    None,
    Maps,
    Settings,
    Help,
    Restart,
    Replace { map: usize, difficulty: Difficulty },
    Result(Option<Award>),
    Recovery,
}

/// Cosmetic feedback only; never saved and never read by the simulation.
#[derive(Clone, Copy)]
enum Fx {
    Ring { at: P, radius: i32, colour: Color32 },
    Burst { at: P, seed: u32, colour: Color32 },
    Flash { from: P, to: P, colour: Color32 },
    Gate { at: P },
}
struct Effect {
    fx: Fx,
    born: Instant,
    /// Lifetime in seconds.
    life: f32,
}

pub struct App {
    save: Save,
    path: PathBuf,
    /// A rejected save disables all writes until explicit archive-and-reset.
    blocked: bool,
    error: Option<String>,
    write_error: Option<String>,
    panel: Panel,
    paused: bool,
    pause_reason: Option<String>,
    /// Tower chosen in the shop and awaiting placement.
    pending: Option<TowerKind>,
    /// Shared pointer/keyboard board cursor.
    cursor: (i32, i32),
    selected: Option<u32>,
    notice: Option<(String, Instant)>,
    clock: Clock,
    last: Instant,
    checkpoint: Instant,
    theme: arcade_platform::theme::Theme,
    themed: Instant,
    audio: Audio,
    effects: Vec<Effect>,
    leave: bool,
    enabled: bool,
    last_pointer: Option<Pos2>,
    /// Last drawn board transform, for pointer mapping in tests.
    view: Option<View>,
    /// Large centred wave announcement: heading, detail and start time.
    banner: Option<(String, String, Instant)>,
    /// Most recent breach, for the red edge pulse.
    breach: Option<Instant>,
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
        let (mut save, error) = match storage::load(&path) {
            Ok(s) => (s, None),
            Err(e) => (Save::default(), Some(e)),
        };
        // A result saved before its award (e.g. after a crash) is awarded now, once.
        let award = if error.is_none() {
            save.observe()
        } else {
            None
        };
        let blocked = error.is_some();
        let resumed = save.battle.as_ref().is_some_and(|b| !b.finished());
        let panel = if blocked {
            Panel::Recovery
        } else if save.battle.is_none() {
            Panel::Maps
        } else if save.battle.as_ref().is_some_and(Battle::finished) {
            Panel::Result(award)
        } else {
            Panel::None
        };
        let mut app = Self {
            save,
            path,
            blocked,
            error,
            write_error: None,
            panel,
            paused: resumed,
            pause_reason: resumed
                .then(|| "Your defence is saved exactly as you left it. Resume when ready.".into()),
            pending: None,
            cursor: (COLS / 2, ROWS / 2),
            selected: None,
            notice: None,
            clock: Clock::default(),
            last: Instant::now(),
            checkpoint: Instant::now(),
            theme: arcade_platform::theme::Theme::load(),
            themed: Instant::now(),
            audio: Audio::default(),
            effects: vec![],
            leave: false,
            enabled: true,
            last_pointer: None,
            view: None,
            banner: None,
            breach: None,
        };
        if award.is_some() {
            app.persist();
        }
        app
    }

    fn persist(&mut self) {
        self.checkpoint = Instant::now();
        if self.blocked {
            return;
        }
        self.write_error = storage::write(&self.path, &self.save)
            .err()
            .map(|e| format!("Progress is not saved: {e}"));
    }
    /// Host lifecycle: pause, stop owned audio, clear transient input and save.
    pub fn suspend(&mut self) {
        if self.running() {
            self.paused = true;
        }
        self.pending = None;
        self.clock.reset();
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
    fn running(&self) -> bool {
        self.save
            .battle
            .as_ref()
            .is_some_and(|b| b.phase == Phase::Running)
    }
    fn pause(&mut self, reason: Option<&str>) {
        if self.running() && !self.paused {
            self.paused = true;
            self.pause_reason = reason.map(str::to_owned);
            self.clock.reset();
            self.audio.stop();
            self.persist();
        }
    }
    fn resume(&mut self) {
        if self.blocked {
            return;
        }
        self.paused = false;
        self.pause_reason = None;
        self.clock.reset();
        self.last = Instant::now();
    }
    fn toggle_pause(&mut self) {
        if self.paused {
            self.resume();
        } else {
            self.pause(None);
        }
    }
    fn open(&mut self, panel: Panel) {
        self.pause(None);
        self.pending = None;
        self.panel = panel;
        self.persist();
    }
    fn close_panel(&mut self) {
        self.panel = if self.blocked {
            Panel::Recovery
        } else if self.save.battle.is_none() {
            Panel::Maps
        } else if self.save.battle.as_ref().is_some_and(Battle::finished) {
            Panel::Result(None)
        } else {
            Panel::None
        };
    }
    fn say(&mut self, text: impl Into<String>) {
        self.notice = Some((text.into(), Instant::now()));
    }
    fn cue(&mut self, cue: Cue) {
        if self.save.preferences.sound {
            self.audio.play(cue);
        }
    }
    fn toggle_sound(&mut self) {
        self.save.preferences.sound = !self.save.preferences.sound;
        self.audio.stop();
        self.persist();
    }
    fn toggle_speed(&mut self) {
        self.save.preferences.speed = if self.save.preferences.speed == 1 {
            2
        } else {
            1
        };
        self.clock.reset();
        self.persist();
    }

    /// Choose a map, confirming before an unfinished defence would be replaced.
    fn choose(&mut self, map: usize, difficulty: Difficulty, confirmed: bool) {
        if self.blocked || !self.save.progress.unlocked(map, difficulty) {
            return;
        }
        if !confirmed && self.save.battle.as_ref().is_some_and(Battle::unfinished) {
            self.panel = Panel::Replace { map, difficulty };
            return;
        }
        self.save.battle = Some(Battle::new(map, difficulty));
        self.pending = None;
        self.selected = None;
        self.effects.clear();
        self.paused = false;
        self.pause_reason = None;
        self.panel = Panel::None;
        self.persist();
    }

    /// Every player action goes through the engine's ordinary command path.
    fn command(&mut self, command: Command) -> Result<(), Reject> {
        let Some(b) = self.save.battle.as_mut() else {
            return Err(Reject::Finished);
        };
        let result = b.apply(command);
        match (command, result) {
            (_, Err(e)) => {
                self.say(e.to_string());
            }
            (Command::Build { kind, .. }, Ok(())) => {
                self.selected = self
                    .save
                    .battle
                    .as_ref()
                    .and_then(|b| b.towers.last())
                    .map(|t| t.id);
                self.say(format!("{} built", kind.name()));
                self.cue(Cue::Build);
                self.persist();
            }
            (Command::Upgrade { .. }, Ok(())) => {
                self.cue(Cue::Upgrade);
                self.persist();
            }
            (Command::Sell { .. }, Ok(())) => {
                self.selected = None;
                self.cue(Cue::Sell);
                self.persist();
            }
            (Command::StartWave, Ok(())) => {
                if let Some(b) = &self.save.battle {
                    self.banner = Some((
                        format!("WAVE {:02}", b.wave + 1),
                        composition(b.preview()),
                        Instant::now(),
                    ));
                }
                self.resume();
                self.cue(Cue::Wave);
                self.persist();
            }
        }
        result
    }
    fn place(&mut self, x: i32, y: i32) {
        if let Some(kind) = self.pending {
            if self.command(Command::Build { x, y, kind }).is_ok() {
                self.pending = None;
            }
        }
    }
    fn select_at(&mut self, x: i32, y: i32) {
        self.selected = self
            .save
            .battle
            .as_ref()
            .and_then(|b| b.tower_at(x, y))
            .map(|t| t.id);
    }
    fn start_wave(&mut self) {
        if self
            .save
            .battle
            .as_ref()
            .is_some_and(|b| b.phase == Phase::Waiting)
        {
            let _ = self.command(Command::StartWave);
        }
    }

    fn simulate(&mut self, elapsed: f64, now: Instant) {
        let speed = self.save.preferences.speed;
        let ticks = match self.clock.advance(elapsed, speed) {
            Ok(t) => t,
            Err(reason) => {
                self.pause(Some(reason));
                return;
            }
        };
        let reduced = self.save.preferences.reduced_effects;
        for _ in 0..ticks {
            let Some(b) = self.save.battle.as_mut() else {
                return;
            };
            if b.phase != Phase::Running {
                break;
            }
            let events = b.step();
            let mut cues = vec![];
            for event in events {
                let fx = |fx, life| Effect {
                    fx,
                    born: now,
                    life,
                };
                match event {
                    Event::Fire { tower, kind } if !reduced => {
                        let b = self.save.battle.as_ref().unwrap();
                        if let (Some(t), Some(shot)) = (b.tower(tower), b.shots.last()) {
                            let colour = if kind == TowerKind::Cryo {
                                Color32::from_rgb(150, 215, 250)
                            } else {
                                Color32::from_rgb(255, 214, 140)
                            };
                            self.effects.push(fx(
                                Fx::Flash {
                                    from: t.pos(),
                                    to: shot.to,
                                    colour,
                                },
                                0.12,
                            ));
                        }
                    }
                    Event::Hit { at, splash, kind } if !reduced => {
                        let colour = match kind {
                            TowerKind::Cryo => Color32::from_rgb(150, 215, 250),
                            TowerKind::Mortar => Color32::from_rgb(255, 170, 90),
                            _ => BRASS,
                        };
                        self.effects.push(fx(
                            Fx::Ring {
                                at,
                                radius: splash.max(160),
                                colour,
                            },
                            if splash > 0 { 0.45 } else { 0.22 },
                        ));
                    }
                    Event::Kill { at, bounty, .. } if !reduced => {
                        self.effects.push(fx(
                            Fx::Burst {
                                at,
                                seed: (at.x as u32) ^ (at.y as u32).rotate_left(16) ^ bounty,
                                colour: arcade_presentation::IVORY,
                            },
                            0.5,
                        ));
                    }
                    Event::Leak { at, damage } => {
                        self.effects.push(fx(Fx::Gate { at }, 0.6));
                        self.breach = Some(now);
                        self.notice = Some((format!("Breach · base −{damage}"), now));
                        cues.push(Cue::Leak);
                    }
                    Event::WaveCleared { wave, award } => {
                        self.banner = Some((
                            format!("WAVE {:02} HELD", wave + 1),
                            format!("+{award} credits · build, then send the next wave"),
                            now,
                        ));
                        cues.push(Cue::Clear);
                    }
                    Event::Victory => cues.push(Cue::Victory),
                    Event::Defeat => cues.push(Cue::Defeat),
                    _ => {}
                }
            }
            if self.effects.len() > 96 {
                self.effects.drain(..self.effects.len() - 96);
            }
            if let Some(cue) = cues.last() {
                self.cue(*cue);
            }
            let b = self.save.battle.as_ref().unwrap();
            if b.phase != Phase::Running {
                // Wave boundary or result: save immediately.
                if b.finished() {
                    let award = self.save.observe();
                    self.pending = None;
                    self.panel = Panel::Result(award);
                }
                self.persist();
                break;
            }
        }
    }

    fn keyboard(&mut self, ctx: &egui::Context) {
        // Focused widgets (Tab navigation) keep Enter/Space/arrows for themselves.
        let widget = ctx.memory(|m| m.focused().is_some());
        let modal = self.panel != Panel::None;
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape)) {
            if modal {
                if !matches!(self.panel, Panel::Maps | Panel::Result(_) | Panel::Recovery)
                    || self.save.battle.as_ref().is_some_and(|b| !b.finished())
                {
                    self.close_panel();
                }
            } else if self.pending.is_some() {
                // Cancelling placement takes priority over pausing.
                self.pending = None;
            } else if self.running() {
                self.toggle_pause();
            } else {
                self.selected = None;
            }
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, Key::Comma)) {
            self.open(Panel::Settings);
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, Key::M)) {
            self.toggle_sound();
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::F1)) {
            self.open(Panel::Help);
        }
        if modal || widget || self.blocked || self.save.battle.as_ref().is_none_or(Battle::finished)
        {
            return;
        }
        let pressed =
            |ctx: &egui::Context, key| ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key));
        for (key, kind) in [Key::Num1, Key::Num2, Key::Num3, Key::Num4]
            .into_iter()
            .zip(TowerKind::ALL)
        {
            if pressed(ctx, key) {
                self.pending = if self.pending == Some(kind) {
                    None
                } else {
                    Some(kind)
                };
            }
        }
        for (key, dx, dy) in [
            (Key::ArrowLeft, -1, 0),
            (Key::ArrowRight, 1, 0),
            (Key::ArrowUp, 0, -1),
            (Key::ArrowDown, 0, 1),
        ] {
            if pressed(ctx, key) {
                self.cursor = (
                    (self.cursor.0 + dx).clamp(0, COLS - 1),
                    (self.cursor.1 + dy).clamp(0, ROWS - 1),
                );
            }
        }
        if pressed(ctx, Key::Enter) {
            let (x, y) = self.cursor;
            if self.pending.is_some() {
                self.place(x, y);
            } else {
                self.select_at(x, y);
            }
        }
        if pressed(ctx, Key::U) {
            if let Some(id) = self.selected {
                let _ = self.command(Command::Upgrade { tower: id });
            }
        }
        if pressed(ctx, Key::Space) {
            self.start_wave();
        }
        if pressed(ctx, Key::F) {
            self.toggle_speed();
        }
        if pressed(ctx, Key::P) && self.running() {
            self.toggle_pause();
        }
    }

    // ----- Heads-up display -------------------------------------------------

    fn hud(&mut self, ui: &mut egui::Ui, pal: &Palette) {
        let compact = ui.available_width() < 1150.;
        let battle = self.save.battle.clone();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 14.;
            let title_w = if compact { 150. } else { 210. };
            let (r, _) = ui.allocate_exact_size(Vec2::new(title_w, 44.), Sense::hover());
            let p = ui.painter();
            match &battle {
                Some(b) => {
                    p.text(
                        r.left_top(),
                        Align2::LEFT_TOP,
                        format!("RIDGELINE · {:02}", b.map + 1),
                        ui::mono(10.5),
                        BRASS,
                    );
                    p.text(
                        r.left_bottom(),
                        Align2::LEFT_BOTTOM,
                        b.data().name,
                        ui::text(if compact { 16. } else { 19. }),
                        pal.text,
                    );
                    let w = p
                        .layout_no_wrap(
                            b.data().name.into(),
                            ui::text(if compact { 16. } else { 19. }),
                            pal.text,
                        )
                        .size()
                        .x;
                    if w + 60. < title_w {
                        ui::chip(
                            p,
                            Pos2::new(r.left() + w + 8., r.bottom() - 11.),
                            &b.difficulty.name().to_uppercase(),
                            if b.difficulty == Difficulty::Hard {
                                pal.bad
                            } else {
                                pal.muted
                            },
                            pal,
                        );
                    }
                }
                None => {
                    p.text(
                        r.left_top(),
                        Align2::LEFT_TOP,
                        "OMARCHY ARCADE",
                        ui::mono(10.5),
                        BRASS,
                    );
                    p.text(
                        r.left_bottom(),
                        Align2::LEFT_BOTTOM,
                        "Ridgeline",
                        ui::text(19.),
                        pal.text,
                    );
                }
            }
            let Some(b) = battle else {
                return;
            };
            divider(ui, pal);
            // Wave: count plus a 20-step progress track.
            let (r, _) = ui.allocate_exact_size(
                Vec2::new(if compact { 76. } else { 92. }, 44.),
                Sense::hover(),
            );
            let p = ui.painter();
            p.text(
                r.left_top(),
                Align2::LEFT_TOP,
                "WAVE",
                ui::mono(10.),
                pal.muted,
            );
            p.text(
                r.left_top() + Vec2::new(0., 13.),
                Align2::LEFT_TOP,
                format!("{:02}", b.wave + 1),
                ui::mono(20.),
                pal.text,
            );
            p.text(
                r.left_top() + Vec2::new(29., 20.),
                Align2::LEFT_TOP,
                format!("/{WAVES}"),
                ui::mono(11.),
                pal.muted,
            );
            let done = if b.phase == Phase::Victory {
                WAVES
            } else {
                b.wave
            };
            let step = r.width() / WAVES as f32;
            for i in 0..WAVES {
                let x = r.left() + i as f32 * step;
                let colour = if i < done {
                    BRASS
                } else if i == b.wave && b.phase == Phase::Running {
                    pal.accent
                } else {
                    pal.line
                };
                p.rect_filled(
                    Rect::from_min_size(Pos2::new(x, r.bottom() - 4.), Vec2::new(step - 1.5, 4.)),
                    1.,
                    colour,
                );
            }
            // Base health: shield, number and segmented meter.
            let (r, _) = ui.allocate_exact_size(
                Vec2::new(if compact { 120. } else { 150. }, 44.),
                Sense::hover(),
            );
            let p = ui.painter();
            p.text(
                r.left_top(),
                Align2::LEFT_TOP,
                "BASE",
                ui::mono(10.),
                pal.muted,
            );
            let flash = self
                .breach
                .is_some_and(|t| t.elapsed() < Duration::from_millis(600));
            ui::shield(
                p,
                r.left_top() + Vec2::new(7., 25.),
                7.,
                if flash { pal.bad } else { BRASS },
                pal,
            );
            p.text(
                r.left_top() + Vec2::new(19., 14.),
                Align2::LEFT_TOP,
                format!("{}", b.health),
                ui::mono(20.),
                if flash { pal.bad } else { pal.text },
            );
            ui::pips(
                p,
                Rect::from_min_size(
                    Pos2::new(r.left(), r.bottom() - 4.),
                    Vec2::new(r.width(), 4.),
                ),
                b.health,
                BASE_HEALTH,
                pal,
            );
            // Credits.
            let (r, _) = ui.allocate_exact_size(
                Vec2::new(if compact { 78. } else { 96. }, 44.),
                Sense::hover(),
            );
            let p = ui.painter();
            p.text(
                r.left_top(),
                Align2::LEFT_TOP,
                "CREDITS",
                ui::mono(10.),
                pal.muted,
            );
            ui::coin(p, r.left_top() + Vec2::new(7., 26.), 6.5, pal);
            p.text(
                r.left_top() + Vec2::new(19., 14.),
                Align2::LEFT_TOP,
                format!("{}", b.credits),
                ui::mono(20.),
                pal.text,
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 8.;
                let playable = !self.blocked && self.panel == Panel::None;
                let label = match b.phase {
                    Phase::Waiting => format!("Start wave {}", b.wave + 1),
                    Phase::Running => format!("Wave {} underway", b.wave + 1),
                    Phase::Victory => "Victory".into(),
                    Phase::Defeat => "Defeated".into(),
                };
                let waiting = b.phase == Phase::Waiting;
                let size = Vec2::new(if compact { 158. } else { 190. }, 38.);
                if b.phase == Phase::Running {
                    wave_status(ui, &b, size, self.paused, pal);
                } else if ui::button(
                    ui,
                    &label,
                    waiting.then_some("Space"),
                    Tone::Primary,
                    size,
                    waiting && playable,
                    pal,
                )
                .clicked()
                {
                    self.start_wave();
                }
                let running = b.phase == Phase::Running;
                if ui::button(
                    ui,
                    if self.paused && running {
                        "Resume"
                    } else {
                        "Pause"
                    },
                    Some("Esc"),
                    Tone::Secondary,
                    Vec2::new(if compact { 92. } else { 104. }, 38.),
                    running && playable,
                    pal,
                )
                .on_hover_text("Building stays available while paused.")
                .clicked()
                {
                    self.toggle_pause();
                }
                if let Some(i) = ui::segmented(
                    ui,
                    &["1×", "2×"],
                    self.save.preferences.speed as usize - 1,
                    if compact { 70. } else { 84. },
                    pal,
                ) {
                    if i as u32 + 1 != self.save.preferences.speed {
                        self.toggle_speed();
                    }
                }
                if !compact {
                    ui.label(
                        egui::RichText::new("SPEED  F")
                            .font(ui::mono(9.5))
                            .color(pal.muted),
                    );
                }
            });
        });
    }

    // ----- Sidebar ------------------------------------------------------------

    fn sidebar(&mut self, ui: &mut egui::Ui, pal: &Palette) {
        let w = ui.available_width();
        ui.spacing_mut().item_spacing = Vec2::new(6., 6.);
        ui.horizontal(|ui| {
            let bw = (w - 18.) / 4.;
            let has = self.save.battle.is_some();
            for (label, panel, enabled) in [
                ("Maps", Panel::Maps, !self.blocked),
                ("Restart", Panel::Restart, !self.blocked && has),
                ("Settings", Panel::Settings, true),
                ("Help", Panel::Help, true),
            ] {
                if ui::button(
                    ui,
                    label,
                    None,
                    Tone::Ghost,
                    Vec2::new(bw, 28.),
                    enabled,
                    pal,
                )
                .clicked()
                {
                    self.open(panel);
                }
            }
        });
        if ui::button(
            ui,
            "Back to Arcade",
            Some("Ctrl+H"),
            Tone::Secondary,
            Vec2::new(w, 30.),
            true,
            pal,
        )
        .clicked()
        {
            self.suspend();
            self.leave = true;
        }
        if let Some(e) = self.write_error.clone() {
            alert(ui, &e, pal);
            if ui::button(
                ui,
                "Retry save",
                None,
                Tone::Secondary,
                Vec2::new(w, 28.),
                true,
                pal,
            )
            .clicked()
            {
                self.persist();
            }
        }
        if let Some(e) = self.error.clone() {
            alert(ui, &e, pal);
        }
        let Some(b) = self.save.battle.clone() else {
            return;
        };
        let interactive = !self.blocked && !b.finished() && self.panel == Panel::None;
        ui::section(ui, "BUILD", Some("1–4"), pal);
        for (i, kind) in TowerKind::ALL.into_iter().enumerate() {
            if self.tower_card(ui, i, kind, &b, interactive, pal).clicked() && interactive {
                self.pending = if self.pending == Some(kind) {
                    None
                } else {
                    Some(kind)
                };
                self.selected = None;
            }
        }
        if let Some(kind) = self.pending {
            ui::section(ui, "PLACING", Some("Enter · Esc"), pal);
            self.placing(ui, &b, kind, pal);
        } else if let Some(t) = self.selected.and_then(|id| b.tower(id)).cloned() {
            ui::section(ui, "INSPECT", Some("U upgrade"), pal);
            self.inspector(ui, &b, &t, interactive, pal);
        } else {
            ui.add_space(4.);
            ui.label(
                egui::RichText::new("Choose a tower with 1–4 or its card, then place it on open terrace. Click a tower on the board to inspect, upgrade or sell it.")
                    .font(ui::text(12.))
                    .color(pal.muted),
            );
        }
    }

    fn tower_card(
        &self,
        ui: &mut egui::Ui,
        index: usize,
        kind: TowerKind,
        b: &Battle,
        interactive: bool,
        pal: &Palette,
    ) -> egui::Response {
        let stats = tower(kind, 0);
        let (r, response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 60.), Sense::click());
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::SelectableLabel,
                interactive,
                self.pending == Some(kind),
                kind.name(),
            )
        });
        let p = ui.painter();
        let chosen = self.pending == Some(kind);
        let short = stats.cost.saturating_sub(b.credits);
        ui::card(p, r, pal, response.hovered() && interactive, chosen);
        let well = Rect::from_min_size(r.min + Vec2::new(8., 7.), Vec2::splat(46.));
        p.rect_filled(well, 3., pal.surface.lerp_to_gamma(pal.ink, 0.25));
        p.rect_stroke(
            well,
            3.,
            Stroke::new(1_f32, art::kind_colour(kind, pal).gamma_multiply(0.7)),
            egui::StrokeKind::Inside,
        );
        art::tower_icon(p, well.center(), 40., kind, 0, pal, Vec2::new(0.7, -0.7));
        let dim = if short > 0 { 0.55 } else { 1. };
        p.text(
            Pos2::new(well.right() + 10., r.top() + 9.),
            Align2::LEFT_TOP,
            kind.name(),
            ui::text(15.),
            pal.text.gamma_multiply(dim),
        );
        let mut x = well.right() + 10.;
        let target = kind.targets();
        x += ui::chip(
            p,
            Pos2::new(x, r.bottom() - 15.),
            &target.name().to_uppercase(),
            if target == Targets::Air {
                pal.air
            } else {
                pal.muted
            },
            pal,
        ) + 4.;
        let extra = match kind {
            TowerKind::Cannon => "DIRECT",
            TowerKind::Mortar => "SPLASH",
            TowerKind::Flak => "RAPID",
            TowerKind::Cryo => "SLOW",
        };
        ui::chip(p, Pos2::new(x, r.bottom() - 15.), extra, BRASS, pal);
        ui::keycap(
            p,
            Pos2::new(r.right() - 16., r.top() + 16.),
            &(index + 1).to_string(),
            pal,
        );
        let cost = Pos2::new(r.right() - 10., r.bottom() - 15.);
        let g = p.layout_no_wrap(
            stats.cost.to_string(),
            ui::mono(15.),
            if short > 0 { pal.bad } else { pal.text },
        );
        p.galley(
            cost - Vec2::new(g.size().x, g.size().y / 2.),
            g.clone(),
            pal.text,
        );
        ui::coin(p, cost - Vec2::new(g.size().x + 9., 0.), 5., pal);
        if short > 0 {
            // Beside the hotkey, clear of the role chips at any width.
            p.text(
                Pos2::new(r.right() - 30., r.top() + 16.),
                Align2::RIGHT_CENTER,
                format!("need {short}"),
                ui::mono(9.5),
                pal.bad,
            );
        }
        if response.has_focus() {
            ui::focus_ring(p, r, pal);
        }
        response.on_hover_text(format!("{} · targets {}", kind.role(), target.name()))
    }

    fn placing(&mut self, ui: &mut egui::Ui, b: &Battle, kind: TowerKind, pal: &Palette) {
        let (x, y) = self.cursor;
        let check = b.check_build(x, y, kind);
        let stats = tower(kind, 0);
        panel_card(ui, pal, |ui| {
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(Vec2::splat(40.), Sense::hover());
                art::tower_icon(
                    ui.painter(),
                    r.center(),
                    36.,
                    kind,
                    0,
                    pal,
                    Vec2::new(0.7, -0.7),
                );
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(format!("Placing {}", kind.name()))
                            .font(ui::text(15.))
                            .color(pal.text),
                    );
                    let (text, colour) = match check {
                        Ok(cost) => (format!("Cell {x},{y} · valid · {cost} credits"), pal.good),
                        Err(e) => (format!("Cell {x},{y} · {e}"), pal.bad),
                    };
                    ui.label(egui::RichText::new(text).font(ui::text(11.5)).color(colour));
                });
            });
            tower_stats(ui, &stats, None, pal);
            if ui::button(
                ui,
                "Cancel placement",
                Some("Esc"),
                Tone::Secondary,
                Vec2::new(ui.available_width(), 30.),
                true,
                pal,
            )
            .clicked()
            {
                self.pending = None;
            }
        });
    }

    fn inspector(
        &mut self,
        ui: &mut egui::Ui,
        b: &Battle,
        t: &crate::battle::Tower,
        interactive: bool,
        pal: &Palette,
    ) {
        let now = t.stats();
        let max = usize::from(t.tier) + 1 >= TIERS;
        let next = (!max).then(|| tower(t.kind, t.tier + 1));
        panel_card(ui, pal, |ui| {
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(Vec2::splat(40.), Sense::hover());
                art::tower_icon(
                    ui.painter(),
                    r.center(),
                    36.,
                    t.kind,
                    t.tier,
                    pal,
                    Vec2::new(0.7, -0.7),
                );
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{} · tier {}", t.kind.name(), t.tier + 1))
                            .font(ui::text(15.))
                            .color(pal.text),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "{} · targets {}",
                            t.kind.role(),
                            t.kind.targets().name().to_lowercase()
                        ))
                        .font(ui::text(11.))
                        .color(pal.muted),
                    );
                });
            });
            tower_stats(ui, &now, next.as_ref(), pal);
            let ready = t.reload as f32 / 60.;
            ui.label(
                egui::RichText::new(if ready > 0. {
                    format!("Reloading · {ready:.2}s")
                } else {
                    "Ready to fire".into()
                })
                .font(ui::mono(10.5))
                .color(pal.muted),
            );
            let w = ui.available_width();
            let refund = refund(t.spent);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.;
                match (next, b.check_upgrade(t.id)) {
                    (Some(n), check) => {
                        if ui::button(
                            ui,
                            &format!("Upgrade · {}", n.cost),
                            Some("U"),
                            Tone::Primary,
                            Vec2::new(w * 0.6 - 3., 34.),
                            interactive && check.is_ok(),
                            pal,
                        )
                        .clicked()
                        {
                            let _ = self.command(Command::Upgrade { tower: t.id });
                        }
                    }
                    (None, _) => {
                        ui::button(
                            ui,
                            "Max tier",
                            None,
                            Tone::Primary,
                            Vec2::new(w * 0.6 - 3., 34.),
                            false,
                            pal,
                        );
                    }
                }
                if ui::button(
                    ui,
                    &format!("Sell · {refund}"),
                    None,
                    Tone::Secondary,
                    Vec2::new(w * 0.4 - 3., 34.),
                    interactive,
                    pal,
                )
                .on_hover_text(
                    "Refunds 70% of credits spent, rounded down. Shots in flight still land.",
                )
                .clicked()
                {
                    let _ = self.command(Command::Sell { tower: t.id });
                }
            });
            if let Err(Reject::Unaffordable { need }) = b.check_upgrade(t.id) {
                ui.label(
                    egui::RichText::new(format!("Need {need} more credits to upgrade"))
                        .font(ui::text(11.))
                        .color(pal.bad),
                );
            }
        });
    }

    // ----- Board --------------------------------------------------------------

    fn board(&mut self, ui: &mut egui::Ui, pal: &Palette) {
        arcade_presentation::backdrop(ui);
        // Quiet the cabinet in play so the board carries the eye.
        ui.painter().rect_filled(
            ui.max_rect(),
            0.,
            if pal.light {
                Color32::from_white_alpha(70)
            } else {
                Color32::from_black_alpha(120)
            },
        );
        let Some(b) = self.save.battle.clone() else {
            return;
        };
        // Board and incoming-wave strip form one block, centred in the space.
        let avail = ui.available_rect_before_wrap();
        const STRIP: f32 = 56.;
        const GAP: f32 = 24.;
        let fit = View::fit(
            Rect::from_min_max(avail.min, avail.max - Vec2::new(0., STRIP + GAP)).shrink(16.),
        );
        let block = fit.rect.height() + GAP + STRIP;
        let top = (avail.center().y - block / 2.).max(avail.top() + 16.);
        let view = View {
            rect: Rect::from_min_size(Pos2::new(fit.rect.left(), top), fit.rect.size()),
            cell: fit.cell,
        };
        let strip = Rect::from_min_size(
            Pos2::new(view.rect.left() - 10., view.rect.bottom() + GAP),
            Vec2::new(view.rect.width() + 20., STRIP),
        );
        incoming_strip(ui, strip, &b, pal);
        self.view = Some(view);
        let response = ui.allocate_rect(view.rect, Sense::click());
        let interactive =
            !self.blocked && !b.finished() && self.panel == Panel::None && self.enabled;
        let mut hovered_tower = None;
        if interactive {
            // Pointer movement over the board moves the shared cursor; a pointer
            // that merely rests there never overrides keyboard movement.
            if let (Some(pos), Some(last)) = (response.hover_pos(), self.last_pointer) {
                if pos != last {
                    if let Some(cell) = view.cell_at(pos) {
                        self.cursor = cell;
                    }
                }
            }
            self.last_pointer = response.hover_pos();
            hovered_tower = response
                .hover_pos()
                .and_then(|p| view.cell_at(p))
                .and_then(|(x, y)| b.tower_at(x, y))
                .map(|t| t.id);
            if response.clicked() {
                if let Some((x, y)) = response
                    .interact_pointer_pos()
                    .and_then(|p| view.cell_at(p))
                {
                    self.cursor = (x, y);
                    if self.pending.is_some() {
                        self.place(x, y);
                    } else {
                        self.select_at(x, y);
                    }
                }
            }
            if response.secondary_clicked() {
                if self.pending.is_some() {
                    self.pending = None;
                } else {
                    self.selected = None;
                }
            }
        }
        let c = view.cell;
        let p = ui.painter_at(view.rect.expand(16.));
        arcade_presentation::bezel(&p, view.rect, pal.accent);
        let board = ui.painter_at(view.rect);
        art::terrain(&board, view, b.data(), b.map, pal);
        // Placement mode reveals every open build cell.
        if let Some(kind) = self.pending {
            for y in 0..ROWS {
                for x in 0..COLS {
                    if b.data().terrain(x, y) == Terrain::Build && b.tower_at(x, y).is_none() {
                        board.circle_filled(
                            view.cell_rect(x, y).center(),
                            (c * 0.06).max(1.5),
                            pal.good.gamma_multiply(0.55),
                        );
                    }
                }
            }
            let _ = kind;
        }
        if let Some(id) =
            hovered_tower.filter(|id| Some(*id) != self.selected && self.pending.is_none())
        {
            if let Some(t) = b.tower(id) {
                art::range(
                    &board,
                    view,
                    t.pos(),
                    &t.stats(),
                    pal.text.gamma_multiply(0.6),
                );
            }
        }
        if let Some(t) = self.selected.and_then(|id| b.tower(id)) {
            art::range(&board, view, t.pos(), &t.stats(), BRASS);
        }
        for t in &b.towers {
            art::tower(&board, view, &b, t, pal);
        }
        if let Some(t) = self.selected.and_then(|id| b.tower(id)) {
            brackets(&board, view.cell_rect(t.x, t.y), BRASS, 2.);
        }
        let mut enemies: Vec<_> = b.enemies.iter().collect();
        enemies.sort_by_key(|e| e.kind.stats().air);
        for e in &enemies {
            if !e.kind.stats().air {
                art::enemy(&board, view, &b, e, pal);
            }
        }
        for s in &b.shots {
            art::shot(&board, view, s, pal);
        }
        for e in &enemies {
            if e.kind.stats().air {
                art::enemy(&board, view, &b, e, pal);
            }
        }
        let now = Instant::now();
        self.effects
            .retain(|e| now.duration_since(e.born).as_secs_f32() < e.life);
        for e in &self.effects {
            let age = now.duration_since(e.born).as_secs_f32() / e.life;
            match e.fx {
                Fx::Ring { at, radius, colour } => {
                    art::ring(&board, view.pt(at), view.len(radius), age, colour)
                }
                Fx::Burst { at, seed, colour } => {
                    art::burst(&board, view.pt(at), c, age, seed, colour)
                }
                Fx::Flash { from, to, colour } => art::flash(
                    &board,
                    view.pt(from),
                    (view.pt(to) - view.pt(from)).normalized(),
                    c,
                    age,
                    colour,
                ),
                Fx::Gate { at } => art::ring(&board, view.pt(at), c * 1.3, age, pal.bad),
            }
        }
        // Cursor and placement preview.
        let (cx, cy) = self.cursor;
        let cell = view.cell_rect(cx, cy);
        if let Some(kind) = self.pending {
            let check = b.check_build(cx, cy, kind);
            let colour = if check.is_ok() { pal.good } else { pal.bad };
            art::range(&board, view, P::cell(cx, cy), &tower(kind, 0), colour);
            board.rect_filled(cell, 2., colour.gamma_multiply(0.18));
            art::tower_icon(
                &board,
                cell.center(),
                c * 0.95,
                kind,
                0,
                pal,
                Vec2::new(0.7, -0.7),
            );
            brackets(&board, cell, colour, 2.5);
            // Validity is also a symbol, not colour alone.
            board.text(
                cell.right_top() + Vec2::new(-3., 1.),
                Align2::RIGHT_TOP,
                if check.is_ok() { "+" } else { "×" },
                ui::mono(c * 0.42),
                colour,
            );
        } else if interactive {
            brackets(&board, cell, pal.text.gamma_multiply(0.85), 1.6);
        }
        art::vignette(
            &board,
            view.rect,
            c * 1.2,
            Color32::from_black_alpha(if pal.light { 40 } else { 110 }),
        );
        if let Some(t) = self.breach {
            let age = t.elapsed().as_secs_f32() / 0.7;
            if age < 1. {
                art::vignette(
                    &board,
                    view.rect,
                    c * 1.6,
                    pal.bad.gamma_multiply(0.55 * (1. - age)),
                );
            }
        }
        // Status plate at the top of the board.
        let plate = match (b.phase, self.paused) {
            (Phase::Waiting, _) if interactive => {
                Some((format!("Wave {} ready", b.wave + 1), "Space", "to send it"))
            }
            (Phase::Running, true) => Some((
                self.pause_reason
                    .clone()
                    .unwrap_or_else(|| "Paused · building allowed".into()),
                "Esc",
                "to resume",
            )),
            _ => None,
        };
        if let Some((text, key, tail)) = plate {
            status_plate(
                &p,
                view.rect.center_top() + Vec2::new(0., 22.),
                &text,
                key,
                tail,
                pal,
            );
        }
        if let Some((head, sub, at)) = &self.banner {
            let age = at.elapsed().as_secs_f32() / 1.8;
            if age < 1. {
                banner(&p, view.rect, head, sub, age, pal);
            }
        }
        if let Some((text, at)) = &self.notice {
            if at.elapsed() < Duration::from_millis(2600) {
                toast(
                    &p,
                    view.rect.center_bottom() - Vec2::new(0., 26.),
                    text,
                    pal,
                );
            }
        }
        if self.panel != Panel::None {
            p.rect_filled(
                view.rect,
                0.,
                if pal.light {
                    Color32::from_white_alpha(90)
                } else {
                    Color32::from_black_alpha(120)
                },
            );
        }
    }

    // ----- Campaign screen ------------------------------------------------------

    fn campaign(&mut self, ui: &mut egui::Ui, pal: &Palette) {
        arcade_presentation::backdrop(ui);
        let active = self.panel == Panel::Maps && !self.blocked;
        let full = ui.available_rect_before_wrap();
        let mut content =
            ui.new_child(egui::UiBuilder::new().max_rect(full.shrink2(Vec2::new(28., 18.))));
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(&mut content, |ui| {
                let w = ui.available_width();
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("OMARCHY ARCADE  ·  TOWER DEFENCE")
                                .font(ui::mono(11.))
                                .color(BRASS),
                        );
                        ui.label(
                            egui::RichText::new("RIDGELINE")
                                .font(ui::mono(if w < 800. { 30. } else { 40. }))
                                .color(pal.text)
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new("Hold the pass. Spend every credit well.")
                                .font(ui::text(15.))
                                .color(pal.muted),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let records = &self.save.progress.records;
                        let cleared = records.iter().filter(|r| r[0].medal != Medal::None).count();
                        let hard = records.iter().filter(|r| r[1].medal != Medal::None).count();
                        let perfect = records
                            .iter()
                            .flatten()
                            .filter(|r| r.medal == Medal::Perfect)
                            .count();
                        for (label, value) in
                            [("PERFECT", perfect), ("HARD", hard), ("CLEARED", cleared)]
                        {
                            let (r, _) =
                                ui.allocate_exact_size(Vec2::new(78., 50.), Sense::hover());
                            let p = ui.painter();
                            p.text(
                                r.center_top() + Vec2::new(0., 4.),
                                Align2::CENTER_TOP,
                                format!("{value}/{}", maps().len()),
                                ui::mono(20.),
                                pal.text,
                            );
                            p.text(
                                r.center_bottom(),
                                Align2::CENTER_BOTTOM,
                                label,
                                ui::mono(9.5),
                                pal.muted,
                            );
                        }
                    });
                });
                ui.add_space(10.);
                if let Some(b) = self.save.battle.clone().filter(|b| !b.finished()) {
                    let (r, _) = ui.allocate_exact_size(Vec2::new(w, 58.), Sense::hover());
                    let p = ui.painter();
                    ui::card(p, r, pal, false, true);
                    p.text(
                        r.left_center() + Vec2::new(18., -9.),
                        Align2::LEFT_CENTER,
                        "DEFENCE IN PROGRESS",
                        ui::mono(10.),
                        BRASS,
                    );
                    p.text(
                        r.left_center() + Vec2::new(18., 9.),
                        Align2::LEFT_CENTER,
                        format!(
                            "{} · {} · wave {} · base {}/{}",
                            b.data().name,
                            b.difficulty.name(),
                            b.wave + 1,
                            b.health,
                            BASE_HEALTH
                        ),
                        ui::text(14.),
                        pal.text,
                    );
                    let mut child =
                        ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_max(
                            r.right_top() - Vec2::new(170., -12.),
                            r.right_bottom() - Vec2::new(12., 12.),
                        )));
                    if ui::button(
                        &mut child,
                        "Continue",
                        Some("Esc"),
                        Tone::Primary,
                        Vec2::new(158., 34.),
                        active,
                        pal,
                    )
                    .clicked()
                    {
                        self.panel = Panel::None;
                    }
                    ui.add_space(8.);
                }
                let columns = ((w + 14.) / 300.).floor().clamp(1., 5.) as usize;
                let card_w = (w - 14. * (columns - 1) as f32) / columns as f32;
                for row in (0..maps().len()).collect::<Vec<_>>().chunks(columns) {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 14.;
                        for &i in row {
                            self.map_card(ui, i, card_w, active, pal);
                        }
                    });
                    ui.add_space(14.);
                }
                let (r, _) = ui.allocate_exact_size(Vec2::new(w, 34.), Sense::hover());
                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(r));
                child.horizontal(|ui| {
                    if ui::button(
                        ui,
                        "Back to Arcade",
                        Some("Ctrl+H"),
                        Tone::Secondary,
                        Vec2::new(190., 32.),
                        true,
                        pal,
                    )
                    .clicked()
                    {
                        self.suspend();
                        self.leave = true;
                    }
                    ui.label(
                        egui::RichText::new(
                            "Normal wins unlock the next map and that map's Hard table.",
                        )
                        .font(ui::text(12.))
                        .color(pal.muted),
                    );
                });
            });
    }

    fn map_card(&mut self, ui: &mut egui::Ui, i: usize, w: f32, active: bool, pal: &Palette) {
        let map = &maps()[i];
        let thumb_h = w * ROWS as f32 / COLS as f32;
        let (r, _) = ui.allocate_exact_size(Vec2::new(w, thumb_h + 132.), Sense::hover());
        let p = ui.painter().clone();
        let unlocked = self.save.progress.unlocked(i, Difficulty::Normal);
        ui::card(&p, r, pal, false, false);
        let thumb = Rect::from_min_size(
            r.min + Vec2::splat(6.),
            Vec2::new(w - 12., thumb_h - 12. * ROWS as f32 / COLS as f32),
        );
        let view = View::fit(thumb);
        let tp = p.with_clip_rect(view.rect);
        art::thumbnail(&tp, view, map, i, pal);
        if !unlocked {
            tp.rect_filled(view.rect, 0., Color32::from_black_alpha(150));
            ui::lock(&p, view.rect.center(), 14., IVORY.gamma_multiply(0.85));
        }
        p.rect_stroke(
            view.rect,
            2.,
            Stroke::new(1_f32, pal.line),
            egui::StrokeKind::Outside,
        );
        let badge = Rect::from_min_size(view.rect.min + Vec2::splat(6.), Vec2::new(30., 20.));
        p.rect_filled(badge, 3., Color32::from_black_alpha(170));
        p.text(
            badge.center(),
            Align2::CENTER_CENTER,
            format!("{:02}", i + 1),
            ui::mono(12.),
            BRASS,
        );
        let top = view.rect.bottom() + 10.;
        p.text(
            Pos2::new(r.left() + 12., top),
            Align2::LEFT_TOP,
            map.name,
            ui::text(16.),
            pal.text,
        );
        let lesson = p.layout(map.lesson.into(), ui::text(11.5), pal.muted, w - 24.);
        p.galley(Pos2::new(r.left() + 12., top + 23.), lesson, pal.muted);
        let row = Rect::from_min_max(
            Pos2::new(r.left() + 10., r.bottom() - 46.),
            r.right_bottom() - Vec2::new(10., 10.),
        );
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(row));
        child.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.;
            let bw = (row.width() - 8.) / 2.;
            for d in Difficulty::ALL {
                let open = self.save.progress.unlocked(i, d);
                let record = self.save.progress.record(i, d);
                let label = if open { d.name() } else { "Locked" };
                let tone = if d == Difficulty::Hard {
                    Tone::Secondary
                } else {
                    Tone::Primary
                };
                let response = ui::button(
                    ui,
                    label,
                    None,
                    if open { tone } else { Tone::Ghost },
                    Vec2::new(bw, 36.),
                    open && active,
                    pal,
                );
                let medal_at = response.rect.left_center() + Vec2::new(16., 0.);
                if record.medal != Medal::None {
                    ui::medal(ui.painter(), medal_at, 8., record.medal, pal);
                } else if !open {
                    ui::lock(ui.painter(), medal_at, 7., pal.muted);
                }
                let tip = if record.wins > 0 {
                    format!(
                        "{} · best base {}/{} · {} wins",
                        record.medal.name(),
                        record.best_health,
                        BASE_HEALTH,
                        record.wins
                    )
                } else if open {
                    format!("Play {} on {}", map.name, d.name())
                } else if d == Difficulty::Hard {
                    "Win this map on Normal to unlock Hard".into()
                } else {
                    "Win the previous map on Normal to unlock".into()
                };
                if response.on_hover_text(tip).clicked() {
                    self.choose(i, d, false);
                }
            }
        });
    }

    // ----- Modal panels ---------------------------------------------------------

    fn panels(&mut self, ctx: &egui::Context, pal: &Palette) {
        if matches!(self.panel, Panel::None | Panel::Maps) {
            return;
        }
        let width = match self.panel {
            Panel::Help => 620.,
            Panel::Result(_) => 460.,
            _ => 420_f32,
        }
        .min(ctx.screen_rect().width() - 60.);
        egui::Window::new("ridgeline-modal")
            .title_bar(false)
            .frame(ui::modal_frame(pal))
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .min_width(width)
            .max_width(width)
            .show(ctx, |ui| {
                ui.set_width(width);
                egui::ScrollArea::vertical()
                    .max_height((ctx.screen_rect().height() - 160.).max(220.))
                    .show(ui, |ui| self.panel_body(ui, pal));
            });
    }

    fn panel_body(&mut self, ui: &mut egui::Ui, pal: &Palette) {
        let w = ui.available_width();
        ui.spacing_mut().item_spacing = Vec2::new(8., 8.);
        match self.panel {
            Panel::None | Panel::Maps => {}
            Panel::Settings => {
                ui::title(ui, "SETTINGS", "Preferences", pal);
                if ui::toggle(ui, "Sound  ·  Ctrl+M", self.save.preferences.sound, pal) {
                    self.toggle_sound();
                }
                if ui::toggle(
                    ui,
                    "Reduced effects",
                    self.save.preferences.reduced_effects,
                    pal,
                ) {
                    self.save.preferences.reduced_effects = !self.save.preferences.reduced_effects;
                }
                ui.label(egui::RichText::new("Reduced effects removes muzzle flashes, impact rings and particles. Breaches still show.").font(ui::text(11.5)).color(pal.muted));
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Simulation speed")
                            .font(ui::text(13.5))
                            .color(pal.text),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(i) = ui::segmented(
                            ui,
                            &["1×", "2×"],
                            self.save.preferences.speed as usize - 1,
                            96.,
                            pal,
                        ) {
                            self.save.preferences.speed = i as u32 + 1;
                        }
                    });
                });
                ui.add_space(6.);
                if ui::button(
                    ui,
                    "Done",
                    Some("Esc"),
                    Tone::Primary,
                    Vec2::new(w, 36.),
                    true,
                    pal,
                )
                .clicked()
                {
                    self.persist();
                    self.close_panel();
                }
            }
            Panel::Help => {
                ui::title(ui, "HOW TO PLAY", "Hold the pass", pal);
                ui.label(egui::RichText::new("Enemies follow the roads, and aircraft follow the dotted flight lanes, to the brass gates. Every one that gets through costs base health. Hold all 20 waves to win. Waves only start when you send them.").font(ui::text(13.)).color(pal.text));
                ui::section(ui, "TOWERS", Some("build on open terrace"), pal);
                for kind in TowerKind::ALL {
                    help_row(
                        ui,
                        pal,
                        |p, c| art::tower_icon(p, c, 34., kind, 0, pal, Vec2::new(0.7, -0.7)),
                        kind.name(),
                        &format!(
                            "{} · {} · {} credits",
                            kind.role(),
                            kind.targets().name(),
                            tower(kind, 0).cost
                        ),
                    );
                }
                ui::section(ui, "ENEMIES", None, pal);
                for kind in EnemyKind::ALL {
                    let s = kind.stats();
                    help_row(
                        ui,
                        pal,
                        |p, c| art::enemy_icon(p, c, 11., kind, pal, Vec2::new(1., 0.), 0.),
                        kind.name(),
                        &format!(
                            "{} · armour {} · base damage {}",
                            kind.role(),
                            s.armour,
                            s.damage
                        ),
                    );
                }
                ui::section(ui, "RULES", None, pal);
                for line in [
                    "Armour subtracts from every hit, but each hit does at least 1 damage.",
                    "The strongest Cryo slow applies; slows never stack or stop a unit.",
                    "Kills pay a bounty once. Holding a wave pays a fixed award. Selling refunds 70%.",
                    "Upgrades apply at once and keep the current reload. Mortars cannot fire up close.",
                ] {
                    ui.label(egui::RichText::new(format!("·  {line}")).font(ui::text(12.5)).color(pal.text));
                }
                ui::section(ui, "CONTROLS", None, pal);
                for (keys, action) in [
                    ("1 2 3 4", "Choose Cannon, Mortar, Flak or Cryo"),
                    ("← ↑ → ↓", "Move the build cursor"),
                    ("Enter", "Place, or inspect the tower under the cursor"),
                    ("Esc", "Cancel placement, then pause or resume"),
                    ("U", "Upgrade the selected tower"),
                    ("Space", "Start the next wave"),
                    ("F", "Toggle 1× / 2× speed"),
                    ("F1  Ctrl+,  Ctrl+M", "Help · Settings · Sound"),
                    ("Ctrl+H  Ctrl+Q", "Back to Arcade · Quit"),
                ] {
                    let (r, _) = ui
                        .allocate_exact_size(Vec2::new(ui.available_width(), 22.), Sense::hover());
                    let p = ui.painter();
                    let mut x = r.left();
                    for k in keys.split_whitespace() {
                        x += ui::keycap(p, Pos2::new(x + 12., r.center().y), k, pal) + 4.;
                    }
                    p.text(
                        Pos2::new(r.left() + 160., r.center().y),
                        Align2::LEFT_CENTER,
                        action,
                        ui::text(12.5),
                        pal.text,
                    );
                }
                ui.label(egui::RichText::new("Mouse: pick a tower card, click open terrace to build, click a tower to inspect it, right-click to cancel.").font(ui::text(12.)).color(pal.muted));
                ui.add_space(4.);
                ui.label(egui::RichText::new("Ridgeline is an original Omarchy Arcade game: maps, waves, artwork and synthesized sound. Canyon Defense and other tower-defence games are design references only; nothing is copied.").font(ui::text(11.)).color(pal.muted));
                if ui::button(
                    ui,
                    "Close",
                    Some("Esc"),
                    Tone::Primary,
                    Vec2::new(ui.available_width(), 36.),
                    true,
                    pal,
                )
                .clicked()
                {
                    self.close_panel();
                }
            }
            Panel::Restart => {
                ui::title(ui, "RESTART", "Restart this map?", pal);
                ui.label(egui::RichText::new("Your towers and credits reset to wave 1. Medals, records and unlocks are kept.").font(ui::text(13.)).color(pal.text));
                ui.add_space(6.);
                ui.horizontal(|ui| {
                    if ui::button(
                        ui,
                        "Restart map",
                        None,
                        Tone::Danger,
                        Vec2::new(w / 2. - 4., 36.),
                        true,
                        pal,
                    )
                    .clicked()
                    {
                        if let Some(b) = &self.save.battle {
                            let (map, d) = (b.map, b.difficulty);
                            self.choose(map, d, true);
                        }
                    }
                    if ui::button(
                        ui,
                        "Keep defending",
                        Some("Esc"),
                        Tone::Secondary,
                        Vec2::new(w / 2. - 4., 36.),
                        true,
                        pal,
                    )
                    .clicked()
                    {
                        self.close_panel();
                    }
                });
            }
            Panel::Replace { map, difficulty } => {
                ui::title(ui, "UNFINISHED DEFENCE", "Replace this defence?", pal);
                if let Some(b) = &self.save.battle {
                    ui.label(egui::RichText::new(format!(
                        "{} on {} is at wave {} with base {}/{}. Starting {} on {} replaces it. Medals, records and unlocks are kept.",
                        b.data().name,
                        b.difficulty.name(),
                        b.wave + 1,
                        b.health,
                        BASE_HEALTH,
                        maps()[map].name,
                        difficulty.name()
                    )).font(ui::text(13.)).color(pal.text));
                }
                ui.add_space(6.);
                ui.horizontal(|ui| {
                    if ui::button(
                        ui,
                        "Replace defence",
                        None,
                        Tone::Danger,
                        Vec2::new(w / 2. - 4., 36.),
                        true,
                        pal,
                    )
                    .clicked()
                    {
                        self.choose(map, difficulty, true);
                    }
                    if ui::button(
                        ui,
                        "Keep current defence",
                        None,
                        Tone::Secondary,
                        Vec2::new(w / 2. - 4., 36.),
                        true,
                        pal,
                    )
                    .clicked()
                    {
                        self.panel = Panel::Maps;
                    }
                });
            }
            Panel::Result(award) => self.result(ui, award, pal),
            Panel::Recovery => {
                ui::title(
                    ui,
                    "SAVE PROTECTED",
                    "Your campaign file needs attention",
                    pal,
                );
                if let Some(e) = self.error.clone() {
                    alert(ui, &e, pal);
                }
                ui.label(egui::RichText::new("Nothing is written while the original is protected. Archive it to a unique recovery file beside it and start a fresh campaign? The archive is never overwritten.").font(ui::text(13.)).color(pal.text));
                ui.add_space(6.);
                ui.horizontal(|ui| {
                    if ui::button(
                        ui,
                        "Archive and start fresh",
                        None,
                        Tone::Primary,
                        Vec2::new(w * 0.58 - 4., 36.),
                        true,
                        pal,
                    )
                    .clicked()
                    {
                        match storage::archive(&self.path) {
                            Ok(backup) => {
                                self.blocked = false;
                                self.save = Save::default();
                                self.error =
                                    Some(format!("Original archived at {}", backup.display()));
                                self.persist();
                                self.panel = Panel::Maps;
                            }
                            Err(e) => {
                                self.error = Some(format!("Archive failed; nothing changed: {e}"))
                            }
                        }
                    }
                    if ui::button(
                        ui,
                        "Back to Arcade",
                        None,
                        Tone::Secondary,
                        Vec2::new(w * 0.42 - 4., 36.),
                        true,
                        pal,
                    )
                    .clicked()
                    {
                        self.suspend();
                        self.leave = true;
                    }
                });
            }
        }
    }

    fn result(&mut self, ui: &mut egui::Ui, award: Option<Award>, pal: &Palette) {
        let Some(b) = self.save.battle.clone() else {
            return;
        };
        let w = ui.available_width();
        let won = b.phase == Phase::Victory;
        let record = self.save.progress.record(b.map, b.difficulty);
        let medal = award.map_or(if won { record.medal } else { Medal::None }, |a| a.medal);
        let (r, _) = ui.allocate_exact_size(Vec2::new(w, 96.), Sense::hover());
        let p = ui.painter();
        if won {
            ui::medal(p, r.center(), 34., medal, pal);
        } else {
            let c = r.center();
            ui::shield(p, c, 32., pal.metal_lit, pal);
            p.add(Shape::line(
                vec![
                    c + Vec2::new(-4., -26.),
                    c + Vec2::new(6., -6.),
                    c + Vec2::new(-6., 8.),
                    c + Vec2::new(4., 30.),
                ],
                Stroke::new(3_f32, pal.bad),
            ));
        }
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new(if won { "VICTORY" } else { "DEFEATED" })
                    .font(ui::mono(30.))
                    .color(if won { pal.gold } else { pal.bad })
                    .strong(),
            );
            ui.label(
                egui::RichText::new(format!("{} · {}", b.data().name, b.difficulty.name()))
                    .font(ui::text(14.))
                    .color(pal.muted),
            );
            if won {
                ui.label(
                    egui::RichText::new(format!(
                        "Medal: {}{}",
                        medal.name(),
                        if medal == Medal::Perfect {
                            " — no damage taken"
                        } else {
                            ""
                        }
                    ))
                    .font(ui::text(13.))
                    .color(pal.text),
                );
            } else {
                ui.label(
                    egui::RichText::new(format!("The line broke on wave {}.", b.wave + 1))
                        .font(ui::text(13.))
                        .color(pal.text),
                );
            }
        });
        ui.add_space(6.);
        let (r, _) = ui.allocate_exact_size(Vec2::new(w, 58.), Sense::hover());
        let p = ui.painter();
        let tiles = [
            ("BASE", format!("{}/{}", b.health, BASE_HEALTH)),
            ("DESTROYED", b.kills.to_string()),
            ("BREACHES", b.leaked.to_string()),
            (
                "COMBAT",
                format!("{}:{:02}", b.tick / 3600, b.tick / 60 % 60),
            ),
        ];
        let tw = (w - 18.) / 4.;
        for (k, (label, value)) in tiles.iter().enumerate() {
            let t = Rect::from_min_size(
                r.min + Vec2::new(k as f32 * (tw + 6.), 0.),
                Vec2::new(tw, 58.),
            );
            ui::card(p, t, pal, false, false);
            p.text(
                t.center_top() + Vec2::new(0., 10.),
                Align2::CENTER_TOP,
                value,
                ui::mono(17.),
                pal.text,
            );
            p.text(
                t.center_bottom() - Vec2::new(0., 9.),
                Align2::CENTER_BOTTOM,
                *label,
                ui::mono(9.),
                pal.muted,
            );
        }
        if let Some(a) = award {
            if a.unlocked_next && b.map + 1 < maps().len() {
                unlock_line(
                    ui,
                    &format!("Unlocked map {:02}: {}", b.map + 2, maps()[b.map + 1].name),
                    pal,
                );
            }
            if a.unlocked_hard {
                unlock_line(ui, &format!("Unlocked {} on Hard", b.data().name), pal);
            }
        }
        ui.add_space(8.);
        let next = b.map + 1;
        let can_next =
            won && next < maps().len() && self.save.progress.unlocked(next, Difficulty::Normal);
        let bw = if can_next {
            (w - 16.) / 3.
        } else {
            (w - 8.) / 2.
        };
        ui.horizontal(|ui| {
            if can_next
                && ui::button(
                    ui,
                    &format!("Next: {}", maps()[next].name),
                    None,
                    Tone::Primary,
                    Vec2::new(bw, 38.),
                    true,
                    pal,
                )
                .clicked()
            {
                self.choose(next, Difficulty::Normal, true);
            }
            let again_tone = if can_next {
                Tone::Secondary
            } else {
                Tone::Primary
            };
            if ui::button(
                ui,
                "Play again",
                None,
                again_tone,
                Vec2::new(bw, 38.),
                true,
                pal,
            )
            .clicked()
            {
                self.choose(b.map, b.difficulty, true);
            }
            if ui::button(
                ui,
                "Maps",
                None,
                Tone::Secondary,
                Vec2::new(bw, 38.),
                true,
                pal,
            )
            .clicked()
            {
                self.panel = Panel::Maps;
            }
        });
    }

    fn draw(&mut self, ctx: &egui::Context) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last).as_secs_f64();
        self.last = now;
        if self.themed.elapsed() > Duration::from_secs(2) {
            self.theme = arcade_platform::theme::Theme::load();
            self.themed = now;
        }
        let pal = Palette::new(&self.theme);
        let mut visuals = if pal.light {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };
        visuals.override_text_color = Some(pal.text);
        visuals.panel_fill = self.theme.background;
        visuals.window_fill = self.theme.background;
        visuals.selection.bg_fill = self.theme.accent;
        visuals.selection.stroke = Stroke::new(1_f32, self.theme.accent_text());
        ctx.set_visuals(visuals);
        arcade_presentation::apply(ctx);
        let focused = ctx.input(|i| i.focused);
        if !focused || !self.enabled {
            self.pause(Some("Paused because you left the game"));
        }
        if self.enabled && focused {
            self.keyboard(ctx);
        }
        let bar = egui::Frame::NONE
            .fill(self.theme.background)
            .inner_margin(egui::Margin::symmetric(16, 8))
            .stroke(Stroke::new(1_f32, pal.line));
        // The campaign screen carries its own header until a battle exists.
        if self.save.battle.is_some() {
            egui::TopBottomPanel::top("ridgeline-hud")
                .frame(bar)
                .show(ctx, |ui| self.hud(ui, &pal));
        }
        let campaign = self.save.battle.is_none() || self.panel == Panel::Maps;
        if !campaign {
            egui::SidePanel::right("ridgeline-side")
                .resizable(false)
                .exact_width(if ctx.screen_rect().width() < 1000. {
                    262.
                } else {
                    292.
                })
                .frame(
                    egui::Frame::NONE
                        .fill(self.theme.background)
                        .inner_margin(egui::Margin::same(12))
                        .stroke(Stroke::new(1_f32, pal.line)),
                )
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| self.sidebar(ui, &pal));
                });
        }
        let active = self.enabled
            && focused
            && !self.paused
            && !self.blocked
            && self.panel == Panel::None
            && self.running();
        if active {
            self.simulate(elapsed, now);
        } else {
            self.clock.reset();
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(self.theme.background))
            .show(ctx, |ui| {
                if campaign {
                    self.campaign(ui, &pal);
                } else {
                    self.board(ui, &pal);
                }
            });
        self.panels(ctx, &pal);
        // Pointer clicks must not leave a button holding Space/Enter.
        if ctx.input(|i| i.pointer.any_click()) {
            ctx.memory_mut(|m| {
                if let Some(id) = m.focused() {
                    m.surrender_focus(id);
                }
            });
        }
        if self.running() && !self.paused && self.checkpoint.elapsed() > Duration::from_secs(15) {
            self.persist();
        }
        let animating = self
            .banner
            .as_ref()
            .is_some_and(|b| b.2.elapsed() < Duration::from_secs(2))
            || !self.effects.is_empty()
            || self
                .breach
                .is_some_and(|t| t.elapsed() < Duration::from_secs(1));
        ctx.request_repaint_after(if active || animating {
            Duration::from_millis(8)
        } else {
            Duration::from_millis(100)
        });
    }
}

/// Live wave progress shown in place of the start button while a wave runs.
fn wave_status(ui: &mut egui::Ui, b: &Battle, size: Vec2, paused: bool, pal: &Palette) {
    let (r, _) = ui.allocate_exact_size(size, Sense::hover());
    let p = ui.painter();
    ui::card(p, r, pal, false, false);
    let total = b.preview().spawns.len().max(1);
    let to_come = total - b.spawned;
    let resolved = total - to_come - b.enemies.len();
    let t = ui.input(|i| i.time) as f32;
    let pulse = if paused {
        0.5
    } else {
        0.55 + 0.45 * (t * 4.).sin()
    };
    p.circle_filled(
        r.left_center() + Vec2::new(14., -4.),
        4.,
        pal.accent.gamma_multiply(pulse),
    );
    p.text(
        r.left_center() + Vec2::new(26., -4.),
        Align2::LEFT_CENTER,
        format!("Wave {} live", b.wave + 1),
        ui::text(13.),
        pal.text,
    );
    p.text(
        r.right_center() + Vec2::new(-10., -4.),
        Align2::RIGHT_CENTER,
        format!("{} left", to_come + b.enemies.len()),
        ui::mono(10.5),
        pal.muted,
    );
    let bar = Rect::from_min_size(
        Pos2::new(r.left() + 10., r.bottom() - 9.),
        Vec2::new(r.width() - 20., 3.),
    );
    p.rect_filled(bar, 1.5, pal.line);
    p.rect_filled(
        Rect::from_min_size(
            bar.min,
            Vec2::new(bar.width() * resolved as f32 / total as f32, 3.),
        ),
        1.5,
        pal.accent,
    );
    if !paused {
        ui.ctx().request_repaint_after(Duration::from_millis(33));
    }
}

fn divider(ui: &mut egui::Ui, pal: &Palette) {
    let (r, _) = ui.allocate_exact_size(Vec2::new(1., 40.), Sense::hover());
    ui.painter().line_segment(
        [r.center_top(), r.center_bottom()],
        Stroke::new(1_f32, pal.line),
    );
}

/// Corner brackets around a cell: a crisp cursor that never hides its contents.
fn brackets(p: &egui::Painter, r: Rect, colour: Color32, width: f32) {
    let l = r.width() * 0.28;
    for (corner, dx, dy) in [
        (r.left_top(), 1., 1.),
        (r.right_top(), -1., 1.),
        (r.left_bottom(), 1., -1.),
        (r.right_bottom(), -1., -1.),
    ] {
        p.add(Shape::line(
            vec![
                corner + Vec2::new(dx * l, 0.),
                corner,
                corner + Vec2::new(0., dy * l),
            ],
            Stroke::new(width, colour),
        ));
    }
}

fn panel_card(ui: &mut egui::Ui, pal: &Palette, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::NONE
        .fill(pal.card)
        .stroke(Stroke::new(1_f32, pal.line))
        .corner_radius(ui::RADIUS as u8)
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui);
        });
}

fn alert(ui: &mut egui::Ui, text: &str, pal: &Palette) {
    egui::Frame::NONE
        .fill(pal.bad.gamma_multiply(0.12))
        .stroke(Stroke::new(1_f32, pal.bad.gamma_multiply(0.6)))
        .corner_radius(ui::RADIUS as u8)
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                egui::RichText::new(text)
                    .font(ui::text(12.))
                    .color(pal.text),
            );
        });
}

fn tower_stats(ui: &mut egui::Ui, now: &TowerStats, next: Option<&TowerStats>, pal: &Palette) {
    let rate = |s: &TowerStats| 60. / s.reload as f32;
    let fr = |v: f32, max: f32| v / max;
    let delta = |a: f32, b: f32, fmt: &dyn Fn(f32) -> String| (b > a + 1e-3).then(|| fmt(b - a));
    let d = next.and_then(|n| delta(now.damage as f32, n.damage as f32, &|v| format!("+{v:.0}")));
    ui::stat(
        ui,
        "Damage",
        &now.damage.to_string(),
        fr(now.damage as f32, 44.),
        next.zip(d.as_deref())
            .map(|(n, t)| (fr(n.damage as f32, 44.), t)),
        pal,
    );
    let d = next.and_then(|n| {
        delta(now.range as f32, n.range as f32, &|v| {
            format!("+{:.1}", v / CELL as f32)
        })
    });
    ui::stat(
        ui,
        "Range",
        &format!("{:.1}", now.range as f32 / CELL as f32),
        fr(now.range as f32, 4900.),
        next.zip(d.as_deref())
            .map(|(n, t)| (fr(n.range as f32, 4900.), t)),
        pal,
    );
    let d = next.and_then(|n| delta(rate(now), rate(n), &|v| format!("+{v:.2}")));
    ui::stat(
        ui,
        "Shots per second",
        &format!("{:.2}", rate(now)),
        fr(rate(now), 6.),
        next.zip(d.as_deref()).map(|(n, t)| (fr(rate(n), 6.), t)),
        pal,
    );
    if now.splash > 0 {
        let d = next.and_then(|n| {
            delta(now.splash as f32, n.splash as f32, &|v| {
                format!("+{:.2}", v / CELL as f32)
            })
        });
        ui::stat(
            ui,
            "Splash radius",
            &format!("{:.2}", now.splash as f32 / CELL as f32),
            fr(now.splash as f32, 1150.),
            next.zip(d.as_deref())
                .map(|(n, t)| (fr(n.splash as f32, 1150.), t)),
            pal,
        );
        ui.label(
            egui::RichText::new(format!(
                "Cannot fire within {:.1} cells",
                now.min_range as f32 / CELL as f32
            ))
            .font(ui::text(11.))
            .color(pal.muted),
        );
    }
    if now.slow > 0 {
        let d = next.and_then(|n| delta(now.slow as f32, n.slow as f32, &|v| format!("+{v:.0}%")));
        ui::stat(
            ui,
            "Slow",
            &format!("{}% · {:.1}s", now.slow, now.slow_ticks as f32 / 60.),
            fr(now.slow as f32, 50.),
            next.zip(d.as_deref())
                .map(|(n, t)| (fr(n.slow as f32, 50.), t)),
            pal,
        );
    }
}

/// The next (or current) wave as one strip beneath the board.
fn incoming_strip(ui: &egui::Ui, r: Rect, b: &Battle, pal: &Palette) {
    let p = ui.painter();
    let wave = b.preview();
    ui::card(p, r, pal, false, false);
    let (eyebrow, colour) = match b.phase {
        Phase::Waiting => ("INCOMING", BRASS),
        Phase::Running => ("ON FIELD", pal.accent),
        _ => ("FINAL WAVE", pal.muted),
    };
    p.text(
        r.left_top() + Vec2::new(14., 11.),
        Align2::LEFT_TOP,
        eyebrow,
        ui::mono(10.),
        colour,
    );
    p.text(
        r.left_bottom() + Vec2::new(14., -10.),
        Align2::LEFT_BOTTOM,
        format!("WAVE {:02}", b.wave + 1),
        ui::mono(17.),
        pal.text,
    );
    let mut x = r.left() + 112.;
    let mut threat = 0;
    let compact = r.width() < 760.;
    for g in &wave.groups {
        let s = g.kind.stats();
        threat += s.damage * g.count;
        let hp = (s.hp * wave.health / 100).max(1);
        p.line_segment(
            [
                Pos2::new(x - 10., r.top() + 10.),
                Pos2::new(x - 10., r.bottom() - 10.),
            ],
            Stroke::new(1_f32, pal.line),
        );
        let size = if g.kind == EnemyKind::Hauler { 11. } else { 9. };
        art::enemy_icon(
            p,
            Pos2::new(x + 12., r.center().y),
            size,
            g.kind,
            pal,
            Vec2::new(1., 0.),
            0.4,
        );
        let head = p.layout_no_wrap(
            format!("×{} {}", g.count, g.kind.name()),
            ui::text(13.5),
            pal.text,
        );
        let mut detail = format!("{hp} hp");
        if s.armour > 0 {
            detail += &format!(" · armour {}", s.armour);
        }
        if s.air {
            detail += " · air";
        }
        let sub = p.layout_no_wrap(
            detail,
            ui::mono(if compact { 9. } else { 10. }),
            if s.air { pal.air } else { pal.muted },
        );
        let w = head.size().x.max(sub.size().x);
        p.galley(
            Pos2::new(x + 30., r.center().y - head.size().y),
            head,
            pal.text,
        );
        p.galley(Pos2::new(x + 30., r.center().y + 2.), sub, pal.muted);
        x += 30. + w + 22.;
    }
    let right = r.right() - 14.;
    if x < right - 120. {
        p.text(
            Pos2::new(right, r.center().y - 8.),
            Align2::RIGHT_CENTER,
            format!("−{threat}"),
            ui::mono(16.),
            pal.bad,
        );
        p.text(
            Pos2::new(right, r.center().y + 10.),
            Align2::RIGHT_CENTER,
            "base if all pass",
            ui::mono(9.),
            pal.muted,
        );
    }
}

fn help_row(
    ui: &mut egui::Ui,
    pal: &Palette,
    icon: impl FnOnce(&egui::Painter, Pos2),
    name: &str,
    detail: &str,
) {
    let (r, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 40.), Sense::hover());
    let p = ui.painter();
    icon(p, r.left_center() + Vec2::new(20., 0.));
    p.text(
        r.left_center() + Vec2::new(48., -8.),
        Align2::LEFT_CENTER,
        name,
        ui::text(14.),
        pal.text,
    );
    p.text(
        r.left_center() + Vec2::new(48., 9.),
        Align2::LEFT_CENTER,
        detail,
        ui::text(11.5),
        pal.muted,
    );
}

fn unlock_line(ui: &mut egui::Ui, text: &str, pal: &Palette) {
    let (r, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 26.), Sense::hover());
    let p = ui.painter();
    p.rect_filled(r, ui::RADIUS, pal.accent.gamma_multiply(0.14));
    p.rect_stroke(
        r,
        ui::RADIUS,
        Stroke::new(1_f32, pal.accent.gamma_multiply(0.6)),
        egui::StrokeKind::Inside,
    );
    ui::lock(p, r.left_center() + Vec2::new(14., 0.), 6., pal.accent);
    p.text(
        r.left_center() + Vec2::new(28., 0.),
        Align2::LEFT_CENTER,
        text,
        ui::text(12.5),
        pal.text,
    );
}

fn status_plate(p: &egui::Painter, at: Pos2, text: &str, key: &str, tail: &str, pal: &Palette) {
    let g = p.layout_no_wrap(text.to_owned(), ui::text(14.), pal.text);
    let t = p.layout_no_wrap(tail.to_owned(), ui::text(12.), pal.muted);
    let kw = p
        .layout_no_wrap(key.to_owned(), ui::mono(10.), pal.muted)
        .size()
        .x
        + 8.;
    let width = g.size().x + kw + t.size().x + 44.;
    let r = Rect::from_center_size(at, Vec2::new(width, 32.));
    p.rect_filled(
        r.translate(Vec2::new(0., 3.)),
        16.,
        Color32::from_black_alpha(80),
    );
    p.rect_filled(
        r,
        16.,
        if pal.light {
            Color32::from_white_alpha(235)
        } else {
            Color32::from_rgba_unmultiplied(16, 18, 16, 230)
        },
    );
    p.rect_stroke(
        r,
        16.,
        Stroke::new(1_f32, BRASS.gamma_multiply(0.85)),
        egui::StrokeKind::Inside,
    );
    let mut x = r.left() + 16.;
    p.galley(
        Pos2::new(x, r.center().y - g.size().y / 2.),
        g.clone(),
        pal.text,
    );
    x += g.size().x + 10.;
    ui::keycap(p, Pos2::new(x + kw / 2., r.center().y), key, pal);
    x += kw + 6.;
    p.galley(Pos2::new(x, r.center().y - t.size().y / 2.), t, pal.muted);
}

fn banner(p: &egui::Painter, board: Rect, head: &str, sub: &str, age: f32, pal: &Palette) {
    let fade = if age < 0.12 {
        age / 0.12
    } else {
        (1. - (age - 0.6) / 0.4).clamp(0., 1.)
    };
    let band = Rect::from_center_size(board.center(), Vec2::new(board.width(), 92.));
    p.rect_filled(band, 0., Color32::from_black_alpha((150. * fade) as u8));
    for y in [band.top(), band.bottom()] {
        p.line_segment(
            [Pos2::new(band.left(), y), Pos2::new(band.right(), y)],
            Stroke::new(1_f32, BRASS.gamma_multiply(fade)),
        );
    }
    let rise = (1. - fade) * 8.;
    p.text(
        band.center() - Vec2::new(0., 12. + rise),
        Align2::CENTER_CENTER,
        head,
        ui::mono(34.),
        IVORY.gamma_multiply(fade),
    );
    p.text(
        band.center() + Vec2::new(0., 24. - rise),
        Align2::CENTER_CENTER,
        sub,
        ui::text(14.),
        BRASS.lerp_to_gamma(IVORY, 0.3).gamma_multiply(fade),
    );
    let _ = pal;
}

fn toast(p: &egui::Painter, at: Pos2, text: &str, pal: &Palette) {
    let g = p.layout_no_wrap(text.to_owned(), ui::text(13.), pal.text);
    let r = Rect::from_center_size(at, g.size() + Vec2::new(28., 14.));
    p.rect_filled(
        r.translate(Vec2::new(0., 3.)),
        r.height() / 2.,
        Color32::from_black_alpha(70),
    );
    p.rect_filled(
        r,
        r.height() / 2.,
        if pal.light {
            Color32::from_white_alpha(238)
        } else {
            Color32::from_rgba_unmultiplied(16, 18, 16, 235)
        },
    );
    p.rect_stroke(
        r,
        r.height() / 2.,
        Stroke::new(1_f32, pal.line),
        egui::StrokeKind::Inside,
    );
    p.galley(r.center() - g.size() / 2., g, pal.text);
}

fn composition(wave: &Wave) -> String {
    wave.groups
        .iter()
        .map(|g| {
            format!(
                "{} {}{}",
                g.count,
                g.kind.name(),
                if g.count == 1 { "" } else { "s" }
            )
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.draw(ctx);
    }
    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        self.suspend();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::Replay;

    fn frame(app: &mut App, ctx: &egui::Context, events: Vec<egui::Event>) -> egui::FullOutput {
        frame_sized(app, ctx, events, Vec2::new(1280., 900.))
    }
    fn frame_sized(
        app: &mut App,
        ctx: &egui::Context,
        events: Vec<egui::Event>,
        size: Vec2,
    ) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
                focused: true,
                events,
                ..Default::default()
            },
            |ctx| app.draw(ctx),
        )
    }
    fn key(key: Key) -> Vec<egui::Event> {
        [true, false]
            .map(|pressed| egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            })
            .to_vec()
    }
    fn press(app: &mut App, ctx: &egui::Context, k: Key) {
        frame(app, ctx, key(k));
    }
    fn find(output: &egui::FullOutput, label: &str) -> Option<Pos2> {
        output.shapes.iter().rev().find_map(|s| match &s.shape {
            egui::Shape::Text(t) if t.galley.job.text == label => {
                Some(t.pos + t.galley.size() / 2.)
            }
            _ => None,
        })
    }
    fn click_at(app: &mut App, ctx: &egui::Context, pos: Pos2, button: egui::PointerButton) {
        frame(app, ctx, vec![egui::Event::PointerMoved(pos)]);
        for pressed in [true, false] {
            frame(
                app,
                ctx,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
    }
    fn click(app: &mut App, ctx: &egui::Context, label: &str) {
        frame(app, ctx, vec![]);
        let out = frame(app, ctx, vec![]);
        let pos = find(&out, label).unwrap_or_else(|| panic!("missing visible action: {label}"));
        click_at(app, ctx, pos, egui::PointerButton::Primary);
    }
    fn view(app: &mut App, ctx: &egui::Context) -> View {
        frame(app, ctx, vec![]);
        app.view.unwrap()
    }
    fn app() -> (tempfile::TempDir, App, egui::Context) {
        let dir = tempfile::tempdir().unwrap();
        let mut app = App::from_path(dir.path().join("ridgeline.json"));
        app.save.preferences.sound = false;
        (dir, app, egui::Context::default())
    }
    fn battle(app: &App) -> &Battle {
        app.save.battle.as_ref().unwrap()
    }

    #[test]
    fn mouse_builds_inspects_upgrades_sells_and_returns() {
        let (_d, mut app, ctx) = app();
        assert_eq!(app.panel, Panel::Maps);
        click(&mut app, &ctx, "Normal");
        assert_eq!(app.panel, Panel::None);
        assert_eq!(battle(&app).map, 0);
        let credits = battle(&app).credits;
        click(&mut app, &ctx, "Cannon");
        assert_eq!(app.pending, Some(TowerKind::Cannon));
        let v = view(&mut app, &ctx);
        // A click on the shop never reaches the board: nothing was built.
        assert!(battle(&app).towers.is_empty());
        click_at(
            &mut app,
            &ctx,
            v.cell_rect(5, 1).center(),
            egui::PointerButton::Primary,
        );
        assert_eq!(battle(&app).towers.len(), 1, "cursor {:?}", app.cursor);
        assert_eq!(battle(&app).towers[0].x, 5);
        assert_eq!(battle(&app).credits, credits - 50);
        assert_eq!(app.pending, None);
        // Road clicks are rejected without spending.
        click(&mut app, &ctx, "Cannon");
        click_at(
            &mut app,
            &ctx,
            v.cell_rect(0, 2).center(),
            egui::PointerButton::Primary,
        );
        assert_eq!(battle(&app).towers.len(), 1);
        assert_eq!(app.pending, Some(TowerKind::Cannon));
        // Right-click cancels placement and spends nothing.
        click_at(
            &mut app,
            &ctx,
            v.cell_rect(3, 3).center(),
            egui::PointerButton::Secondary,
        );
        assert_eq!(app.pending, None);
        assert_eq!(battle(&app).credits, credits - 50);
        click_at(
            &mut app,
            &ctx,
            v.cell_rect(5, 1).center(),
            egui::PointerButton::Primary,
        );
        assert_eq!(app.selected, Some(battle(&app).towers[0].id));
        click(&mut app, &ctx, "Upgrade · 45");
        assert_eq!(battle(&app).towers[0].tier, 1);
        click(&mut app, &ctx, "Sell · 66");
        assert!(battle(&app).towers.is_empty());
        assert_eq!(battle(&app).credits, credits - 95 + 66);
        click(&mut app, &ctx, "Start wave 1");
        assert_eq!(battle(&app).phase, Phase::Running);
        click(&mut app, &ctx, "Pause");
        assert!(app.paused);
        // Building remains available during a tactical pause.
        click(&mut app, &ctx, "Flak");
        click_at(
            &mut app,
            &ctx,
            v.cell_rect(2, 1).center(),
            egui::PointerButton::Primary,
        );
        assert_eq!(battle(&app).towers.len(), 1);
        let tick = battle(&app).tick;
        for _ in 0..5 {
            frame(&mut app, &ctx, vec![]);
        }
        assert_eq!(battle(&app).tick, tick, "paused time never advances");
        click(&mut app, &ctx, "Back to Arcade");
        assert!(app.finished());
        let saved = storage::load(&app.path).unwrap();
        assert_eq!(saved.battle.as_ref(), app.save.battle.as_ref());
    }

    #[test]
    fn keyboard_places_inspects_cancels_pauses_and_starts() {
        let (_d, mut app, ctx) = app();
        app.choose(0, Difficulty::Normal, false);
        press(&mut app, &ctx, Key::Num1);
        assert_eq!(app.pending, Some(TowerKind::Cannon));
        // Escape cancels placement before it can pause anything.
        press(&mut app, &ctx, Key::Escape);
        assert_eq!(app.pending, None);
        press(&mut app, &ctx, Key::Num2);
        app.cursor = (0, 0);
        // A pointer resting on the board never overrides the keyboard cursor.
        let centre = view(&mut app, &ctx).rect.center();
        app.last_pointer = None;
        frame(&mut app, &ctx, vec![egui::Event::PointerMoved(centre)]);
        frame(&mut app, &ctx, vec![egui::Event::PointerMoved(centre)]);
        assert_eq!(app.cursor, (0, 0));
        for k in [
            Key::ArrowRight,
            Key::ArrowRight,
            Key::ArrowRight,
            Key::ArrowDown,
            Key::ArrowDown,
            Key::ArrowDown,
            Key::ArrowDown,
        ] {
            press(&mut app, &ctx, k);
        }
        assert_eq!(app.cursor, (3, 4));
        press(&mut app, &ctx, Key::Enter);
        assert_eq!(battle(&app).towers.len(), 1);
        assert_eq!(battle(&app).towers[0].kind, TowerKind::Mortar);
        app.selected = None;
        press(&mut app, &ctx, Key::Enter);
        assert_eq!(app.selected, Some(battle(&app).towers[0].id));
        press(&mut app, &ctx, Key::U);
        assert_eq!(battle(&app).towers[0].tier, 1);
        press(&mut app, &ctx, Key::F);
        assert_eq!(app.save.preferences.speed, 2);
        press(&mut app, &ctx, Key::Space);
        assert_eq!(battle(&app).phase, Phase::Running);
        press(&mut app, &ctx, Key::Escape);
        assert!(app.paused);
        // Space is not a resume or second start while running.
        press(&mut app, &ctx, Key::Space);
        assert!(app.paused);
        press(&mut app, &ctx, Key::Escape);
        assert!(!app.paused);
    }

    #[test]
    fn focus_loss_and_host_gate_pause_and_save_then_reopen_paused() {
        let (dir, mut app, ctx) = app();
        app.choose(0, Difficulty::Normal, false);
        app.command(Command::Build {
            x: 5,
            y: 1,
            kind: TowerKind::Cannon,
        })
        .unwrap();
        app.command(Command::StartWave).unwrap();
        app.last = Instant::now() - Duration::from_millis(100);
        frame(&mut app, &ctx, vec![]);
        assert!(battle(&app).tick > 0);
        let _ = ctx.run(
            egui::RawInput {
                focused: false,
                events: vec![egui::Event::WindowFocused(false)],
                ..Default::default()
            },
            |ctx| app.draw(ctx),
        );
        assert!(app.paused);
        let saved = storage::load(&dir.path().join("ridgeline.json")).unwrap();
        assert_eq!(saved.battle.as_ref(), app.save.battle.as_ref());
        app.resume();
        app.set_input_enabled(false);
        assert!(app.paused);
        // Reopening restores the same combat state, paused.
        let reopened = App::from_path(dir.path().join("ridgeline.json"));
        assert!(reopened.paused);
        assert_eq!(reopened.save.battle, app.save.battle);
        assert_eq!(reopened.panel, Panel::None);
        // A rendering stall pauses instead of skipping simulation.
        let (_d2, mut stalled, ctx2) = self::app();
        stalled.choose(0, Difficulty::Normal, false);
        stalled.command(Command::StartWave).unwrap();
        stalled.last = Instant::now() - Duration::from_secs(2);
        frame(&mut stalled, &ctx2, vec![]);
        assert!(stalled.paused);
        assert!(stalled.pause_reason.as_ref().unwrap().contains("stalled"));
    }

    #[test]
    fn replacing_an_unfinished_defence_requires_confirmation() {
        let (_d, mut app, ctx) = app();
        app.save.progress.records[0][0] = storage::Record {
            medal: storage::Medal::Clear,
            best_health: 10,
            wins: 1,
        };
        app.choose(0, Difficulty::Normal, false);
        app.command(Command::Build {
            x: 5,
            y: 1,
            kind: TowerKind::Cannon,
        })
        .unwrap();
        let before = app.save.battle.clone();
        click(&mut app, &ctx, "Maps");
        click(&mut app, &ctx, "Hard");
        assert!(matches!(app.panel, Panel::Replace { .. }));
        click(&mut app, &ctx, "Keep current defence");
        assert_eq!(app.save.battle, before);
        click(&mut app, &ctx, "Hard");
        click(&mut app, &ctx, "Replace defence");
        assert_eq!(battle(&app).difficulty, Difficulty::Hard);
        assert!(battle(&app).towers.is_empty());
    }

    #[test]
    fn replay_through_app_awards_medal_and_unlocks_once() {
        let (dir, mut app, ctx) = app();
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/replays/01-normal.json"
        ))
        .unwrap();
        let replay: Replay = serde_json::from_str(&text).unwrap();
        app.choose(0, Difficulty::Normal, false);
        for (tick, c) in &replay.commands {
            while battle(&app).tick < *tick && battle(&app).phase == Phase::Running {
                app.save.battle.as_mut().unwrap().step();
            }
            app.command(*c).unwrap();
            if battle(&app).phase == Phase::Running {
                // Let the frontend observe the final wave's result itself.
                while battle(&app).phase == Phase::Running
                    && *c == Command::StartWave
                    && battle(&app).wave == WAVES - 1
                {
                    app.last = Instant::now() - Duration::from_millis(200);
                    frame(&mut app, &ctx, vec![]);
                }
            }
        }
        assert_eq!(battle(&app).phase, Phase::Victory);
        assert!(matches!(app.panel, Panel::Result(Some(_))));
        assert!(app.save.progress.unlocked(1, Difficulty::Normal));
        assert!(app.save.progress.unlocked(0, Difficulty::Hard));
        let wins = app.save.progress.record(0, Difficulty::Normal).wins;
        assert_eq!(wins, 1);
        frame(&mut app, &ctx, vec![]);
        app.suspend();
        let reopened = App::from_path(dir.path().join("ridgeline.json"));
        assert_eq!(reopened.save.progress.record(0, Difficulty::Normal).wins, 1);
        assert_eq!(reopened.panel, Panel::Result(None));
        click(&mut app, &ctx, "Next: Dry Wash");
        assert_eq!(battle(&app).map, 1);
    }

    #[test]
    fn unrecorded_saved_victory_is_awarded_once_on_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ridgeline.json");
        let mut b = Battle::new(0, Difficulty::Normal);
        b.phase = Phase::Victory;
        b.health = 17;
        let save = Save {
            battle: Some(b),
            ..Save::default()
        };
        storage::write(&path, &save).unwrap();
        let app = App::from_path(path.clone());
        assert!(matches!(app.panel, Panel::Result(Some(_))));
        assert!(app.save.progress.unlocked(1, Difficulty::Normal));
        let again = App::from_path(path);
        assert_eq!(again.panel, Panel::Result(None));
        assert_eq!(again.save.progress.record(0, Difficulty::Normal).wins, 1);
    }

    #[test]
    fn rejected_save_is_never_written_until_archived() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ridgeline.json");
        std::fs::write(&path, b"{\"version\": 99}").unwrap();
        let mut app = App::from_path(path.clone());
        let ctx = egui::Context::default();
        assert!(app.blocked);
        assert_eq!(app.panel, Panel::Recovery);
        app.choose(0, Difficulty::Normal, false);
        app.suspend();
        frame(&mut app, &ctx, key(Key::Space));
        assert_eq!(std::fs::read(&path).unwrap(), b"{\"version\": 99}");
        click(&mut app, &ctx, "Archive and start fresh");
        assert!(!app.blocked);
        assert_eq!(app.panel, Panel::Maps);
        assert!(storage::load(&path).is_ok());
        let archived = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .find(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("ridgeline-recovery-")
            })
            .unwrap();
        assert_eq!(
            std::fs::read(archived.path()).unwrap(),
            b"{\"version\": 99}"
        );
    }

    #[test]
    fn compact_layout_renders_every_panel() {
        let (_d, mut app, ctx) = app();
        app.choose(0, Difficulty::Normal, false);
        for panel in [
            Panel::None,
            Panel::Maps,
            Panel::Settings,
            Panel::Help,
            Panel::Restart,
        ] {
            app.panel = panel;
            let out = frame_sized(&mut app, &ctx, vec![], Vec2::new(900., 760.));
            assert!(!out.shapes.is_empty());
        }
    }
}
