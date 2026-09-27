//! Native Ridgeline frontend: input, panels, rendering and lifecycle.
use crate::{
    art::{self, Palette, View},
    audio::{Audio, Cue},
    battle::{Battle, Clock, Command, Event, Phase, Reject, BASE_HEALTH},
    data::*,
    storage::{self, Award, Save},
};
use arcade_presentation::BRASS;
use eframe::egui::{self, Align2, Color32, FontId, Key, Pos2, Rect, RichText, Sense, Stroke, Vec2};
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
struct Effect {
    at: P,
    radius: i32,
    colour: Color32,
    born: Instant,
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
                match event {
                    Event::Hit { at, splash, kind } if !reduced => self.effects.push(Effect {
                        at,
                        radius: splash.max(180),
                        colour: if kind == TowerKind::Cryo {
                            Color32::from_rgb(130, 200, 240)
                        } else {
                            BRASS
                        },
                        born: now,
                    }),
                    Event::Kill { at, .. } if !reduced => self.effects.push(Effect {
                        at,
                        radius: 420,
                        colour: arcade_presentation::IVORY,
                        born: now,
                    }),
                    Event::Leak { at, damage } => {
                        self.effects.push(Effect {
                            at,
                            radius: 700,
                            colour: Color32::from_rgb(235, 110, 90),
                            born: now,
                        });
                        self.notice = Some((format!("Breach · base −{damage}"), now));
                        cues.push(Cue::Leak);
                    }
                    Event::WaveCleared { wave, award } => {
                        self.notice =
                            Some((format!("Wave {} held · +{award} credits", wave + 1), now));
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

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("RIDGELINE").strong().monospace().color(BRASS));
            let battle = self.save.battle.clone();
            if let Some(b) = &battle {
                ui.label(format!("{} · {}", b.data().name, b.difficulty.name()));
                let running = b.phase == Phase::Running;
                if ui
                    .add_enabled(
                        running && self.panel == Panel::None,
                        egui::Button::new(if self.paused { "Resume" } else { "Pause" }),
                    )
                    .on_hover_text("Escape or P. Building stays available while paused.")
                    .clicked()
                {
                    self.toggle_pause();
                }
                if ui
                    .add_enabled(
                        b.phase == Phase::Waiting && !self.blocked && self.panel == Panel::None,
                        egui::Button::new(format!("Start wave {}", b.wave + 1)),
                    )
                    .on_hover_text("Space")
                    .clicked()
                {
                    self.start_wave();
                }
            }
            if ui
                .button(format!("Speed {}×", self.save.preferences.speed))
                .on_hover_text("F toggles 1× / 2×. 2× runs twice as many ordinary ticks.")
                .clicked()
            {
                self.toggle_speed();
            }
            if ui
                .add_enabled(!self.blocked, egui::Button::new("Maps"))
                .clicked()
            {
                self.open(Panel::Maps);
            }
            if ui
                .add_enabled(
                    !self.blocked && battle.is_some(),
                    egui::Button::new("Restart"),
                )
                .clicked()
            {
                self.open(Panel::Restart);
            }
            if ui.button("Settings").on_hover_text("Ctrl+,").clicked() {
                self.open(Panel::Settings);
            }
            if ui.button("Help").on_hover_text("F1").clicked() {
                self.open(Panel::Help);
            }
            if ui
                .button("Back to Arcade")
                .on_hover_text("Ctrl+H")
                .clicked()
            {
                self.suspend();
                self.leave = true;
            }
        });
        if let Some(e) = &self.error {
            ui.label(RichText::new(e).color(Color32::from_rgb(235, 110, 90)));
        }
        if let Some(e) = self.write_error.clone() {
            ui.horizontal_wrapped(|ui| {
                ui.label(e);
                if ui.button("Retry save").clicked() {
                    self.persist();
                }
            });
        }
    }

    fn sidebar(&mut self, ui: &mut egui::Ui, pal: &Palette) {
        let Some(b) = self.save.battle.clone() else {
            ui.label("Choose a map to begin.");
            return;
        };
        let interactive = !self.blocked && !b.finished() && self.panel == Panel::None;
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Wave {}/{}", b.wave + 1, WAVES)).strong());
            ui.separator();
            ui.label(RichText::new(format!("Credits {}", b.credits)).strong());
        });
        let health = b.health as f32 / BASE_HEALTH as f32;
        ui.add(
            egui::ProgressBar::new(health)
                .text(format!("Base {}/{}", b.health, BASE_HEALTH))
                .fill(if health > 0.35 { pal.good } else { pal.bad }),
        );
        let status = match b.phase {
            Phase::Waiting => "Waiting · build, then start the wave".to_string(),
            Phase::Running if self.paused => "Paused · you can still build".into(),
            Phase::Running => format!("Wave running · {}×", self.save.preferences.speed),
            Phase::Victory => "Victory".into(),
            Phase::Defeat => "Defeat".into(),
        };
        ui.label(status);
        ui.separator();
        ui.label(RichText::new("Build").strong());
        for (i, kind) in TowerKind::ALL.into_iter().enumerate() {
            let stats = tower(kind, 0);
            let chosen = self.pending == Some(kind);
            let affordable = stats.cost <= b.credits;
            let (rect, _) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), 40.), Sense::hover());
            let button = egui::Button::new(format!(
                "{}  {}  ·  {} cr  ·  {}",
                i + 1,
                kind.name(),
                stats.cost,
                kind.targets().name()
            ))
            .selected(chosen)
            .min_size(Vec2::new(rect.width() - 44., 36.));
            let icon = Rect::from_min_size(rect.min, Vec2::splat(40.));
            let at = icon.center() - Vec2::new(0., if affordable { 0. } else { 5. });
            art::tower_icon(
                ui.painter(),
                at,
                if affordable { 32. } else { 24. },
                kind,
                0,
                pal,
                Vec2::new(0., -1.),
            );
            let response = ui
                .put(
                    Rect::from_min_max(rect.min + Vec2::new(44., 2.), rect.max),
                    button,
                )
                .on_hover_text(format!(
                    "{} · damage {} · range {:.1} · reload {:.2}s{}{}",
                    kind.role(),
                    stats.damage,
                    stats.range as f32 / CELL as f32,
                    stats.reload as f32 / 60.,
                    if stats.min_range > 0 {
                        format!(
                            " · minimum range {:.1}",
                            stats.min_range as f32 / CELL as f32
                        )
                    } else {
                        String::new()
                    },
                    if stats.slow > 0 {
                        format!(" · slows {}%", stats.slow)
                    } else {
                        String::new()
                    },
                ));
            if response.clicked() && interactive {
                self.pending = if chosen { None } else { Some(kind) };
                self.selected = None;
            }
            if !affordable {
                // A word under the icon, so affordability never relies on colour alone.
                ui.painter().text(
                    icon.center_bottom() + Vec2::new(0., 1.),
                    Align2::CENTER_BOTTOM,
                    "short",
                    FontId::proportional(10.),
                    pal.bad,
                );
            }
        }
        ui.separator();
        if let Some(kind) = self.pending {
            let (x, y) = self.cursor;
            ui.label(RichText::new(format!("Placing {}", kind.name())).strong());
            ui.label(stats_text(&tower(kind, 0)));
            match b.check_build(x, y, kind) {
                Ok(cost) => ui.label(format!("Cell {x},{y}: valid · costs {cost}")),
                Err(e) => ui.label(RichText::new(format!("Cell {x},{y}: {e}")).color(pal.bad)),
            };
            ui.label("Click or Enter to place · right-click or Esc cancels");
            if ui.button("Cancel placement").clicked() {
                self.pending = None;
            }
        } else if let Some(t) = self.selected.and_then(|id| b.tower(id)).cloned() {
            ui.label(RichText::new(format!("{} · tier {}", t.kind.name(), t.tier + 1)).strong());
            ui.label(format!(
                "{} · targets {}",
                t.kind.role(),
                t.kind.targets().name()
            ));
            let now = t.stats();
            ui.label(stats_text(&now));
            let reload = t.reload as f32 / 60.;
            ui.label(format!("Next shot ready in {reload:.2}s"));
            match b.check_upgrade(t.id) {
                Err(Reject::MaxTier) => {
                    ui.label("Fully upgraded");
                }
                Err(Reject::Finished | Reject::NoTower) => {}
                check => {
                    let next = tower(t.kind, t.tier + 1);
                    ui.label(
                        RichText::new(format!("Upgrade to tier {} for {}:", t.tier + 2, next.cost))
                            .strong(),
                    );
                    for line in changes(&now, &next) {
                        ui.label(line);
                    }
                    if let Err(e) = check {
                        ui.label(RichText::new(e.to_string()).color(pal.bad));
                    }
                    if ui
                        .add_enabled(
                            interactive && check.is_ok(),
                            egui::Button::new(format!("Upgrade ({}) · U", next.cost)),
                        )
                        .clicked()
                    {
                        let _ = self.command(Command::Upgrade { tower: t.id });
                    }
                }
            }
            let refund = refund(t.spent);
            if ui
                .add_enabled(interactive, egui::Button::new(format!("Sell for {refund}")))
                .on_hover_text(
                    "Refunds 70% of credits spent, rounded down. Shots in flight still land.",
                )
                .clicked()
            {
                let _ = self.command(Command::Sell { tower: t.id });
            }
            if ui.button("Deselect").clicked() {
                self.selected = None;
            }
        } else {
            ui.label("Select a tower with 1–4, or click a tower to inspect it.");
        }
        ui.separator();
        let wave = b.preview();
        ui.label(
            RichText::new(if b.phase == Phase::Waiting {
                format!("Next: wave {}", b.wave + 1)
            } else {
                format!("This wave: {}", b.wave + 1)
            })
            .strong(),
        );
        for g in &wave.groups {
            let stats = g.kind.stats();
            let hp = (stats.hp * wave.health / 100).max(1);
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(Vec2::splat(22.), Sense::hover());
                art::enemy_icon(ui.painter(), r.center(), 9., g.kind, pal);
                ui.label(format!(
                    "{} × {} · {} hp{} · {} dmg{}",
                    g.count,
                    g.kind.name(),
                    hp,
                    if stats.armour > 0 {
                        format!(" · armour {}", stats.armour)
                    } else {
                        String::new()
                    },
                    stats.damage,
                    if stats.air { " · AIR" } else { "" },
                ))
                .on_hover_text(g.kind.role());
            });
        }
    }

    fn board(&mut self, ui: &mut egui::Ui, pal: &Palette) {
        arcade_presentation::backdrop(ui);
        let Some(b) = self.save.battle.clone() else {
            return;
        };
        let view = View::fit(ui.available_rect_before_wrap().shrink(18.));
        self.view = Some(view);
        let response = ui.allocate_rect(view.rect, Sense::click());
        let interactive =
            !self.blocked && !b.finished() && self.panel == Panel::None && self.enabled;
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
        let p = ui.painter_at(view.rect.expand(14.));
        arcade_presentation::bezel(&p, view.rect, pal.accent);
        art::terrain(&p, view, b.data(), b.map, pal);
        for t in &b.towers {
            art::tower(&p, view, &b, t, pal);
        }
        let mut enemies: Vec<_> = b.enemies.iter().collect();
        // Aircraft draw above ground units.
        enemies.sort_by_key(|e| e.kind.stats().air);
        for e in enemies {
            art::enemy(&p, view, &b, e, pal);
        }
        for s in &b.shots {
            art::shot(&p, view, s, pal);
        }
        let now = Instant::now();
        if !self.save.preferences.reduced_effects {
            for e in &self.effects {
                let age = now.duration_since(e.born).as_secs_f32() / 0.35;
                if age < 1. {
                    p.circle_stroke(
                        view.pt(e.at),
                        view.len(e.radius) * (0.4 + age * 0.6),
                        Stroke::new(2.0_f32, e.colour.gamma_multiply(1. - age)),
                    );
                }
            }
        } else {
            for e in &self.effects {
                if now.duration_since(e.born) < Duration::from_millis(500) && e.radius >= 700 {
                    p.circle_stroke(
                        view.pt(e.at),
                        view.len(e.radius),
                        Stroke::new(2.0_f32, e.colour),
                    );
                }
            }
        }
        self.effects
            .retain(|e| now.duration_since(e.born) < Duration::from_millis(600));
        // Overlays: selected tower range, pending placement preview and cursor.
        if let Some(t) = self.selected.and_then(|id| b.tower(id)) {
            art::range(&p, view, t.pos(), &t.stats(), BRASS);
            p.rect_stroke(
                view.cell_rect(t.x, t.y),
                2.,
                Stroke::new(2.0_f32, BRASS),
                egui::StrokeKind::Inside,
            );
        }
        let (cx, cy) = self.cursor;
        let cell = view.cell_rect(cx, cy);
        if let Some(kind) = self.pending {
            let check = b.check_build(cx, cy, kind);
            let colour = if check.is_ok() { pal.good } else { pal.bad };
            art::range(&p, view, P::cell(cx, cy), &tower(kind, 0), colour);
            art::tower_icon(
                &p,
                cell.center(),
                view.cell * 0.9,
                kind,
                0,
                pal,
                Vec2::new(0., -1.),
            );
            p.rect_stroke(
                cell,
                2.,
                Stroke::new(2.5_f32, colour),
                egui::StrokeKind::Inside,
            );
            // Validity is also a symbol, not colour alone.
            p.text(
                cell.right_top() + Vec2::new(-2., 2.),
                Align2::RIGHT_TOP,
                if check.is_ok() { "+" } else { "×" },
                FontId::monospace(view.cell * 0.45),
                colour,
            );
        } else if interactive {
            p.rect_stroke(
                cell,
                2.,
                Stroke::new(1.5_f32, pal.text.gamma_multiply(0.8)),
                egui::StrokeKind::Inside,
            );
        }
        let banner = match (b.phase, self.paused) {
            (Phase::Waiting, _) if interactive => {
                Some(format!("Wave {} ready · Space or Start wave", b.wave + 1))
            }
            (Phase::Running, true) => Some(
                self.pause_reason
                    .clone()
                    .unwrap_or_else(|| "Paused · Escape to resume · building allowed".into()),
            ),
            _ => None,
        };
        if let Some(text) = banner {
            text_box(&p, view.rect.center_top() + Vec2::new(0., 18.), &text, pal);
        }
        if let Some((text, at)) = &self.notice {
            if at.elapsed() < Duration::from_secs(3) {
                text_box(
                    &p,
                    view.rect.center_bottom() - Vec2::new(0., 22.),
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
                    Color32::from_white_alpha(70)
                } else {
                    Color32::from_black_alpha(70)
                },
            );
        }
    }

    fn panels(&mut self, ctx: &egui::Context) {
        let title = match self.panel {
            Panel::None => return,
            Panel::Maps => "Ridgeline · Choose a map",
            Panel::Settings => "Settings",
            Panel::Help => "How to play Ridgeline",
            Panel::Restart => "Restart this map?",
            Panel::Replace { .. } => "Replace unfinished defence?",
            Panel::Result(_) => match self.save.battle.as_ref().map(|b| b.phase) {
                Some(Phase::Victory) => "Victory",
                _ => "Defeat",
            },
            Panel::Recovery => "Saved campaign protected",
        };
        egui::Window::new(title)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .min_width(340.)
            .max_width(520.)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .max_height((ctx.screen_rect().height() - 180.).max(200.))
                    .show(ui, |ui| self.panel_body(ui));
            });
    }

    fn panel_body(&mut self, ui: &mut egui::Ui) {
        match self.panel {
            Panel::None => {}
            Panel::Maps => {
                ui.label("Normal wins unlock the next map and that map's Hard table. Records are kept separately for each difficulty.");
                ui.add_space(4.);
                egui::Grid::new("ridgeline-maps")
                    .striped(true)
                    .num_columns(4)
                    .show(ui, |ui| {
                        for heading in ["Map", "Play Normal", "Play Hard", "Medals N / H"] {
                            ui.label(RichText::new(heading).small().strong());
                        }
                        ui.end_row();
                        for (i, map) in maps().iter().enumerate() {
                            ui.label(RichText::new(format!("{:02}  {}", i + 1, map.name)).strong())
                                .on_hover_text(map.lesson);
                            for d in Difficulty::ALL {
                                let unlocked = self.save.progress.unlocked(i, d);
                                let r = self.save.progress.record(i, d);
                                let label = if unlocked {
                                    d.name().to_string()
                                } else {
                                    format!("{} · locked", d.name())
                                };
                                let hint = if r.wins > 0 {
                                    format!(
                                        "{} · best base {}/{} · {} wins",
                                        r.medal.name(),
                                        r.best_health,
                                        BASE_HEALTH,
                                        r.wins
                                    )
                                } else if unlocked {
                                    "Not yet won".into()
                                } else if d == Difficulty::Hard {
                                    "Win this map on Normal to unlock Hard".into()
                                } else {
                                    "Win the previous map on Normal to unlock".into()
                                };
                                if ui
                                    .add_enabled(unlocked, egui::Button::new(label))
                                    .on_hover_text(hint)
                                    .on_disabled_hover_text("Locked")
                                    .clicked()
                                {
                                    self.choose(i, d, false);
                                }
                            }
                            let medals = Difficulty::ALL
                                .map(|d| self.save.progress.record(i, d).medal.name())
                                .join(" / ");
                            ui.label(medals);
                            ui.end_row();
                        }
                    });
                if let Some(b) = self.save.battle.as_ref().filter(|b| !b.finished()) {
                    ui.add_space(6.);
                    if ui
                        .button(format!("Continue {} · wave {}", b.data().name, b.wave + 1))
                        .clicked()
                    {
                        self.panel = Panel::None;
                    }
                }
                if ui.button("Back to Arcade").clicked() {
                    self.suspend();
                    self.leave = true;
                }
            }
            Panel::Settings => {
                let mut sound = self.save.preferences.sound;
                if ui.checkbox(&mut sound, "Sound · Ctrl+M").changed() {
                    self.toggle_sound();
                }
                ui.checkbox(
                    &mut self.save.preferences.reduced_effects,
                    "Reduced effects (no impact rings)",
                );
                ui.horizontal(|ui| {
                    ui.label("Speed");
                    for s in [1, 2] {
                        if ui
                            .selectable_label(self.save.preferences.speed == s, format!("{s}×"))
                            .clicked()
                        {
                            self.save.preferences.speed = s;
                        }
                    }
                });
                if ui.button("Done").clicked() {
                    self.persist();
                    self.close_panel();
                }
            }
            Panel::Help => {
                ui.label("Enemies follow fixed roads (aircraft fly marked lanes) to the brass gates. Each one that gets through costs base health; at zero the defence falls. Hold all 20 waves to win.");
                ui.label("Build on open terrace, never on roads or rock. Cannon: cheap direct damage. Mortar: slow splash that cannot fire at close range. Flak: the only tower that hits aircraft. Cryo: light damage that slows; the strongest slow applies.");
                ui.label("Armour subtracts from every hit, with at least 1 damage. Kills pay a bounty once; clearing a wave pays a fixed award. Selling refunds 70% of what you spent. Upgrades apply at once but keep the current reload.");
                ui.label("Mouse: pick a tower, then click open terrace. Click a tower to inspect, upgrade or sell. Right-click cancels.");
                ui.label("Keyboard: 1–4 choose a tower, arrows move the cursor, Enter places or inspects, U upgrades, Space starts the next wave, F toggles 2×, Escape cancels placement or pauses. Tab reaches every button.");
                ui.label("Ctrl+, Settings · Ctrl+M sound · F1 help · Ctrl+H Arcade · Ctrl+Q quit.");
                ui.separator();
                ui.label("Ridgeline is an original Omarchy Arcade game: maps, waves, artwork and synthesized sound. Canyon Defense and other tower-defence games are design references only; nothing is copied.");
                if ui.button("Close").clicked() {
                    self.close_panel();
                }
            }
            Panel::Restart => {
                ui.label("Restart this map from wave 1? Towers and credits are lost. Records and unlocks are kept.");
                ui.horizontal(|ui| {
                    if ui.button("Restart map").clicked() {
                        if let Some(b) = &self.save.battle {
                            let (map, d) = (b.map, b.difficulty);
                            self.choose(map, d, true);
                        }
                    }
                    if ui.button("Keep defending").clicked() {
                        self.close_panel();
                    }
                });
            }
            Panel::Replace { map, difficulty } => {
                if let Some(b) = &self.save.battle {
                    ui.label(format!(
                        "Your {} defence on {} (wave {}) is unfinished. Starting {} {} replaces it. Records and unlocks are kept.",
                        b.difficulty.name(),
                        b.data().name,
                        b.wave + 1,
                        maps()[map].name,
                        difficulty.name()
                    ));
                }
                ui.horizontal(|ui| {
                    if ui.button("Replace defence").clicked() {
                        self.choose(map, difficulty, true);
                    }
                    if ui.button("Keep current defence").clicked() {
                        self.panel = Panel::Maps;
                    }
                });
            }
            Panel::Result(award) => {
                let Some(b) = self.save.battle.clone() else {
                    return;
                };
                if b.phase == Phase::Victory {
                    ui.label(
                        RichText::new(format!(
                            "{} held on {}.",
                            b.data().name,
                            b.difficulty.name()
                        ))
                        .strong(),
                    );
                    ui.label(format!(
                        "Base {}/{} · {} destroyed · {} breached",
                        b.health, BASE_HEALTH, b.kills, b.leaked
                    ));
                    if let Some(a) = award {
                        ui.label(format!(
                            "Medal: {}{}",
                            a.medal.name(),
                            if a.medal == storage::Medal::Perfect {
                                " (no damage taken)"
                            } else {
                                ""
                            }
                        ));
                        if a.unlocked_next {
                            ui.label(format!("Unlocked: {}", maps()[b.map + 1].name));
                        }
                        if a.unlocked_hard {
                            ui.label(format!("Unlocked: {} on Hard", b.data().name));
                        }
                    }
                } else {
                    ui.label(
                        RichText::new(format!("The line broke on wave {}.", b.wave + 1)).strong(),
                    );
                    ui.label(format!("{} destroyed · {} breached", b.kills, b.leaked));
                }
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Play again").clicked() {
                        self.choose(b.map, b.difficulty, true);
                    }
                    let next = b.map + 1;
                    if b.phase == Phase::Victory
                        && next < maps().len()
                        && self.save.progress.unlocked(next, Difficulty::Normal)
                        && ui.button(format!("Next: {}", maps()[next].name)).clicked()
                    {
                        self.choose(next, Difficulty::Normal, true);
                    }
                    if ui.button("Maps").clicked() {
                        self.panel = Panel::Maps;
                    }
                });
            }
            Panel::Recovery => {
                if let Some(e) = &self.error {
                    ui.label(e);
                }
                ui.label("Nothing will be written while the original is protected. Archive it to a unique recovery file beside it and start a fresh campaign? The archive is never overwritten.");
                ui.horizontal(|ui| {
                    if ui.button("Archive and start fresh").clicked() {
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
                    if ui.button("Back to Arcade").clicked() {
                        self.suspend();
                        self.leave = true;
                    }
                });
            }
        }
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
        visuals.override_text_color = Some(self.theme.foreground);
        visuals.panel_fill = self.theme.background;
        visuals.window_fill = self.theme.background;
        visuals.selection.bg_fill = self.theme.accent;
        visuals.selection.stroke = Stroke::new(1.0_f32, self.theme.accent_text());
        ctx.set_visuals(visuals);
        arcade_presentation::apply(ctx);
        let focused = ctx.input(|i| i.focused);
        if !focused || !self.enabled {
            self.pause(Some("Paused because you left the game. Resume when ready."));
        }
        if self.enabled && focused {
            self.keyboard(ctx);
        }
        egui::TopBottomPanel::top("ridgeline-controls").show(ctx, |ui| self.toolbar(ui));
        egui::SidePanel::right("ridgeline-side")
            .resizable(false)
            .exact_width(if ctx.screen_rect().width() < 1000. {
                270.
            } else {
                310.
            })
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.sidebar(ui, &pal));
            });
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
        egui::CentralPanel::default().show(ctx, |ui| self.board(ui, &pal));
        self.panels(ctx);
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
        ctx.request_repaint_after(if active {
            Duration::from_millis(8)
        } else {
            Duration::from_millis(100)
        });
    }
}

fn stats_text(s: &TowerStats) -> String {
    let mut text = format!(
        "Damage {} · range {:.1} · reload {:.2}s",
        s.damage,
        s.range as f32 / CELL as f32,
        s.reload as f32 / 60.
    );
    if s.min_range > 0 {
        text += &format!(" · min range {:.1}", s.min_range as f32 / CELL as f32);
    }
    if s.splash > 0 {
        text += &format!(" · splash {:.1}", s.splash as f32 / CELL as f32);
    }
    if s.slow > 0 {
        text += &format!(" · slow {}% for {:.1}s", s.slow, s.slow_ticks as f32 / 60.);
    }
    text
}

/// Exact stat changes an upgrade would make.
fn changes(a: &TowerStats, b: &TowerStats) -> Vec<String> {
    let cells = |u: i32| format!("{:.2}", u as f32 / CELL as f32);
    let mut out = vec![];
    let mut add = |name: &str, x: String, y: String| {
        if x != y {
            out.push(format!("  {name} {x} → {y}"));
        }
    };
    add("Damage", a.damage.to_string(), b.damage.to_string());
    add("Range", cells(a.range), cells(b.range));
    add(
        "Reload",
        format!("{:.2}s", a.reload as f32 / 60.),
        format!("{:.2}s", b.reload as f32 / 60.),
    );
    add("Splash", cells(a.splash), cells(b.splash));
    add("Slow", format!("{}%", a.slow), format!("{}%", b.slow));
    add(
        "Slow time",
        format!("{:.1}s", a.slow_ticks as f32 / 60.),
        format!("{:.1}s", b.slow_ticks as f32 / 60.),
    );
    out
}

fn text_box(p: &egui::Painter, at: Pos2, text: &str, pal: &Palette) {
    let galley = p.layout_no_wrap(text.to_owned(), FontId::proportional(14.), pal.text);
    let rect = Rect::from_center_size(at, galley.size() + Vec2::new(18., 8.));
    p.rect_filled(
        rect,
        2.,
        if pal.light {
            Color32::from_white_alpha(225)
        } else {
            Color32::from_black_alpha(200)
        },
    );
    p.rect_stroke(
        rect,
        2.,
        Stroke::new(1.0_f32, BRASS.gamma_multiply(0.8)),
        egui::StrokeKind::Inside,
    );
    p.galley(rect.min + Vec2::new(9., 4.), galley, pal.text);
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
        click(&mut app, &ctx, "1  Cannon  ·  50 cr  ·  Ground");
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
        click(&mut app, &ctx, "1  Cannon  ·  50 cr  ·  Ground");
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
        click(&mut app, &ctx, "Upgrade (45) · U");
        assert_eq!(battle(&app).towers[0].tier, 1);
        click(&mut app, &ctx, "Sell for 66");
        assert!(battle(&app).towers.is_empty());
        assert_eq!(battle(&app).credits, credits - 95 + 66);
        click(&mut app, &ctx, "Start wave 1");
        assert_eq!(battle(&app).phase, Phase::Running);
        click(&mut app, &ctx, "Pause");
        assert!(app.paused);
        // Building remains available during a tactical pause.
        click(&mut app, &ctx, "3  Flak  ·  70 cr  ·  Air");
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
        stalled.last = Instant::now() - Duration::from_secs(1);
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
