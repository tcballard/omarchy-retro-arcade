use crate::{
    audio::{Audio, Cue},
    course,
    engine::{Mode, Phase, Point, DT},
    input::Controls,
    menu, render,
    session::Session,
    storage::{self, Save},
    world,
};
use eframe::egui::{self, Color32, Key, Rect, Sense, Vec2};
use omarchy_chess::theme::Theme;
use std::{
    collections::VecDeque,
    path::PathBuf,
    time::{Duration, Instant},
};

pub struct App {
    session: Session,
    path: PathBuf,
    audio: Audio,
    controls: Controls,
    theme: Theme,
    themed: Instant,
    last: Instant,
    accumulator: f64,
    error: Option<String>,
    writable: bool,
    recovery: bool,
    help: bool,
    settings: bool,
    restart: bool,
    pending_mode: Option<Mode>,
    leave: bool,
    pause_reason: String,
    tracks: VecDeque<(Point, Point)>,
    landing: Option<(Point, u64)>,
}
impl App {
    pub fn new() -> Result<Self, String> {
        Self::open(storage::path()?)
    }
    fn open(path: PathBuf) -> Result<Self, String> {
        let obstacles = world::practice();
        let (state, error, writable) = match storage::load(&path, &obstacles) {
            Ok(s) => (s, None, true),
            Err(e) => (Save::default(), Some(e), false),
        };
        let mut app = Self {
            session: Session::new(state),
            path,
            audio: Audio::default(),
            controls: Controls::default(),
            theme: Theme::load(),
            themed: Instant::now(),
            last: Instant::now(),
            accumulator: 0.,
            error,
            writable,
            recovery: !writable,
            help: false,
            settings: false,
            restart: false,
            pending_mode: None,
            leave: false,
            pause_reason: "Your run is saved. Resume when you are ready.".into(),
            tracks: VecDeque::new(),
            landing: None,
        };
        app.refresh_obstacles();
        app.flush();
        Ok(app)
    }
    fn flush(&mut self) {
        if self.writable {
            self.session.state.record_result();
            match storage::write(&self.path, &self.session.state, &self.session.obstacles) {
                Ok(()) => self.error = None,
                Err(e) => {
                    self.audio.stop();
                    self.error = Some(e);
                    self.session.state.run.pause();
                    self.controls.clear();
                    self.accumulator = 0.;
                    self.pause_reason =
                        "Saving failed. Your run is paused; retry saving before continuing.".into();
                }
            }
        }
    }
    pub fn suspend(&mut self) {
        self.pause("Your run is saved. Resume when you are ready.");
    }
    pub fn finished(&mut self) -> bool {
        self.leave
    }
    fn pause(&mut self, reason: &str) {
        self.audio.stop();
        self.session.state.run.pause();
        self.controls.clear();
        self.accumulator = 0.;
        self.last = Instant::now();
        self.pause_reason = reason.into();
        self.flush();
    }
    fn begin(&mut self) {
        self.session.state.run.start();
        self.controls.clear();
        self.accumulator = 0.;
        self.last = Instant::now();
        self.flush();
    }
    fn refresh_obstacles(&mut self) {
        self.session.refresh();
    }
    fn mountain_seed() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
    fn new_run(&mut self) {
        self.audio.stop();
        let mode = self.pending_mode.take().unwrap_or(self.session.state.mode);
        let seed = if mode == Mode::FreeSki {
            Self::mountain_seed()
        } else {
            0
        };
        let chase = self.session.state.chase_enabled && mode == Mode::FreeSki;
        self.session.state.select_mode(mode, seed);
        self.session.state.set_chase_enabled(chase);
        self.session.obstacles = self.session.state.obstacles();
        self.refresh_obstacles();
        self.restart = false;
        self.controls.clear();
        self.tracks.clear();
        self.landing = None;
        self.accumulator = 0.;
        self.flush();
    }
    fn play_event(&mut self, event: crate::engine::Event) {
        use crate::engine::Event;
        self.audio.play(match event {
            Event::Jump => Cue::Jump,
            Event::Crash => Cue::Crash,
            Event::Finish => Cue::Finish,
            Event::Gate => Cue::Gate,
            Event::Missed => Cue::Miss,
            Event::Warning | Event::Spawn => Cue::Warning,
            Event::Caught => Cue::Caught,
        });
    }
    fn blocked(&self) -> bool {
        self.help || self.settings || self.restart || self.recovery
    }
    pub fn show(&mut self, ctx: &egui::Context) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last).as_secs_f64();
        self.last = now;
        self.frame(ctx, elapsed);
    }
    fn frame(&mut self, ctx: &egui::Context, elapsed: f64) {
        if self.themed.elapsed() > Duration::from_secs(2) {
            self.theme = Theme::load();
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
        let focused = ctx.input(|i| i.focused);
        if !focused && self.session.state.run.phase == Phase::Running {
            self.pause("Focus changed. Your run is paused; resume when you are ready.");
        }
        let was_running = self.session.state.run.phase == Phase::Running && !self.blocked();
        let mut enter = false;
        let mut fast_key = false;
        ctx.input_mut(|i| {
            if focused && self.session.state.run.phase == Phase::Ready && !self.blocked() {
                for (key, mode) in [
                    (Key::P, Mode::Practice),
                    (Key::F, Mode::FreeSki),
                    (Key::L, Mode::Slalom),
                ] {
                    if i.consume_key(egui::Modifiers::NONE, key) && self.session.state.mode != mode
                    {
                        self.pending_mode = Some(mode);
                        self.new_run();
                    }
                }
                if self.session.state.mode == Mode::FreeSki
                    && i.consume_key(egui::Modifiers::NONE, Key::C)
                {
                    self.session
                        .state
                        .set_chase_enabled(!self.session.state.chase_enabled);
                    self.flush();
                }
                if self.session.state.mode == Mode::Slalom {
                    for (index, key) in [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5]
                        .into_iter()
                        .enumerate()
                    {
                        if i.consume_key(egui::Modifiers::NONE, key)
                            && self.session.state.select_course(index as u8)
                        {
                            self.refresh_obstacles();
                            self.flush();
                        }
                    }
                }
            }
            if i.consume_key(egui::Modifiers::NONE, Key::F1) {
                self.pause("Help is open. Resume when you are ready.");
                self.help = true;
            }
            if i.consume_key(egui::Modifiers::CTRL, Key::M) {
                self.session.state.muted = !self.session.state.muted;
                self.audio.stop();
                self.flush();
            }
            if i.consume_key(egui::Modifiers::CTRL, Key::Comma) {
                self.pause("Settings are open. Resume when you are ready.");
                self.settings = true;
            }
            if i.consume_key(egui::Modifiers::NONE, Key::Escape) {
                if self.help {
                    self.help = false;
                } else if self.settings {
                    self.settings = false;
                } else if self.restart {
                    self.restart = false;
                    self.pending_mode = None;
                } else if !self.recovery {
                    if self.session.state.run.phase == Phase::Running {
                        self.pause("Take a breath. Your run is saved.");
                    } else if self.session.state.run.phase == Phase::Paused && focused {
                        self.begin();
                    }
                }
            }
            if !self.help && !self.settings && !self.recovery {
                enter = i.consume_key(egui::Modifiers::NONE, Key::Enter);
            }
            if focused && self.session.state.run.phase == Phase::Running && !self.blocked() {
                let fast_press = i.events.iter().find_map(|event| match event {
                    egui::Event::Key {
                        key: Key::F,
                        pressed: true,
                        repeat,
                        modifiers,
                        ..
                    } => Some((*repeat, *modifiers)),
                    _ => None,
                });
                if let Some((repeat, modifiers)) = fast_press {
                    fast_key = !repeat && modifiers == egui::Modifiers::NONE;
                    // Consume repeats and modified presses too, so a focused UI
                    // control cannot reinterpret them as activation.
                    i.consume_key(modifiers, Key::F);
                }
            }
        });
        let mut field = Rect::NOTHING;
        let mut brake = false;
        let mut start = false;
        let mut pause = false;
        let mut restart = false;
        let mut help = false;
        let mut settings = false;
        let mut fast_button = false;
        let mut mode_choice = None;
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .inner_margin(24.)
                    .fill(self.theme.background),
            )
            .show(ctx, |ui| {
                arcade_presentation::backdrop(ui);
                egui::Frame::group(ui.style())
                    .fill(if self.theme.light() { self.theme.background } else { arcade_presentation::INK })
                    .stroke(ui.visuals().window_stroke)
                    .inner_margin(10)
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.spacing_mut().item_spacing = Vec2::new(10., 4.);
                        ui.horizontal_wrapped(|ui| {
                            ui.heading(egui::RichText::new("FREESKI").size(23.).monospace().color(if self.theme.light() { self.theme.foreground } else { arcade_presentation::IVORY }));
                            ui.separator();
                            if self.session.state.run.phase == Phase::Ready && !self.blocked() {
                                for (mode, label) in [(Mode::Practice, "Practice"), (Mode::FreeSki, "Free Ski"), (Mode::Slalom, "Slalom")] {
                                    if menu::choice(ui, label, self.session.state.mode == mode).clicked() && self.session.state.mode != mode {
                                        mode_choice = Some(mode);
                                    }
                                }
                            } else {
                                ui.label(match self.session.state.mode { Mode::Practice => "PRACTICE", Mode::FreeSki => "FREE SKI", Mode::Slalom => "SLALOM CUP" });
                            }
                            ui.separator();
                            ui.add_enabled_ui(!self.blocked() && focused, |ui| {
                                match self.session.state.run.phase {
                                    Phase::Ready => start = ui.add(menu::button("Start skiing  Enter", true)).clicked(),
                                    Phase::Running => pause = ui.button("Pause  Esc").clicked(),
                                    Phase::Paused => {}
                                    _ => restart = ui.button("New run").clicked(),
                                }
                                if self.session.state.run.phase == Phase::Running {
                                    fast_button = ui
                                        .add(menu::button(
                                            if self.session.state.run.fast_mode {
                                                "FAST ON  F"
                                            } else {
                                                "Fast  F"
                                            },
                                            self.session.state.run.fast_mode,
                                        ))
                                        .clicked();
                                    let b = ui.add(egui::Button::new("Hold to brake").sense(Sense::click_and_drag()));
                                    brake = b.is_pointer_button_down_on();
                                }
                                help = ui.button("Help  F1").clicked();
                                settings = ui.button("Settings").clicked();
                            });
                        });
                        ui.horizontal_wrapped(|ui| {
                            ui.strong(if self.session.state.mode == Mode::Practice { format!("{:04.0} / 1200 m", self.session.state.run.distance) } else { format!("{:04.0} m", self.session.state.run.distance) });
                            ui.separator();
                            ui.label(format!("{} crashes left", 3 - self.session.state.run.crashes));
                            ui.separator();
                            ui.label(format!("{:02.0} km/h", self.session.state.run.speed * 3.6));
                            if self.session.state.run.fast_mode {
                                ui.colored_label(arcade_presentation::AMBER, "FAST");
                            }
                            if self.session.state.legacy_run {
                                ui.label("Legacy run");
                            }
                            ui.separator();
                            if self.session.state.mode == Mode::Slalom {
                                ui.label(format!("{:.2} s + {:.0} s", self.session.state.run.ticks as f64 / 60., self.session.state.slalom.penalty_ticks as f64 / 60.));
                                if let Some(best) = self.session.state.slalom_best[self.session.state.course_index as usize] { ui.label(format!("Best {:.2} s", best as f64 / 60.)); }
                            } else { ui.label(format!("Best {:.0} m", self.session.state.best())); }
                            if self.session.state.run.tumble > 0 { ui.colored_label(arcade_presentation::AMBER, "Recovering"); }
                            else if self.session.state.run.protection > 0 { ui.label("Protected"); }
                            else if self.session.state.run.jump.is_some() { ui.label("Airborne"); }
                            if self.session.state.run.phase == Phase::Ready && !self.blocked() && self.session.state.mode == Mode::FreeSki {
                                ui.separator();
                                let mut enabled = self.session.state.chase_enabled;
                                if ui.checkbox(&mut enabled, "Creature pursuit · begins after 1,000 m").changed() { self.session.state.set_chase_enabled(enabled); self.flush(); }
                            }
                        });
                        if self.session.state.run.phase == Phase::Ready && !self.blocked() && self.session.state.mode == Mode::Slalom {
                            ui.horizontal_wrapped(|ui| {
                                for index in 0..course::COURSE_COUNT {
                                    let c = course::course(index).unwrap();
                                    let unlocked = index < self.session.state.unlocked_courses;
                                    let label = if unlocked {
                                        let medal = self.session.state.slalom_best[index as usize].map(|ticks| if ticks <= c.gold_ticks { " · Gold" } else if ticks <= c.silver_ticks { " · Silver" } else { " · Bronze" }).unwrap_or("");
                                        format!("{}. {}{}", index + 1, c.name, medal)
                                    } else { format!("{}. Locked", index + 1) };
                                    if ui.add_enabled_ui(unlocked, |ui| menu::choice(ui, &label, index == self.session.state.course_index)).inner.clicked() { self.session.state.select_course(index); self.refresh_obstacles(); self.flush(); }
                                }
                            });
                        }
                        ui.scope(|ui| {
                            ui.spacing_mut().interact_size.y = 16.;
                            let hint = match self.session.state.mode {
                                Mode::Practice => world::lesson(self.session.state.run.position.y).1.to_owned(),
                                Mode::FreeSki => {
                                    use crate::chase::ChasePhase;
                                    match self.session.state.chase.phase {
                                        ChasePhase::Dormant if self.session.state.chase_enabled => "Pursuit wakes after 1,000 m. Save speed for the chase.",
                                        ChasePhase::Dormant => "Find a line through the trees. Ramps are optional. Three crashes end the run.",
                                        ChasePhase::Warning => "SOMETHING IS COMING · Build speed and plan your turns.",
                                        ChasePhase::Active => "THE CHASE IS ON · Carve around terrain to shake the creature.",
                                    }.to_owned()
                                }
                                Mode::Slalom => {
                                    let c = course::course(self.session.state.course_index).unwrap();
                                    format!("{} · Gate {} / {} · Miss +5 s · Gold ≤ {:.1} s · Silver ≤ {:.1} s", c.name, (self.session.state.slalom.next_gate + 1).min(c.gates.len()), c.gates.len(), c.gold_ticks as f64 / 60., c.silver_ticks as f64 / 60.)
                                }
                            };
                            ui.label(egui::RichText::new(hint).size(13.));
                        });
                        if let Some(error) = &self.error { ui.colored_label(if self.theme.light() { Color32::DARK_RED } else { Color32::LIGHT_RED }, error); }
                        if !self.writable && !self.recovery { ui.label("Playing without saving. The original save is retained."); }
                    });
                ui.add_space(12.);
                let (area, _) = ui
                    .allocate_exact_size(ui.available_size() - Vec2::new(0., 10.), Sense::hover());
                field = render::field(area.shrink(12.));
                let tracks: Vec<_> = self.tracks.iter().copied().collect();
                render::draw(
                    ui,
                    field,
                    &self.session.state,
                    &self.session.obstacles,
                    self.theme.accent,
                    render::Effects { tracks: &tracks, braking: self.controls.braking(), landing: self.landing },
                );
            });
        if let Some(mode) = mode_choice {
            self.pending_mode = Some(mode);
            self.new_run();
        }
        if (fast_key || fast_button) && self.session.state.run.toggle_fast_mode() {
            self.flush();
        }
        if pause {
            self.pause("Take a breath. Your run is saved.");
        }
        if help {
            self.pause("Help is open. Resume when you are ready.");
            self.help = true;
        }
        if settings {
            self.pause("Settings are open. Resume when you are ready.");
            self.settings = true;
        }
        if restart {
            self.new_run();
        }
        if start
            || (enter && self.session.state.run.phase == Phase::Ready && !self.blocked() && focused)
        {
            self.begin();
        }
        // Sampling is allowed at Ready for intentional keyboard starts. Overlays and
        // focus changes clear both input ownership and held keys until fresh release.
        if !self.blocked()
            && focused
            && matches!(self.session.state.run.phase, Phase::Ready | Phase::Running)
        {
            let skier = render::point(
                field,
                &self.session.state.run,
                self.session.state.run.position,
            );
            let (input, intentional) = ctx.input(|i| {
                self.controls
                    .sample(i, field, skier, brake, self.session.state.run.heading)
            });
            if intentional && self.session.state.run.phase == Phase::Ready {
                self.session.state.run.start();
                self.flush();
            }
            if was_running && self.session.state.run.phase == Phase::Running {
                self.accumulator += elapsed;
                if self.accumulator > 0.25 {
                    self.pause("The slope fell behind. Paused so no collision time is skipped.");
                } else {
                    while self.accumulator + 1e-12 >= DT
                        && self.session.state.run.phase == Phase::Running
                    {
                        self.accumulator = (self.accumulator - DT).max(0.);
                        let before = self.session.state.run.position;
                        let was_airborne = self.session.state.run.jump.is_some();
                        let events = self.session.step(
                            self.controls
                                .for_tick(input, self.session.state.run.heading),
                        );
                        if was_airborne
                            && self.session.state.run.jump.is_none()
                            && self.session.state.run.tumble == 0
                            && !self.session.state.run.ended()
                            && !self.session.state.reduced_effects
                        {
                            self.landing = Some((
                                self.session.state.run.position,
                                self.session.state.run.ticks,
                            ));
                        }
                        if !self.session.state.muted {
                            for event in events {
                                self.play_event(event);
                            }
                            if self.session.state.run.ticks.is_multiple_of(24)
                                && self.session.state.run.speed > 8.
                                && self.session.state.run.jump.is_none()
                                && (input.brake || self.session.state.run.heading.abs() > 0.2)
                            {
                                self.audio.play(Cue::Carve);
                            }
                        }
                        if !self.session.state.reduced_effects
                            && self.session.state.run.jump.is_none()
                            && self.session.state.run.tumble == 0
                            && self.session.state.run.ticks.is_multiple_of(3)
                        {
                            let after = self.session.state.run.position;
                            if (before.x - after.x).hypot(before.y - after.y) < 1.
                                && (after.x - before.x).hypot(after.y - before.y) > 0.001
                            {
                                self.tracks.push_back((before, after));
                            }
                            while self.tracks.len() > 240 {
                                self.tracks.pop_front();
                            }
                        }
                        if self.session.state.run.ended()
                            || self.session.state.run.ticks.is_multiple_of(300)
                        {
                            self.flush();
                        }
                    }
                }
            }
        } else {
            self.controls.clear();
            self.accumulator = 0.;
        }
        self.show_menus(ctx, focused, enter);
        if self.session.state.run.phase == Phase::Running {
            ctx.request_repaint_after(Duration::from_millis(16));
        } else {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
    fn show_menus(&mut self, ctx: &egui::Context, focused: bool, enter: bool) {
        let theme = self.theme.clone();
        if self.recovery {
            menu::show(ctx, "freeski-recovery", &theme, |ui| {
                menu::heading(
                    ui,
                    &theme,
                    "SAVE RECOVERY",
                    "Your save needs attention",
                    "The existing file could not be restored. Your original is safely retained.",
                );
                if menu::action(ui, &theme, "Play without saving", true).clicked() {
                    self.recovery = false;
                    self.controls.clear();
                }
                menu::section(ui, "START FRESH");
                ui.label("Archive the original file and begin a new practice run.");
                if menu::action(ui, &theme, "Archive original and start fresh", false).clicked() {
                    match storage::archive(&self.path) {
                        Ok(_) => {
                            self.writable = true;
                            self.recovery = false;
                            self.session.state = Save::default();
                            self.new_run();
                        }
                        Err(e) => self.error = Some(e),
                    }
                }
                menu::section(ui, "ARCADE");
                if menu::action(ui, &theme, "Back to Arcade", false).clicked() {
                    self.leave = true;
                }
            });
        } else if self.restart {
            menu::show(ctx, "freeski-restart", &theme, |ui| {
                menu::heading(
                    ui,
                    &theme,
                    "NEW ATTEMPT",
                    "Replace this run?",
                    "Your unfinished run will be replaced. Records, medals and preferences stay.",
                );
                menu::stats(
                    ui,
                    &[
                        (
                            "DISTANCE",
                            format!("{:.0} m", self.session.state.run.distance),
                        ),
                        (
                            "TIME SKIING",
                            format!("{:.1} s", self.session.state.run.ticks as f64 / 60.),
                        ),
                    ],
                );
                if menu::action(ui, &theme, "Keep this run  Esc", true).clicked() {
                    self.restart = false;
                    self.pending_mode = None;
                }
                if menu::action(ui, &theme, "Replace run  Enter", false).clicked() || enter {
                    self.new_run();
                }
            });
        } else if self.help {
            menu::show(ctx, "freeski-help", &theme, |ui| {
                menu::heading(
                    ui,
                    &theme,
                    "FIELD GUIDE",
                    "Find your line",
                    "Build speed downhill. Carve to turn. Leave a little room to recover.",
                );
                let page_id = egui::Id::new("freeski-help-page");
                let mut modes = ctx.data_mut(|data| *data.get_temp_mut_or_default::<bool>(page_id));
                ui.horizontal(|ui| {
                    if menu::choice(ui, "Controls", !modes).clicked() {
                        modes = false;
                    }
                    if menu::choice(ui, "Run types", modes).clicked() {
                        modes = true;
                    }
                });
                ctx.data_mut(|data| data.insert_temp(page_id, modes));
                ui.add_space(8.);
                if modes {
                    menu::control(ui, "Practice", "P", "A 1,200 m learning slope. Three crashes end a run; the gold ring shows recovery protection.");
                    menu::control(ui, "Free Ski", "F · C pursuit", "An endless mountain. Enable the creature before starting for a chase after 1,000 m.");
                    menu::control(ui, "Slalom", "L · 1–5 courses", "Pass between ordered gates. Each miss adds five seconds. Finish to earn a medal and unlock the next course.");
                } else {
                    menu::control(ui, "Steer", "A / D or ← / →", "Hold to turn up to 90°. Release to keep your heading. Move the pointer left or right of the skier to aim with the mouse.");
                    menu::control(ui, "Fast tuck", "F", "Toggle fast mode in any active run. It raises your top speed from 216 to 324 km/h; turn early and avoid obstacles.");
                    menu::control(ui, "Brake", "S / ↓", "Or hold the right mouse button on the slope. Striped ramps launch you; low rocks can be jumped, trees cannot.");
                    menu::control(ui, "Pause", "Esc", "Enter resumes. Ctrl+H returns to Arcade. Your run saves and reopens paused.");
                }
                ui.label(
                    egui::RichText::new("Ctrl+M mute · Ctrl+, settings · Tab + Space buttons")
                        .size(13.)
                        .color(menu::muted(ui)),
                );
                if menu::action(ui, &theme, "Close  Esc", true).clicked() {
                    self.help = false;
                }
                ui.label(
                    egui::RichText::new(
                        "Original game, art and sound · Omarchy Arcade contributors
GPL-3.0-or-later",
                    )
                    .size(11.)
                    .color(menu::muted(ui)),
                );
            });
        } else if self.settings {
            menu::show(ctx, "freeski-settings", &theme, |ui| {
                menu::heading(
                    ui,
                    &theme,
                    "PREFERENCES",
                    "Settings",
                    "Your preferences save automatically and apply to every FreeSki mode.",
                );
                if let Some(records) = &self.session.state.legacy_records {
                    ui.label(
                        egui::RichText::new("Previous records retained from earlier rules.")
                            .size(13.)
                            .color(menu::muted(ui)),
                    );
                    ui.collapsing("Previous records", |ui| {
                        ui.label(format!(
                            "Practice: {:.0} m · {} completions",
                            records.best_distance, records.completions
                        ));
                        ui.label(format!(
                            "Free Ski: {:.0} m · Pursuit: {:.0} m",
                            records.free_best_distance, records.chase_best_distance
                        ));
                        for (index, best) in records.slalom_best.iter().enumerate() {
                            let course = course::course(index as u8).unwrap();
                            let result = best.map_or_else(
                                || "—".to_owned(),
                                |ticks| format!("{:.2} s", ticks as f64 / 60.),
                            );
                            ui.label(format!("{}: {result}", course.name));
                        }
                    });
                    ui.separator();
                }
                if menu::setting(
                    ui,
                    &mut self.session.state.muted,
                    "Mute sound  Ctrl+M",
                    "Silence skiing and event sounds. Visual warnings stay on.",
                ) {
                    self.audio.stop();
                    self.flush();
                }
                if menu::setting(
                    ui,
                    &mut self.session.state.reduced_effects,
                    "Reduced effects (hide ski tracks)",
                    "Hide tracks, powder, landing puffs and creature stride animation.",
                ) {
                    self.tracks.clear();
                    self.landing = None;
                    self.flush();
                }
                ui.add_space(12.);
                if menu::action(ui, &theme, "Close  Esc", true).clicked() {
                    self.settings = false;
                }
            });
        } else if self.session.state.run.phase == Phase::Paused {
            menu::show(ctx, "freeski-pause", &theme, |ui| {
                let mode = match self.session.state.mode {
                    Mode::Practice => "PRACTICE",
                    Mode::FreeSki => "FREE SKI",
                    Mode::Slalom => "SLALOM CUP",
                };
                menu::heading(ui, &theme, mode, "Run paused", &self.pause_reason);
                menu::stats(
                    ui,
                    &[
                        (
                            "DISTANCE",
                            format!("{:.0} m", self.session.state.run.distance),
                        ),
                        (
                            "CRASHES LEFT",
                            format!("{} / 3", 3 - self.session.state.run.crashes),
                        ),
                        (
                            "PACE",
                            if self.session.state.run.fast_mode {
                                "FAST".to_owned()
                            } else {
                                "NORMAL".to_owned()
                            },
                        ),
                    ],
                );
                let resume = ui
                    .add_enabled_ui(focused, |ui| {
                        menu::action(ui, &theme, "Resume skiing  Enter", true).clicked()
                    })
                    .inner;
                if resume || (enter && focused) {
                    self.begin();
                }
                if self.error.is_some()
                    && self.writable
                    && menu::action(ui, &theme, "Retry saving", false).clicked()
                {
                    self.flush();
                }
                menu::section(ui, "ANOTHER RUN");
                if menu::action(
                    ui,
                    &theme,
                    match self.session.state.mode {
                        Mode::Practice => "Restart practice slope",
                        Mode::FreeSki => "New mountain",
                        Mode::Slalom => "Restart course",
                    },
                    false,
                )
                .clicked()
                {
                    self.pending_mode = None;
                    self.restart = true;
                }
                self.mode_actions(ui, &theme, true);
                menu::section(ui, "OPTIONS");
                menu::pair(ui, |left, right| {
                    if menu::action(left, &theme, "Settings", false).clicked() {
                        self.settings = true;
                    }
                    if menu::action(right, &theme, "Back to Arcade", false).clicked() {
                        self.suspend();
                        self.leave = true;
                    }
                });
            });
        } else if self.session.state.run.ended() {
            menu::show(ctx, "freeski-result", &theme, |ui| {
                let finished = self.session.state.run.phase == Phase::Finished;
                let title = if finished {
                    "Fresh tracks. Slope complete."
                } else if self.session.state.run.phase == Phase::Caught {
                    "Caught! One more mountain?"
                } else {
                    "Time for a fresh start."
                };
                let subtitle = if self.session.state.mode == Mode::Slalom {
                    course::course(self.session.state.course_index)
                        .unwrap()
                        .name
                } else if self.session.state.chase_enabled {
                    "Creature pursuit · distance record"
                } else {
                    "Your latest tracks on the mountain"
                };
                menu::heading(ui, &theme, "RUN COMPLETE", title, subtitle);
                if self.session.state.mode == Mode::Slalom {
                    menu::stats(
                        ui,
                        &[
                            (
                                "FINAL TIME",
                                format!(
                                    "{:.2} s",
                                    self.session
                                        .state
                                        .slalom
                                        .final_ticks(self.session.state.run.ticks)
                                        as f64
                                        / 60.
                                ),
                            ),
                            (
                                "PENALTIES",
                                format!(
                                    "+{:.0} s",
                                    self.session.state.slalom.penalty_ticks as f64 / 60.
                                ),
                            ),
                        ],
                    );
                    ui.label(format!(
                        "{:.2} s skiing · {} missed gates · {} crashes",
                        self.session.state.run.ticks as f64 / 60.,
                        self.session.state.slalom.missed,
                        self.session.state.run.crashes
                    ));
                    if finished {
                        if let Some(medal) = self.session.state.slalom_medal() {
                            ui.label(
                                egui::RichText::new(format!("{medal:?} medal"))
                                    .size(22.)
                                    .strong()
                                    .color(theme.accent),
                            );
                        }
                        if self.session.state.course_index == 4 {
                            ui.label("Slalom Cup complete. All five courses are yours.");
                        } else if menu::action(ui, &theme, "Next course", true).clicked() {
                            self.session
                                .state
                                .select_course(self.session.state.course_index + 1);
                            self.refresh_obstacles();
                            self.tracks.clear();
                            self.landing = None;
                            self.audio.stop();
                            self.controls.clear();
                            self.flush();
                        }
                    }
                } else {
                    menu::stats(
                        ui,
                        &[
                            (
                                "DISTANCE",
                                format!("{:.0} m", self.session.state.run.distance),
                            ),
                            (
                                "PERSONAL BEST",
                                format!("{:.0} m", self.session.state.best()),
                            ),
                        ],
                    );
                    ui.label(format!(
                        "{} crashes · {:.1} seconds skiing",
                        self.session.state.run.crashes,
                        self.session.state.run.ticks as f64 / 60.
                    ));
                }
                ui.add_space(8.);
                let next_available = finished
                    && self.session.state.mode == Mode::Slalom
                    && self.session.state.course_index < 4;
                if menu::action(
                    ui,
                    &theme,
                    match self.session.state.mode {
                        Mode::Practice => "New practice run  Enter",
                        Mode::FreeSki => "New mountain  Enter",
                        Mode::Slalom => "Retry course  Enter",
                    },
                    !next_available,
                )
                .clicked()
                    || enter
                {
                    self.new_run();
                }
                menu::section(ui, "EXPLORE THE MOUNTAIN");
                self.mode_actions(ui, &theme, false);
                if menu::action(ui, &theme, "Back to Arcade", false).clicked() {
                    self.leave = true;
                }
            });
        }
    }
    fn mode_actions(&mut self, ui: &mut egui::Ui, theme: &Theme, confirm: bool) {
        let mut choice = None;
        let alternative = if self.session.state.mode == Mode::Practice {
            ("Try endless Free Ski", Mode::FreeSki)
        } else {
            ("Switch to practice", Mode::Practice)
        };
        if self.session.state.mode == Mode::Slalom {
            if menu::action(ui, theme, alternative.0, false).clicked() {
                choice = Some(alternative.1);
            }
        } else {
            menu::pair(ui, |left, right| {
                if menu::action(left, theme, "Try Slalom Cup", false).clicked() {
                    choice = Some(Mode::Slalom);
                }
                if menu::action(right, theme, alternative.0, false).clicked() {
                    choice = Some(alternative.1);
                }
            });
        }
        if let Some(mode) = choice {
            self.pending_mode = Some(mode);
            if confirm {
                self.restart = true;
            } else {
                self.new_run();
            }
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.show(ctx);
    }
    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        self.suspend();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Event, Modifiers, PointerButton, Pos2, RawInput};
    struct Harness {
        app: App,
        ctx: egui::Context,
        time: f64,
        dir: tempfile::TempDir,
        focused: bool,
        size: Vec2,
    }
    impl Harness {
        fn new() -> Self {
            let dir = tempfile::tempdir().unwrap();
            let app = App::open(dir.path().join("freeski.json")).unwrap();
            let mut h = Self {
                app,
                ctx: egui::Context::default(),
                time: 0.,
                dir,
                focused: true,
                size: Vec2::new(900., 710.),
            };
            h.frame(vec![], DT);
            h.frame(vec![], DT);
            h
        }
        fn frame(&mut self, events: Vec<Event>, dt: f64) -> egui::FullOutput {
            self.time += dt;
            let mut elapsed = dt;
            self.ctx.run(
                RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, self.size)),
                    time: Some(self.time),
                    focused: self.focused,
                    events,
                    ..Default::default()
                },
                |ctx| {
                    self.app.frame(ctx, elapsed);
                    elapsed = 0.;
                },
            )
        }
        fn key_event(key: Key, pressed: bool) -> Event {
            Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: Modifiers::NONE,
            }
        }
        fn key(&mut self, key: Key) {
            self.frame(vec![Self::key_event(key, true)], DT);
            self.frame(vec![Self::key_event(key, false)], DT);
        }
        fn label(&mut self, label: &str) -> Pos2 {
            let out = self.frame(vec![], DT);
            out.shapes
                .iter()
                .rev()
                .find_map(|s| match &s.shape {
                    egui::Shape::Text(t) if t.galley.text() == label => {
                        Some(t.pos + t.galley.size() / 2.)
                    }
                    _ => None,
                })
                .unwrap_or_else(|| panic!("missing label {label}"))
        }
        fn click(&mut self, label: &str) {
            let pos = self.label(label);
            self.frame(
                vec![
                    Event::PointerMoved(pos),
                    Event::PointerButton {
                        pos,
                        button: PointerButton::Primary,
                        pressed: true,
                        modifiers: Modifiers::NONE,
                    },
                ],
                DT,
            );
            self.frame(
                vec![Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                }],
                DT,
            );
            self.frame(vec![], DT);
        }
    }
    #[test]
    fn help_pages_and_preferences_fit_compact_window_without_advancing_run() {
        let mut h = Harness::new();
        h.key(Key::Enter);
        h.key(Key::Escape);
        let saved = h.app.session.state.run.clone();
        for size in [Vec2::new(900., 710.), Vec2::new(1280., 900.)] {
            h.size = size;
            h.key(Key::F1);
            for page in ["Run types", "Controls"] {
                h.click(page);
                let close = h.label("Close  Esc");
                assert!(Rect::from_min_size(Pos2::ZERO, size)
                    .shrink(24.)
                    .contains(close));
                assert_eq!(h.app.session.state.run, saved);
            }
            h.click("Close  Esc");
            h.click("Settings");
            let close = h.label("Close  Esc");
            assert!(Rect::from_min_size(Pos2::ZERO, size)
                .shrink(24.)
                .contains(close));
            h.click("Close  Esc");
            assert_eq!(h.app.session.state.run, saved);
        }
    }

    #[test]
    fn mouse_flow_pause_settings_restart_cancel_and_reopen() {
        let mut h = Harness::new();
        h.click("Start skiing  Enter");
        assert_eq!(h.app.session.state.run.phase, Phase::Running);
        for _ in 0..100 {
            h.frame(vec![], DT);
        }
        h.click("Pause  Esc");
        assert_eq!(h.app.session.state.run.phase, Phase::Paused);
        let saved = h.app.session.state.run.clone();
        h.click("Restart practice slope");
        h.click("Keep this run  Esc");
        assert_eq!(h.app.session.state.run, saved);
        h.click("Settings");
        h.click("Reduced effects (hide ski tracks)");
        h.click("Close  Esc");
        assert!(h.app.session.state.reduced_effects);
        h.click("Resume skiing  Enter");
        assert_eq!(h.app.session.state.run.phase, Phase::Running);
        h.app.suspend();
        let reopened = App::open(h.dir.path().join("freeski.json")).unwrap();
        assert_eq!(reopened.session.state, h.app.session.state);
    }
    #[test]
    fn visible_brake_is_held_and_results_restart_without_losing_record() {
        let mut h = Harness::new();
        h.click("Start skiing  Enter");
        for _ in 0..180 {
            h.frame(vec![], DT);
        }
        assert!(h.app.session.state.run.speed > 10.);
        let pos = h.label("Hold to brake");
        h.frame(
            vec![
                Event::PointerMoved(pos),
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
            ],
            DT,
        );
        // The faster buildup reaches a higher entry speed; verify sustained
        // braking actually slows each tick and reaches rest within three seconds.
        for _ in 0..180 {
            let before = h.app.session.state.run.speed;
            h.frame(vec![], DT);
            assert!(h.app.session.state.run.speed <= before);
        }
        assert!(h.app.session.state.run.speed < 0.01);
        h.frame(
            vec![Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            }],
            DT,
        );
        h.app.session.state.run.phase = Phase::Finished;
        h.app.session.state.run.position.y = world::FINISH;
        h.app.session.state.run.distance = world::FINISH;
        h.app.flush();
        h.frame(vec![], DT);
        h.click("New practice run  Enter");
        assert_eq!(h.app.session.state.run.phase, Phase::Ready);
        assert_eq!(h.app.session.state.completions, 1);
        assert_eq!(h.app.session.state.best_distance, world::FINISH);
    }

    #[test]
    fn focus_loss_and_overlay_dismissal_do_not_steer_or_advance() {
        let mut h = Harness::new();
        h.key(Key::Enter);
        h.frame(vec![Harness::key_event(Key::D, true)], DT);
        h.focused = false;
        h.frame(vec![], DT);
        assert_eq!(h.app.session.state.run.phase, Phase::Paused);
        let paused = h.app.session.state.run.clone();
        h.focused = true;
        h.frame(vec![], DT);
        assert_eq!(h.app.session.state.run, paused);
        h.key(Key::Enter);
        let resumed = h.app.session.state.run.heading;
        h.frame(vec![], DT);
        assert!(
            h.app.session.state.run.heading <= resumed,
            "held D must not continue steering after resume"
        );
        h.frame(vec![Harness::key_event(Key::D, false)], DT);
        h.key(Key::F1);
        let saved = h.app.session.state.run.clone();
        h.frame(vec![Harness::key_event(Key::D, true)], DT);
        h.key(Key::Escape);
        assert_eq!(h.app.session.state.run, saved);
    }
    #[test]
    fn keyboard_turns_to_traverse_and_release_keeps_heading() {
        let mut h = Harness::new();
        h.key(Key::Enter);
        h.frame(vec![Harness::key_event(Key::D, true)], DT);
        for _ in 0..65 {
            h.frame(vec![], DT);
        }
        assert_eq!(h.app.session.state.run.heading, crate::engine::MAX_HEADING);
        let y = h.app.session.state.run.position.y;
        h.frame(vec![Harness::key_event(Key::D, false)], DT);
        for _ in 0..20 {
            h.frame(vec![], DT);
        }
        assert_eq!(h.app.session.state.run.heading, crate::engine::MAX_HEADING);
        assert!((h.app.session.state.run.position.y - y).abs() < 1e-10);
        h.frame(vec![Harness::key_event(Key::A, true)], DT);
        for _ in 0..30 {
            h.frame(vec![], DT);
        }
        h.frame(vec![Harness::key_event(Key::A, false)], DT);
        let heading = h.app.session.state.run.heading;
        assert!(heading > 0. && heading < 0.9);
        for _ in 0..20 {
            h.frame(vec![], DT);
        }
        assert_eq!(h.app.session.state.run.heading, heading);
        assert!(h.app.session.state.run.position.y > y);
    }
    #[test]
    fn render_schedule_and_resize_leave_tick_simulation_equivalent() {
        let mut runs = vec![];
        for hz in [30, 60, 120] {
            let mut h = Harness::new();
            h.app.session.state.run.start();
            h.frame(vec![], DT);
            // Reset to an identical state after layout settles.
            h.app.session.state.run = crate::engine::Sim::default();
            h.app.session.state.run.start();
            h.app.accumulator = 0.;
            for frame in 0..hz * 5 {
                h.size = if frame % 2 == 0 {
                    Vec2::new(900., 710.)
                } else {
                    Vec2::new(1280., 850.)
                };
                h.frame(vec![], 1. / hz as f64);
            }
            assert_eq!(h.app.session.state.run.ticks, 300);
            runs.push(h.app.session.state.run.clone());
        }
        assert_eq!(runs[0], runs[1]);
        assert_eq!(runs[1], runs[2]);
    }
    #[test]
    fn released_heading_does_not_resurrect_after_a_crash_between_render_ticks() {
        let mut runs = vec![];
        for hz in [30, 60, 120] {
            let mut h = Harness::new();
            h.app.session.state.run = crate::engine::Sim {
                phase: Phase::Running,
                position: Point { x: -18., y: 98. },
                distance: 98.,
                speed: 50.,
                heading: 0.4,
                ..Default::default()
            };
            h.app.accumulator = 0.;
            for _ in 0..hz * 2 {
                h.frame(vec![], 1. / hz as f64);
            }
            assert_eq!(h.app.session.state.run.crashes, 1);
            assert_eq!(h.app.session.state.run.heading, 0.);
            runs.push(h.app.session.state.run.clone());
        }
        assert_eq!(runs[0], runs[1]);
        assert_eq!(runs[1], runs[2]);
    }

    #[test]
    fn modes_confirm_replacement_and_preserve_records_and_suspended_terrain() {
        let mut h = Harness::new();
        h.app.session.state.best_distance = world::FINISH;
        h.app.session.state.completions = 2;
        h.click("Free Ski");
        assert_eq!(h.app.session.state.mode, Mode::FreeSki);
        assert_ne!(h.app.session.state.seed, 0);
        h.click("Start skiing  Enter");
        for _ in 0..240 {
            h.frame(vec![], DT);
        }
        h.key(Key::Escape);
        let saved = h.app.session.state.clone();
        h.click("Switch to practice");
        h.click("Keep this run  Esc");
        assert_eq!(h.app.session.state, saved);
        h.app.suspend();
        let reopened = App::open(h.app.path.clone()).unwrap();
        assert_eq!(reopened.session.state, h.app.session.state);
        assert_eq!(
            reopened
                .session
                .obstacles
                .iter()
                .map(|o| (o.id, o.at))
                .collect::<Vec<_>>(),
            h.app
                .session
                .obstacles
                .iter()
                .map(|o| (o.id, o.at))
                .collect::<Vec<_>>()
        );
        h.click("Switch to practice");
        h.click("Replace run  Enter");
        assert_eq!(h.app.session.state.mode, Mode::Practice);
        assert_eq!(h.app.session.state.run.phase, Phase::Ready);
        assert_eq!(h.app.session.state.best_distance, world::FINISH);
        assert_eq!(h.app.session.state.completions, 2);
        assert_eq!(
            h.app.session.state.free_best_distance,
            saved.free_best_distance.max(saved.run.distance)
        );
    }

    #[test]
    fn endless_crosses_chunks_equally_at_different_render_rates() {
        let mut results = vec![];
        for hz in [30, 60, 120] {
            let mut h = Harness::new();
            h.app.session.state.select_mode(Mode::FreeSki, 42);
            h.app.refresh_obstacles();
            h.app.begin();
            h.app.accumulator = 0.;
            // Identical neutral input now encounters generator-2 edge hazards.
            // Rendering cadence must not change crossings, crashes or the result.
            h.app.session.state.run.position.x = -36.;
            for frame in 0..hz * 40 {
                h.size = if frame % 2 == 0 {
                    Vec2::new(900., 760.)
                } else {
                    Vec2::new(1280., 900.)
                };
                h.frame(vec![], 1. / hz as f64);
            }
            assert!(h.app.session.state.run.distance > crate::endless::CHUNK_LENGTH);
            assert_eq!(h.app.session.state.run.phase, Phase::Crashed);
            assert_eq!(h.app.session.state.run.crashes, 3);
            assert!(h.app.session.state.run.ticks < 2400);
            assert!(h.app.session.state.valid(&h.app.session.obstacles));
            results.push(h.app.session.state.clone());
        }
        assert_eq!(results[0], results[1]);
        assert_eq!(results[1], results[2]);
    }

    #[test]
    fn backlog_pauses_without_skipping_ticks_and_invalid_save_stays_intact() {
        let mut h = Harness::new();
        h.key(Key::Enter);
        let ticks = h.app.session.state.run.ticks;
        h.frame(vec![], 0.5);
        assert_eq!(h.app.session.state.run.phase, Phase::Paused);
        assert_eq!(h.app.session.state.run.ticks, ticks);
        h.app.suspend();
        std::fs::write(&h.app.path, b"future-save").unwrap();
        h.app = App::open(h.app.path.clone()).unwrap();
        h.frame(vec![], DT);
        h.click("Play without saving");
        h.click("Start skiing  Enter");
        h.app.suspend();
        assert_eq!(std::fs::read(&h.app.path).unwrap(), b"future-save");
    }
    #[test]
    fn slalom_course_results_unlock_and_next_button_preserve_records() {
        let mut h = Harness::new();
        h.click("Slalom");
        assert_eq!(h.app.session.state.mode, Mode::Slalom);
        assert_eq!(h.app.session.state.unlocked_courses, 1);
        assert!(!h.app.session.state.select_course(1));
        for index in 0..5 {
            assert_eq!(h.app.session.state.course_index, index);
            h.click("Start skiing  Enter");
            for _ in 0..15000 {
                if h.app.session.state.run.ended() {
                    break;
                }
                let heading = course::reference_heading(index, &h.app.session.state.run);
                h.app.session.step(crate::engine::Input {
                    heading,
                    brake: false,
                });
            }
            assert_eq!(h.app.session.state.run.phase, Phase::Finished);
            h.app.flush();
            assert!(h.app.error.is_none(), "{:?}", h.app.error);
            let recorded = h.app.session.state.slalom_best;
            h.frame(vec![], DT);
            if index < 4 {
                h.click("Next course");
            } else {
                h.click("Retry course  Enter");
            }
            assert_eq!(h.app.session.state.slalom_best, recorded);
            assert_eq!(h.app.session.state.run.phase, Phase::Ready);
        }
        h.app.suspend();
        let reopened = App::open(h.app.path.clone()).unwrap();
        assert_eq!(reopened.session.state, h.app.session.state);
        assert_eq!(reopened.session.state.unlocked_courses, 5);
    }

    #[test]
    fn pursuit_choice_mute_and_restart_are_persisted_through_visible_actions() {
        let mut h = Harness::new();
        h.click("Free Ski");
        h.click("Creature pursuit · begins after 1,000 m");
        assert!(h.app.session.state.chase_enabled);
        h.click("Start skiing  Enter");
        h.key(Key::Escape);
        h.click("Settings");
        h.click("Mute sound  Ctrl+M");
        h.click("Close  Esc");
        h.click("New mountain");
        h.click("Keep this run  Esc");
        assert_eq!(h.app.session.state.run.phase, Phase::Paused);
        h.click("New mountain");
        h.click("Replace run  Enter");
        assert!(h.app.session.state.chase_enabled);
        assert!(h.app.session.state.muted);
        h.app.suspend();
        assert_eq!(
            App::open(h.app.path.clone()).unwrap().session.state,
            h.app.session.state
        );
    }
    #[test]
    fn ready_keyboard_choices_and_locked_courses_do_not_start_or_erase_records() {
        let mut h = Harness::new();
        h.key(Key::F);
        h.key(Key::C);
        assert_eq!(h.app.session.state.mode, Mode::FreeSki);
        assert!(h.app.session.state.chase_enabled);
        h.key(Key::L);
        h.key(Key::Num5);
        assert_eq!(h.app.session.state.mode, Mode::Slalom);
        assert_eq!(h.app.session.state.course_index, 0);
        assert_eq!(h.app.session.state.run.phase, Phase::Ready);
        assert!(!h.app.session.state.chase_enabled);
        h.key(Key::Enter);
        let before = h.app.session.state.mode;
        h.key(Key::F);
        assert_eq!(h.app.session.state.mode, before);
    }

    #[test]
    fn fast_mode_requires_fresh_unmodified_running_action_and_has_mouse_control() {
        let mut h = Harness::new();
        h.key(Key::Enter);
        assert!(!h.app.session.state.run.fast_mode);

        h.frame(vec![Harness::key_event(Key::F, true)], DT);
        assert!(h.app.session.state.run.fast_mode);
        h.frame(
            vec![Event::Key {
                key: Key::F,
                physical_key: None,
                pressed: true,
                repeat: true,
                modifiers: Modifiers::NONE,
            }],
            DT,
        );
        assert!(h.app.session.state.run.fast_mode);
        h.frame(vec![Harness::key_event(Key::F, false)], DT);

        h.frame(
            vec![Event::Key {
                key: Key::F,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::CTRL,
            }],
            DT,
        );
        assert!(h.app.session.state.run.fast_mode);
        h.frame(vec![Harness::key_event(Key::F, false)], DT);

        h.key(Key::F);
        assert!(!h.app.session.state.run.fast_mode);
        h.key(Key::Escape);
        h.key(Key::F);
        assert!(!h.app.session.state.run.fast_mode);
        h.key(Key::Enter);
        h.click("Fast  F");
        assert!(h.app.session.state.run.fast_mode);

        h.app.suspend();
        assert!(
            App::open(h.app.path.clone())
                .unwrap()
                .session
                .state
                .run
                .fast_mode
        );
    }

    #[test]
    fn settings_keep_previous_rules_records_readable() {
        let mut h = Harness::new();
        h.app.session.state.legacy_records = Some(storage::LegacyRecords {
            best_distance: world::FINISH,
            free_best_distance: 7_626.,
            chase_best_distance: 2_014.,
            completions: 2,
            slalom_best: [Some(3_600), None, None, None, None],
        });
        h.click("Settings");
        h.label("Previous records retained from earlier rules.");
        h.click("Previous records");
        h.label("Practice: 1200 m · 2 completions");
        h.label("Free Ski: 7626 m · Pursuit: 2014 m");
        h.label("Pinecone Path: 60.00 s");
    }
}
