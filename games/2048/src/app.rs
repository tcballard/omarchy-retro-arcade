use crate::{
    engine::{self, Direction, Game, Turn},
    storage::{self, Save},
};
use eframe::egui::{self, Align2, Color32, FontId, Key, Pos2, Rect, Sense, Vec2};
use omarchy_chess::theme::Theme;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

pub struct App {
    state: Save,
    path: PathBuf,
    error: Option<String>,
    writable: bool,
    theme: Theme,
    themed: Instant,
    animation: Option<(Instant, Turn)>,
    paused: bool,
    help: bool,
    restart: bool,
    drag_origin: Option<Pos2>,
}
impl App {
    pub fn new() -> Result<Self, String> {
        Self::open(storage::path()?)
    }
    fn open(path: PathBuf) -> Result<Self, String> {
        let (state, error, writable) = match storage::load(&path) {
            Ok(s) => (
                s.unwrap_or_else(|| Save::new(Game::new(&mut rand::thread_rng(), 0))),
                None,
                true,
            ),
            Err(e) => (
                Save::new(Game::new(&mut rand::thread_rng(), 0)),
                Some(e),
                false,
            ),
        };
        let mut app = Self {
            state,
            path,
            error,
            writable,
            theme: Theme::load(),
            themed: Instant::now(),
            animation: None,
            paused: false,
            help: false,
            restart: false,
            drag_origin: None,
        };
        if writable {
            app.flush();
        }
        Ok(app)
    }
    fn flush(&mut self) {
        if self.writable {
            self.error = storage::save(&self.path, &self.state).err();
        }
    }
    fn play(&mut self, direction: Direction) {
        if let Some(turn) = self.state.game.play(direction, &mut rand::thread_rng()) {
            // A newer move replaces presentation only. Render from the authoritative board
            // at completion, so no stale animation can leave an unmergeable visual tile.
            self.animation = Some((Instant::now(), turn));
            self.flush();
        }
    }
    fn undo(&mut self) {
        if self.state.game.undo() {
            self.animation = None;
            self.flush();
        }
    }
    fn new_game(&mut self) {
        self.state.game = Game::new(&mut rand::thread_rng(), self.state.game.best);
        self.animation = None;
        self.restart = false;
        self.paused = false;
        self.flush();
    }
    fn tile(&self, ui: &egui::Ui, r: Rect, value: u64) {
        let level = value.ilog2() as f32;
        let color = self.theme.accent.lerp_to_gamma(
            arcade_presentation::AMBER,
            ((level - 1.) / 10.).clamp(0., 1.),
        );
        arcade_presentation::tile(ui.painter(), r, color, true);
        let digits = value.to_string().len() as f32;
        let size = (r.width() * 0.44).min(r.width() * 1.35 / digits).max(9.);
        let ink =
            if color.r() as u32 * 299 + color.g() as u32 * 587 + color.b() as u32 * 114 > 140000 {
                Color32::BLACK
            } else {
                Color32::WHITE
            };
        ui.painter().text(
            r.center(),
            Align2::CENTER_CENTER,
            value.to_string(),
            FontId::monospace(size),
            ink,
        );
    }
    fn board(&mut self, ui: &mut egui::Ui, enabled: bool) -> Option<Direction> {
        let side = (ui.available_height() - 24.)
            .min(ui.available_width() - 32.)
            .clamp(160., 540.);
        let (area, response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), side + 24.), Sense::hover());
        let board = Rect::from_center_size(area.center(), Vec2::splat(side));
        let response = ui.interact(board, response.id.with("board"), Sense::drag());
        response.clone().widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Other,
                enabled,
                "2048 board; drag to slide tiles",
            )
        });
        arcade_presentation::bezel(ui.painter(), board, self.theme.accent);
        let gap = side * 0.025;
        let cell = (side - gap * 5.) / 4.;
        let cell_rect = |r: f32, c: f32| {
            Rect::from_min_size(
                board.min + Vec2::new(gap + c * (cell + gap), gap + r * (cell + gap)),
                Vec2::splat(cell),
            )
        };
        ui.painter().rect_filled(board, 0., self.theme.background);
        for r in 0..4 {
            for c in 0..4 {
                ui.painter().rect_filled(
                    cell_rect(r as f32, c as f32),
                    1.,
                    self.theme
                        .background
                        .lerp_to_gamma(self.theme.foreground, 0.08),
                );
            }
        }
        let animation = self
            .animation
            .as_ref()
            .filter(|_| !self.state.reduced_motion);
        let time = animation
            .map(|(at, _)| at.elapsed().as_secs_f32())
            .unwrap_or(1.);
        if let Some((_, turn)) = animation.filter(|_| time < 0.12) {
            let t = (time / 0.12).clamp(0., 1.);
            let t = 1. - (1. - t).powi(3);
            for m in &turn.moves {
                let r = m.from[0] as f32 + (m.to[0] as f32 - m.from[0] as f32) * t;
                let c = m.from[1] as f32 + (m.to[1] as f32 - m.from[1] as f32) * t;
                self.tile(ui, cell_rect(r, c), m.value);
            }
        } else {
            for r in 0..4 {
                for c in 0..4 {
                    let value = self.state.game.board[r][c];
                    if value == 0 {
                        continue;
                    }
                    let mut rect = cell_rect(r as f32, c as f32);
                    if let Some((_, turn)) = animation.filter(|_| time < 0.24) {
                        let t = ((time - 0.12) / 0.12).clamp(0., 1.);
                        if turn.spawned == Some([r, c]) {
                            rect = Rect::from_center_size(
                                rect.center(),
                                rect.size() * (0.65 + 0.35 * t),
                            );
                        } else if turn.merges.contains(&[r, c]) {
                            rect = rect.expand((t * std::f32::consts::PI).sin() * cell * 0.035);
                        }
                    }
                    self.tile(ui, rect, value);
                }
            }
        }
        if time < 0.24 {
            ui.ctx().request_repaint_after(Duration::from_millis(16));
        }
        if !enabled {
            self.drag_origin = None;
        }
        if enabled && response.drag_started_by(egui::PointerButton::Primary) {
            self.drag_origin = ui.input(|i| i.pointer.press_origin());
        }
        if enabled && response.drag_stopped_by(egui::PointerButton::Primary) {
            let delta = self
                .drag_origin
                .take()
                .zip(ui.input(|i| i.pointer.latest_pos()))
                .map(|(start, end)| end - start)
                .unwrap_or(Vec2::ZERO);
            if delta.length() > 24. {
                return Some(if delta.x.abs() > delta.y.abs() {
                    if delta.x > 0. {
                        Direction::Right
                    } else {
                        Direction::Left
                    }
                } else if delta.y > 0. {
                    Direction::Down
                } else {
                    Direction::Up
                });
            }
        }
        None
    }
}
impl App {
    pub fn show(&mut self, ctx: &egui::Context) {
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
        if !ctx.input(|i| i.focused) {
            self.paused = true;
            self.animation = None;
        }
        let was_blocked = self.paused || self.help || self.restart;
        let mut undo = false;
        let mut direction = None;
        ctx.input_mut(|i| {
            if i.consume_key(egui::Modifiers::NONE, Key::F1) {
                self.help = !self.help;
            }
            if i.consume_key(egui::Modifiers::NONE, Key::Escape) {
                if self.help {
                    self.help = false;
                } else if self.restart {
                    self.restart = false;
                } else {
                    self.paused = !self.paused;
                }
            }
            if !was_blocked && !self.help && !self.paused && !self.restart && i.focused {
                undo = i.consume_key(egui::Modifiers::CTRL, Key::Z)
                    || i.consume_key(egui::Modifiers::NONE, Key::U)
                    || i.consume_key(egui::Modifiers::NONE, Key::Z);
                for (key, d) in [
                    (Key::ArrowLeft, Direction::Left),
                    (Key::A, Direction::Left),
                    (Key::ArrowRight, Direction::Right),
                    (Key::D, Direction::Right),
                    (Key::ArrowUp, Direction::Up),
                    (Key::W, Direction::Up),
                    (Key::ArrowDown, Direction::Down),
                    (Key::S, Direction::Down),
                ] {
                    if i.consume_key(egui::Modifiers::NONE, key) {
                        direction = Some(d);
                    }
                }
            }
        });
        let blocked = was_blocked || self.paused || self.help || self.restart;
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .inner_margin(24.)
                    .fill(self.theme.background),
            )
            .show(ctx, |ui| {
                arcade_presentation::backdrop(ui);
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new("2048").size(34.).monospace());
                    ui.separator();
                    ui.label(format!("SCORE  {}", self.state.game.score));
                    ui.label(format!("BEST  {}", self.state.game.best));
                });
                ui.label("Slide together. Make something bigger.");
                ui.add_space(6.);
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .add_enabled(!blocked, egui::Button::new("New game"))
                        .clicked()
                    {
                        self.restart = true;
                    }
                    undo |= ui
                        .add_enabled(
                            !blocked && !self.state.game.history.is_empty(),
                            egui::Button::new("Undo  Ctrl+Z"),
                        )
                        .clicked();
                    if ui
                        .add_enabled(!blocked, egui::Button::new("Pause  Esc"))
                        .clicked()
                    {
                        self.paused = true;
                    }
                    if ui.button("Help & credits  F1").clicked() {
                        self.help = true;
                    }
                });
                if let Some(error) = &self.error {
                    ui.colored_label(
                        if self.theme.light() {
                            Color32::DARK_RED
                        } else {
                            Color32::LIGHT_RED
                        },
                        error,
                    );
                    if !self.writable {
                        ui.label("Playing without saving. Your existing file is untouched.");
                    }
                }
                let enabled = !blocked
                    && !self.paused
                    && !self.help
                    && !self.restart
                    && !self.state.game.win_prompt()
                    && !engine::over(&self.state.game.board);
                ui.horizontal_wrapped(|ui| {
                    for (label, d) in [
                        ("←  Left", Direction::Left),
                        ("↑  Up", Direction::Up),
                        ("↓  Down", Direction::Down),
                        ("Right  →", Direction::Right),
                    ] {
                        if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
                            direction = Some(d);
                        }
                    }
                    ui.label("Arrows / WASD · Drag to slide");
                });
                ui.label("2048 adaptation: Avi Barit (avibarit) · Original game: Gabriele Cirulli");
                ui.add_space(8.);
                if let Some(d) = self.board(ui, enabled) {
                    direction = Some(d);
                }
            });
        // No action from a closing modal is allowed to become a board move.
        if !blocked && !self.paused && !self.help && !self.restart {
            if undo {
                self.undo();
            } else if let Some(d) = direction {
                self.play(d);
            }
        }
        if self.restart {
            egui::Modal::new(egui::Id::new("2048-restart")).show(ctx, |ui| {
                ui.heading("Start a new game?");
                ui.label(
                    "This replaces the current board and undo history. Your best score stays.",
                );
                ui.horizontal(|ui| {
                    if ui.button("Start new game").clicked() {
                        self.new_game();
                    }
                    if ui.button("Keep playing").clicked() {
                        self.restart = false;
                    }
                });
            });
        } else if self.help {
            let mut changed = false;
            egui::Modal::new(egui::Id::new("2048-help")).show(ctx,|ui| {
                ui.set_max_width(520.);ui.heading("2048 · Help & credits");
                ui.label("Slide with arrows, WASD, the direction buttons or a drag across the board. Equal tiles merge once per move. Reach 2048, then keep going.");
                ui.label("Undo: Ctrl+Z, U or Z (up to 20 moves). Esc pauses. Ctrl+H returns to Arcade. The board, best score and undo history save automatically.");
                changed=ui.checkbox(&mut self.state.reduced_motion,"Reduced motion").changed();
                ui.separator();
                ui.hyperlink_to("Adapted from Avi Barit's 2048 (avibarit)","https://github.com/avibarit/2048");
                ui.label("Used with the author's permission, reported by Tom Ballard. Upstream declares MIT. The original source and Git history are preserved in this repository.");
                ui.hyperlink_to("Original 2048 by Gabriele Cirulli","https://github.com/gabrielecirulli/2048");
                ui.label("Rust/egui adaptation and Arcade integration: Omarchy Arcade contributors.");
                if ui.button("Back to game").clicked() {self.help=false;}
            });
            if changed {
                self.animation = None;
                self.flush();
            }
        } else if self.paused {
            egui::Modal::new(egui::Id::new("2048-pause")).show(ctx, |ui| {
                ui.heading("Paused");
                ui.label("Your board is waiting.");
                if ui.button("Resume").clicked() {
                    self.paused = false;
                }
            });
        } else if self.state.game.win_prompt() || engine::over(&self.state.game.board) {
            let win = self.state.game.win_prompt();
            egui::Modal::new(egui::Id::new("2048-result")).show(ctx, |ui| {
                ui.heading(if win {
                    "You made 2048!"
                } else {
                    "No moves left"
                });
                ui.label(format!(
                    "Score {} · Best {}",
                    self.state.game.score, self.state.game.best
                ));
                if win && ui.button("Keep going").clicked() {
                    self.state.game.keep_going();
                    self.flush();
                }
                if ui
                    .add_enabled(
                        !self.state.game.history.is_empty(),
                        egui::Button::new("Undo last move"),
                    )
                    .clicked()
                {
                    self.undo();
                }
                if ui.button("New game").clicked() {
                    self.new_game();
                }
            });
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.show(ctx);
    }
    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Event, Modifiers, PointerButton, RawInput};
    struct Harness {
        app: App,
        ctx: egui::Context,
        time: f64,
        _dir: tempfile::TempDir,
    }
    impl Harness {
        fn new() -> Self {
            let dir = tempfile::tempdir().unwrap();
            let app = App::open(dir.path().join("2048.json")).unwrap();
            let mut h = Self {
                app,
                ctx: egui::Context::default(),
                time: 0.,
                _dir: dir,
            };
            h.app.state.game.board = [[2, 2, 0, 0], [0; 4], [0; 4], [0; 4]];
            h.frame(vec![]);
            h.frame(vec![]);
            h
        }
        fn frame(&mut self, events: Vec<Event>) -> egui::FullOutput {
            self.time += 0.02;
            let input = RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(900., 710.))),
                time: Some(self.time),
                events,
                focused: true,
                ..Default::default()
            };
            self.ctx.run(input, |ctx| self.app.show(ctx))
        }
        fn key(&mut self, key: Key, modifiers: Modifiers) {
            self.frame(vec![Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            }]);
            self.frame(vec![Event::Key {
                key,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers,
            }]);
        }
        fn label(&mut self, label: &str) -> Pos2 {
            fn find(shapes: &[egui::epaint::ClippedShape], label: &str) -> Option<Pos2> {
                shapes.iter().rev().find_map(|s| match &s.shape {
                    egui::Shape::Text(t) if t.galley.text() == label => {
                        Some(t.pos + t.galley.size() / 2.)
                    }
                    _ => None,
                })
            }
            let out = self.frame(vec![]);
            find(&out.shapes, label).unwrap_or_else(|| panic!("Missing label: {label}"))
        }
        fn click(&mut self, label: &str) {
            let pos = self.label(label);
            self.frame(vec![
                Event::PointerMoved(pos),
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
            ]);
            self.frame(vec![Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            }]);
            self.frame(vec![]);
        }
    }
    #[test]
    fn mouse_keyboard_undo_and_reopen() {
        let mut h = Harness::new();
        let before = h.app.state.game.board;
        h.click("←  Left");
        assert_eq!(h.app.state.game.score, 4);
        h.key(Key::Z, Modifiers::CTRL);
        assert_eq!(h.app.state.game.board, before);
        h.key(Key::D, Modifiers::NONE);
        assert_eq!(h.app.state.game.board[0][3], 4);
        h.click("Undo  Ctrl+Z");
        assert_eq!(h.app.state.game.board, before);
        h.click("Help & credits  F1");
        assert!(h.app.help);
        h.label("Adapted from Avi Barit's 2048 (avibarit)");
        h.label("Original 2048 by Gabriele Cirulli");
        h.click("Reduced motion");
        assert!(h.app.state.reduced_motion);
        h.click("Back to game");
        assert!(!h.app.help);
        let reopened = App::open(h.app.path.clone()).unwrap();
        assert_eq!(reopened.state, h.app.state);
    }
    #[test]
    fn board_drag_moves_once_and_is_blocked_by_help() {
        let mut h = Harness::new();
        let start = Pos2::new(450., 400.);
        let end = Pos2::new(300., 400.);
        h.frame(vec![
            Event::PointerMoved(start),
            Event::PointerButton {
                pos: start,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ]);
        h.frame(vec![Event::PointerMoved(end)]);
        h.frame(vec![Event::PointerButton {
            pos: end,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }]);
        assert_eq!(h.app.state.game.score, 4);
        assert_eq!(h.app.state.game.history.len(), 1);
        h.click("Help & credits  F1");
        let before = h.app.state.clone();
        h.key(Key::ArrowDown, Modifiers::NONE);
        assert_eq!(h.app.state, before);
        h.click("Back to game");
        assert_eq!(h.app.state, before);
    }
    #[test]
    fn pause_and_restart_do_not_leak_input() {
        let mut h = Harness::new();
        h.click("Pause  Esc");
        assert!(h.app.paused);
        let before = h.app.state.clone();
        h.key(Key::A, Modifiers::NONE);
        assert_eq!(h.app.state, before);
        h.click("Resume");
        assert_eq!(h.app.state, before);
        assert!(!h.app.paused);
        h.click("New game");
        assert!(h.app.restart);
        h.key(Key::D, Modifiers::NONE);
        assert_eq!(h.app.state, before);
        h.click("Keep playing");
        assert_eq!(h.app.state, before);
        h.app.state.game.best = 40;
        h.click("New game");
        h.click("Start new game");
        assert_eq!(h.app.state.game.score, 0);
        assert_eq!(h.app.state.game.best, 40);
        assert!(h.app.state.game.history.is_empty());
    }
    #[test]
    fn win_dialog_continue_undo_and_game_over() {
        let mut h = Harness::new();
        h.app.state.game.board = [[1024, 1024, 0, 0], [0; 4], [0; 4], [0; 4]];
        h.key(Key::A, Modifiers::NONE);
        assert!(h.app.state.game.win_prompt());
        h.click("Keep going");
        assert!(h.app.state.game.continued);
        h.key(Key::Z, Modifiers::CTRL);
        assert_eq!(h.app.state.game.score, 0);
        assert!(!h.app.state.game.continued);
        h.app.state.game.board = [
            [2, 4, 8, 16],
            [4, 8, 16, 32],
            [8, 16, 32, 64],
            [16, 32, 64, 128],
        ];
        h.frame(vec![]);
        h.label("No moves left");
        h.click("New game");
        assert!(!engine::over(&h.app.state.game.board));
    }
    #[test]
    fn corrupt_save_stays_untouched_after_play_and_exit() {
        let mut h = Harness::new();
        std::fs::write(&h.app.path, b"future-data").unwrap();
        h.app = App::open(h.app.path.clone()).unwrap();
        assert!(!h.app.writable);
        h.key(Key::A, Modifiers::NONE);
        h.app.flush();
        assert_eq!(std::fs::read(&h.app.path).unwrap(), b"future-data");
        assert!(h.app.error.is_some());
    }
}
