use crate::{
    engine::{Answer, Engine},
    game::{Difficulty, Game, Mode},
    preferences::Preferences,
    sound::Sound,
    storage,
    theme::Theme,
};
use eframe::egui::{self, Align2, Color32, FontId, Key, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use shakmaty::{uci::UciMove, CastlingMode, Color, Move, Position, Role, Square};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

pub struct ChessApp {
    pieces: crate::pieces::Pieces,
    pub game: Game,
    pub flipped: bool,
    pub guides: bool,
    pub selected: Option<Square>,
    pub cursor: Square,
    pub preview: Option<usize>,
    pub board_rect: Option<Rect>,
    pub message: String,
    state_dir: PathBuf,
    save_blocked: bool,
    engine: Engine,
    revision: u64,
    engine_error: bool,
    hint: Option<Move>,
    move_text: String,
    theme: Theme,
    theme_checked: Instant,
    new_dialog: bool,
    new_mode: Mode,
    new_white: bool,
    new_difficulty: Difficulty,
    promotion: Vec<Move>,
    resign_dialog: bool,
    help: bool,
    drag_source: Option<Square>,
    pub preferences: Preferences,
    settings: bool,
    sound: Sound,
    settings_blocked: bool,
}
impl ChessApp {
    pub fn prepare_to_leave(&mut self) -> Result<(), String> {
        self.engine.cancel();
        self.selected = None;
        self.drag_source = None;
        if self.save_blocked || self.settings_blocked {
            return Err(
                "The original Chess save or settings need recovery before saving is allowed."
                    .into(),
            );
        }
        storage::save(&self.state_dir, &self.game, self.flipped, self.guides)
            .map_err(|e| e.to_string())?;
        self.preferences
            .save(&self.state_dir)
            .map_err(|e| e.to_string())
    }
    pub fn new(state_dir: PathBuf) -> Self {
        let mut app = Self {
            pieces: crate::pieces::Pieces::default(),
            game: Game::default(),
            flipped: false,
            guides: true,
            selected: None,
            cursor: Square::E2,
            preview: None,
            board_rect: None,
            message: "Your move.".into(),
            state_dir,
            save_blocked: false,
            engine: Engine::default(),
            revision: 0,
            engine_error: false,
            hint: None,
            move_text: String::new(),
            theme: Theme::load(),
            theme_checked: Instant::now(),
            new_dialog: false,
            new_mode: Mode::Computer,
            new_white: true,
            new_difficulty: Difficulty::Casual,
            promotion: vec![],
            resign_dialog: false,
            help: false,
            drag_source: None,
            preferences: Preferences::default(),
            settings: false,
            sound: Sound::default(),
            settings_blocked: false,
        };
        if app.state_dir.join("session.json").exists() {
            match storage::load(&app.state_dir) {
                Ok((game, flipped, guides)) => {
                    app.game = game;
                    app.flipped = flipped;
                    app.guides = guides;
                }
                Err(e) => {
                    app.message=format!("Could not load your save: {e} Original file is untouched. New game archives it before replacing it.");
                    app.save_blocked = true;
                }
            }
        }
        match Preferences::load(&app.state_dir) {
            Ok(p) => app.preferences = p,
            Err(e) => {
                app.message =
                    format!("Settings could not be read: {e}. Original settings preserved.");
                app.settings_blocked = true;
            }
        }
        if !app.state_dir.join("session.json").exists() && app.engine_path().is_none() {
            app.game.mode = Mode::Local;
            app.new_mode = Mode::Local;
            app.message =
                "Local game ready. Set up Stockfish in Settings for computer play.".into();
        }
        app.theme = if app.preferences.follow_omarchy {
            Theme::load()
        } else {
            Theme::default()
        };
        app
    }
    fn engine_path(&self) -> Option<PathBuf> {
        self.preferences
            .engine_path
            .clone()
            .or_else(crate::engine::find_engine)
    }
    pub fn save_preferences(&mut self) {
        if self.settings_blocked {
            self.message = "Existing settings need recovery before changes can be saved.".into();
            return;
        }
        if let Err(e) = self.preferences.save(&self.state_dir) {
            self.message = format!("Could not save settings: {e}");
        }
    }
    pub fn rematch(&mut self) {
        let game = Game {
            mode: self.game.mode,
            human_white: self.game.human_white,
            difficulty: self.game.difficulty,
            ..Game::default()
        };
        if let Err(e) = self.replace_game(game) {
            self.message = e;
        }
    }
    fn move_sound(&self) {
        if self.preferences.sound {
            self.sound.play(self.game.finished());
        }
    }

    fn persist(&mut self) {
        if !self.save_blocked {
            if let Err(e) = storage::save(&self.state_dir, &self.game, self.flipped, self.guides) {
                self.message = format!("Could not save: {e}. Export PGN to keep a copy.");
            }
        }
    }
    fn invalidate(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        self.engine.cancel();
        self.hint = None;
        self.selected = None;
        self.preview = None;
        self.drag_source = None;
        self.promotion.clear();
    }
    fn status_message(&mut self) {
        if !self.save_blocked {
            self.message = if self.game.finished() {
                "Game finished. Start another, or take back a move to explore."
            } else if self.game.human_turn() {
                "Your move."
            } else {
                "Stockfish is thinking…"
            }
            .into();
        }
    }
    fn maybe_engine(&mut self) {
        if !self.engine.busy
            && !self.engine_error
            && !self.game.finished()
            && !self.game.human_turn()
        {
            self.engine
                .start_with_path(&self.game, self.revision, false, self.engine_path());
            if !self.save_blocked {
                self.message = "Stockfish is thinking…".into();
            }
        }
    }
    pub fn accept_move(&mut self, m: Move) {
        if self.preview.is_some() || !self.game.human_turn() {
            return;
        }
        if !self.game.position().is_legal(&m) {
            self.message = "That move is not legal.".into();
            return;
        }
        self.invalidate();
        match self.game.play(m) {
            Ok(()) => {
                self.engine_error = false;
                self.move_text.clear();
                self.status_message();
                self.move_sound();
                self.persist();
                self.maybe_engine();
            }
            Err(e) => self.message = e,
        }
    }
    pub fn accept_answer(&mut self, answer: Answer) {
        if answer.revision != self.revision {
            return;
        }
        self.engine.complete();
        match answer.result.and_then(|s| self.game.parse_move(&s)) {
            Ok(m) if answer.hint => {
                self.preview = None;
                self.message = format!(
                    "Try {}. You choose whether to play it.",
                    shakmaty::san::SanPlus::from_move(self.game.position().clone(), &m)
                );
                self.hint = Some(m);
            }
            Ok(m) => {
                if !self.game.human_turn() && !self.game.finished() {
                    match self.game.play(m) {
                        Ok(()) => {
                            self.revision = self.revision.wrapping_add(1);
                            self.preview = None;
                            self.status_message();
                            self.move_sound();
                            self.persist();
                        }
                        Err(e) => self.message = e,
                    }
                }
            }
            Err(e) => {
                self.engine_error = true;
                self.message = e;
            }
        }
    }
    pub fn takeback(&mut self) {
        if self.game.can_takeback() {
            self.invalidate();
            self.game.takeback();
            self.engine_error = false;
            self.status_message();
            self.persist();
            self.maybe_engine();
        }
    }
    fn request_hint(&mut self) {
        if self.game.human_turn() && self.preview.is_none() && !self.engine.busy {
            self.engine
                .start_with_path(&self.game, self.revision, true, self.engine_path());
            self.message = "Finding a hint…".into();
        }
    }
    pub fn replace_game(&mut self, game: Game) -> Result<(), String> {
        storage::archive(&self.state_dir, &self.game, self.save_blocked)?;
        self.save_blocked = false;
        self.invalidate();
        self.game = game;
        self.engine_error = false;
        self.flipped = self.game.mode == Mode::Computer && !self.game.human_white;
        self.status_message();
        self.persist();
        self.maybe_engine();
        Ok(())
    }
    fn import_pgn(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Chess games", &["pgn"])
            .pick_file()
        {
            let result = storage::read_bounded(&path, crate::game::MAX_PGN)
                .and_then(|b| String::from_utf8(b).map_err(|e| e.to_string()))
                .and_then(|s| Game::from_pgn(&s))
                .and_then(|g| self.replace_game(g));
            match result {
                Ok(()) => {
                    self.message =
                        "Imported main line for local play. Original file is unchanged.".into()
                }
                Err(e) => self.message = format!("Could not import: {e}"),
            }
        }
    }
    fn export_pgn(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Chess games", &["pgn"])
            .set_file_name("omarchy-chess.pgn")
            .save_file()
        {
            self.message = match storage::atomic_write(&path, self.game.pgn().as_bytes()) {
                Ok(()) => "Game exported.".into(),
                Err(e) => format!("Could not export: {e}"),
            };
        }
    }
    fn flip(&mut self) {
        self.flipped = !self.flipped;
        self.persist();
    }
    fn apply_style(&self, ctx: &egui::Context) {
        let mut v = if self.theme.light() {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };
        v.panel_fill = self.theme.background;
        v.window_fill = self.theme.background;
        v.override_text_color = Some(self.theme.foreground);
        v.selection.bg_fill = self.theme.accent;
        v.selection.stroke = Stroke::new(1_f32, self.theme.accent_text());
        v.widgets.active.bg_fill = self.theme.accent;
        v.widgets.active.fg_stroke = Stroke::new(1_f32, self.theme.accent_text());
        ctx.set_visuals(v);
        arcade_presentation::apply(ctx);
        ctx.style_mut(|style| {
            style.spacing.item_spacing = Vec2::new(10., 10.);
            style.spacing.button_padding = Vec2::new(10., 6.);
        });
    }
    fn shortcuts(&mut self, ctx: &egui::Context) {
        if self.new_dialog
            || self.help
            || self.settings
            || self.resign_dialog
            || !self.promotion.is_empty()
        {
            return;
        }
        let action = ctx.input_mut(|i| {
            [
                (Key::N, 0),
                (Key::O, 1),
                (Key::S, 2),
                (Key::Z, 3),
                (Key::H, 4),
                (Key::F, 5),
                (Key::L, 6),
                (Key::Q, 7),
                (Key::Comma, 8),
                (Key::M, 9),
            ]
            .into_iter()
            .find_map(|(k, a)| i.consume_key(egui::Modifiers::CTRL, k).then_some(a))
        });
        match action {
            Some(0) => self.new_dialog = true,
            Some(1) => self.import_pgn(),
            Some(2) => self.export_pgn(),
            Some(3) => self.takeback(),
            Some(4) => self.request_hint(),
            Some(5) => self.flip(),
            Some(6) => self.preview = None,
            Some(7) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            Some(8) => self.settings = true,
            Some(9) => {
                self.preferences.sound = !self.preferences.sound;
                self.save_preferences();
            }
            _ => (),
        }
        if ctx.input(|i| i.key_pressed(Key::F1)) {
            self.help = true;
        }
    }
    pub fn draw(&mut self, ctx: &egui::Context) {
        for answer in self.engine.poll() {
            self.accept_answer(answer);
        }
        self.maybe_engine();
        if self.theme_checked.elapsed() >= Duration::from_secs(2) {
            self.theme = if self.preferences.follow_omarchy {
                Theme::load()
            } else {
                Theme::default()
            };
            self.theme_checked = Instant::now();
        }
        self.pieces
            .refresh(ctx, self.theme.accent, self.preferences.piece_style);
        self.apply_style(ctx);
        self.shortcuts(ctx);
        ctx.request_repaint_after(if self.engine.busy {
            Duration::from_millis(25)
        } else {
            Duration::from_secs(2)
        });
        egui::TopBottomPanel::top("header")
            .frame(
                egui::Frame::new()
                    .fill(self.theme.background)
                    .inner_margin(12),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("ARCADE / CHESS")
                            .color(self.theme.accent)
                            .size(13.),
                    );
                    ui.separator();
                    if ui.button("New game").clicked() {
                        self.new_dialog = true;
                    }
                    ui.menu_button("Game", |ui| {
                        if ui.button("New game…   Ctrl+N").clicked() {
                            self.new_dialog = true;
                            ui.close_menu();
                        }
                        if ui.button("Rematch").clicked() {
                            self.rematch();
                            ui.close_menu();
                        }
                        if ui.button("Import PGN…   Ctrl+O").clicked() {
                            ui.close_menu();
                            self.import_pgn();
                        }
                        if ui.button("Export PGN…   Ctrl+S").clicked() {
                            ui.close_menu();
                            self.export_pgn();
                        }
                        if ui
                            .add_enabled(!self.game.finished(), egui::Button::new("Resign…"))
                            .clicked()
                        {
                            ui.close_menu();
                            self.resign_dialog = true;
                        }
                        if ui.button("Quit   Ctrl+Q").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    if ui.button("Settings").clicked() {
                        self.settings = true;
                    }
                    ui.menu_button("Help", |ui| {
                        if ui.button("How to play / About   F1").clicked() {
                            self.help = true;
                            ui.close_menu();
                        }
                    });
                });
            });
        egui::TopBottomPanel::bottom("message")
            .frame(
                egui::Frame::new()
                    .fill(self.theme.background)
                    .inner_margin(10),
            )
            .show(ctx, |ui| {
                let status = ui.label(&self.message);
                ctx.accesskit_node_builder(status.id, |node| {
                    node.set_live(egui::accesskit::Live::Polite)
                });
            });
        egui::SidePanel::right("controls")
            .exact_width(260.)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(self.theme.background)
                    .inner_margin(16),
            )
            .show(ctx, |ui| {
                ui.add_space(12.);
                ui.heading(self.game.status());
                if self.engine.busy {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Thinking…");
                    });
                }
                if self.game.finished() && ui.button("Play again").clicked() {
                    self.rematch();
                }
                ui.add_space(10.);
                ui.label(if self.game.mode == Mode::Computer {
                    format!(
                        "Stockfish · {}\nYou play {}",
                        self.game.difficulty.label(),
                        if self.game.human_white {
                            "white"
                        } else {
                            "black"
                        }
                    )
                } else {
                    "Two players · Shared board".into()
                });
                ui.add_space(16.);
                ui.label(
                    RichText::new("MOVE HISTORY")
                        .small()
                        .color(self.theme.accent),
                );
                egui::ScrollArea::vertical()
                    .id_salt("history")
                    .max_height((ui.available_height() - 300.).max(45.))
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        let live = self.game.moves.len();
                        for i in 0..=live {
                            let label = if i == 0 {
                                "Starting position".to_string()
                            } else {
                                self.game.notation[i - 1].clone()
                            };
                            if ui
                                .selectable_label(
                                    self.preview.unwrap_or(live) == i,
                                    RichText::new(label).color(
                                        if self.preview.unwrap_or(live) == i {
                                            self.theme.accent_text()
                                        } else {
                                            self.theme.foreground
                                        },
                                    ),
                                )
                                .clicked()
                            {
                                self.preview = if i == live { None } else { Some(i) };
                                self.selected = None;
                            }
                        }
                    });
                if self.preview.is_some() && ui.button("Return to live board").clicked() {
                    self.preview = None;
                }
                ui.add_space(8.);
                let response = ui.add_enabled(
                    self.game.human_turn() && self.preview.is_none(),
                    egui::TextEdit::singleline(&mut self.move_text)
                        .hint_text("Move: e4 or e2e4")
                        .desired_width(f32::INFINITY),
                );
                if response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                    match self.game.parse_move(&self.move_text) {
                        Ok(m) => self.accept_move(m),
                        Err(e) => self.message = e,
                    }
                }
                ui.add_space(8.);
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(
                            self.game.human_turn() && !self.engine.busy && self.preview.is_none(),
                            egui::Button::new("Hint"),
                        )
                        .clicked()
                    {
                        self.request_hint();
                    }
                    if ui
                        .add_enabled(self.game.can_takeback(), egui::Button::new("Take back"))
                        .clicked()
                    {
                        self.takeback();
                    }
                });
                if self.engine_error && ui.button("Retry engine").clicked() {
                    self.engine_error = false;
                    self.maybe_engine();
                }
                if self.game.human_turn()
                    && self.game.can_claim_draw()
                    && ui.button("Claim draw").clicked()
                {
                    self.invalidate();
                    match self.game.claim_draw() {
                        Ok(()) => {
                            self.status_message();
                            self.persist();
                        }
                        Err(e) => self.message = e,
                    }
                }
                ui.add_space(8.);
                if ui.button("Flip board   Ctrl+F").clicked() {
                    self.flip();
                }
                if ui.checkbox(&mut self.guides, "Show legal moves").changed() {
                    self.persist();
                }
                if ui.checkbox(&mut self.preferences.sound, "Sound").changed() {
                    self.save_preferences();
                }
            });
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(self.theme.background)
                    .inner_margin(16),
            )
            .show(ctx, |ui| {
                arcade_presentation::backdrop(ui);
                self.draw_board(ui);
            });
        self.dialogs(ctx);
    }
    fn activate(&mut self, square: Square) {
        if self.settings
            || self.new_dialog
            || self.help
            || self.resign_dialog
            || !self.promotion.is_empty()
        {
            return;
        }
        self.cursor = square;
        if self.preview.is_some() || !self.game.human_turn() {
            return;
        }
        if self
            .game
            .position()
            .board()
            .piece_at(square)
            .is_some_and(|p| p.color == self.game.position().turn())
        {
            self.selected = if self.selected == Some(square) {
                None
            } else {
                Some(square)
            };
        } else if let Some(from) = self.selected.take() {
            self.request_move(from, square);
        }
    }
    fn request_move(&mut self, from: Square, to: Square) {
        let candidates = self
            .game
            .position()
            .legal_moves()
            .into_iter()
            .filter(|m| endpoints(m) == (from, to))
            .collect::<Vec<_>>();
        if candidates.len() == 1 {
            self.accept_move(candidates[0].clone());
        } else if candidates.len() > 1 {
            self.promotion = candidates;
        } else {
            self.message = "That move is not legal. Choose another square.".into();
        }
    }
    fn player_label(&self, color: Color) -> String {
        let name = if self.game.mode == Mode::Computer
            && (color == Color::White) != self.game.human_white
        {
            "Stockfish"
        } else if color == Color::White {
            "White"
        } else {
            "Black"
        };
        let captured = self
            .game
            .moves
            .iter()
            .zip(&self.game.positions)
            .filter_map(|(m, p)| {
                if p.turn() == color {
                    m.capture()
                        .map(|r| format!("{} ", r.char().to_ascii_uppercase()))
                } else {
                    None
                }
            })
            .collect::<String>();
        if captured.is_empty() {
            name.into()
        } else {
            format!("{name}    Captured: {captured}")
        }
    }
    fn draw_board(&mut self, ui: &mut egui::Ui) {
        ui.label(self.player_label(if self.flipped {
            Color::White
        } else {
            Color::Black
        }));
        let side = (ui.available_width().min(ui.available_height() - 50.) - 24.).max(160.);
        let (_, outer) = ui.allocate_space(Vec2::new(ui.available_width(), side + 40.));
        let rect = Rect::from_min_size(
            Pos2::new(outer.center().x - side / 2., outer.min.y + 16.),
            Vec2::splat(side),
        );
        arcade_presentation::bezel(ui.painter(), rect, self.theme.accent);
        self.board_rect = Some(rect);
        let response = ui.interact(rect, ui.id().with("board"), Sense::click_and_drag());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Button,
                true,
                "Chess board: arrow keys navigate; Enter selects a piece or destination",
            )
        });
        let position = self.game.positions[self.preview.unwrap_or(self.game.moves.len())].clone();
        let last = self
            .preview
            .unwrap_or(self.game.moves.len())
            .checked_sub(1)
            .and_then(|i| self.game.moves.get(i))
            .map(endpoints);
        let targets = if self.guides && self.preview.is_none() {
            position
                .legal_moves()
                .into_iter()
                .filter(|m| Some(endpoints(m).0) == self.selected)
                .map(|m| endpoints(&m).1)
                .collect::<Vec<_>>()
        } else {
            vec![]
        };
        for square in Square::ALL {
            let cell = square_rect(rect, square, self.flipped);
            let id = ui.id().with(("square", square as u8));
            let square_response = ui.interact(cell, id, Sense::hover());
            let description = position.board().piece_at(square).map_or_else(
                || format!("{square}, empty"),
                |p| {
                    format!(
                        "{square}, {} {}",
                        if p.color == Color::White {
                            "white"
                        } else {
                            "black"
                        },
                        role_name(p.role)
                    )
                },
            );
            square_response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Button,
                    self.game.human_turn(),
                    &description,
                )
            });
            ui.ctx().accesskit_node_builder(id, |node| {
                node.add_action(egui::accesskit::Action::Click);
            });
            if ui.input(|i| i.events.iter().any(|event| matches!(event, egui::Event::AccessKitActionRequest(request) if request.target == egui::accesskit::NodeId(id.value()) && request.action == egui::accesskit::Action::Click))) { self.activate(square); }

            let dark = (square.file() as u8 + square.rank() as u8).is_multiple_of(2);
            ui.painter().rect_filled(cell, 0., self.theme.square(dark));
            arcade_presentation::grain(
                ui.painter(),
                cell,
                Color32::from_black_alpha(if dark { 10 } else { 6 }),
                4.,
            );
            ui.painter().line_segment(
                [cell.left_top(), cell.right_top()],
                Stroke::new(0.6_f32, Color32::from_white_alpha(35)),
            );
            if last.is_some_and(|(a, b)| a == square || b == square) {
                ui.painter().rect_filled(
                    cell,
                    0.,
                    Color32::from_rgba_unmultiplied(220, 211, 106, 100),
                );
            }
            if self.selected == Some(square) {
                ui.painter().rect_filled(
                    cell,
                    0.,
                    Color32::from_rgba_unmultiplied(240, 220, 105, 145),
                );
            }
            if position.is_check() && position.board().king_of(position.turn()) == Some(square) {
                ui.painter().rect_filled(
                    cell,
                    0.,
                    Color32::from_rgba_unmultiplied(220, 72, 60, 150),
                );
            }
            if let Some(piece) = position
                .board()
                .piece_at(square)
                .filter(|_| self.drag_source != Some(square))
            {
                ui.painter().add(egui::Shape::ellipse_filled(
                    cell.center() + Vec2::new(0., cell.height() * 0.32),
                    Vec2::new(cell.width() * 0.27, cell.height() * 0.06),
                    Color32::from_black_alpha(75),
                ));
                egui::Image::new(self.pieces.image(piece.color, piece.role)).paint_at(ui, cell);
            }
            if targets.contains(&square) {
                ui.painter().circle_filled(
                    cell.center(),
                    cell.width() * 0.10,
                    Color32::from_rgba_unmultiplied(20, 35, 25, 120),
                );
            }
            if response.has_focus() && square == self.cursor {
                ui.painter().rect_stroke(
                    cell.shrink(3.),
                    0.,
                    Stroke::new(2_f32, Color32::from_rgb(20, 30, 20)),
                    egui::StrokeKind::Inside,
                );
            }
        }
        if let (Some(from), Some(pointer)) =
            (self.drag_source, ui.input(|i| i.pointer.interact_pos()))
        {
            if let Some(piece) = position.board().piece_at(from) {
                egui::Image::new(self.pieces.image(piece.color, piece.role))
                    .paint_at(ui, Rect::from_center_size(pointer, Vec2::splat(side / 8.)));
            }
        }
        if let Some(m) = &self.hint {
            if self.preview.is_none() {
                let (a, b) = endpoints(m);
                let start = square_rect(rect, a, self.flipped).center();
                let end = square_rect(rect, b, self.flipped).center();
                ui.painter()
                    .arrow(start, end - start, Stroke::new(4_f32, self.theme.accent));
            }
        }
        for i in 0..8 {
            let file = if self.flipped { 7 - i } else { i };
            let rank = if self.flipped { i + 1 } else { 8 - i };
            ui.painter().text(
                Pos2::new(
                    rect.left() + (i as f32 + 0.5) * side / 8.,
                    rect.bottom() + 13.,
                ),
                Align2::CENTER_CENTER,
                ((b'a' + file) as char).to_string(),
                FontId::proportional(12.),
                self.theme.foreground,
            );
            ui.painter().text(
                Pos2::new(rect.left() - 13., rect.top() + (i as f32 + 0.5) * side / 8.),
                Align2::CENTER_CENTER,
                rank.to_string(),
                FontId::proportional(12.),
                self.theme.foreground,
            );
        }
        if response.drag_started() {
            if let Some(pos) = ui.input(|i| i.pointer.press_origin()) {
                self.drag_source = square_at(rect, pos, self.flipped).filter(|sq| {
                    self.game.human_turn()
                        && self.preview.is_none()
                        && position
                            .board()
                            .piece_at(*sq)
                            .is_some_and(|p| p.color == position.turn())
                });
                self.selected = self.drag_source;
            }
        }
        if response.drag_stopped() {
            if let (Some(from), Some(pos)) =
                (self.drag_source.take(), response.interact_pointer_pos())
            {
                if let Some(to) = square_at(rect, pos, self.flipped) {
                    if from != to {
                        self.selected = None;
                        self.request_move(from, to);
                    }
                }
            }
        }
        if response.clicked()
            && ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary))
        {
            response.request_focus();
            ui.ctx().request_repaint();
            if let Some(pos) = response.interact_pointer_pos() {
                if let Some(square) = square_at(rect, pos, self.flipped) {
                    self.activate(square);
                }
            }
        }
        if response.has_focus() {
            ui.memory_mut(|m| {
                m.set_focus_lock_filter(
                    response.id,
                    egui::EventFilter {
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        escape: true,
                        ..Default::default()
                    },
                )
            });
        }
        if response.has_focus()
            && !self.new_dialog
            && !self.help
            && !self.settings
            && !self.resign_dialog
            && self.promotion.is_empty()
        {
            for (key, dx, dy) in [
                (Key::ArrowLeft, -1, 0),
                (Key::ArrowRight, 1, 0),
                (Key::ArrowUp, 0, 1),
                (Key::ArrowDown, 0, -1),
            ] {
                if ui.input(|i| i.key_pressed(key)) {
                    let sign = if self.flipped { -1 } else { 1 };
                    let file = (self.cursor.file() as i32 + dx * sign).clamp(0, 7);
                    let rank = (self.cursor.rank() as i32 + dy * sign).clamp(0, 7);
                    self.cursor = Square::new((rank * 8 + file) as u32);
                }
            }
            if ui.input(|i| i.key_pressed(Key::Enter) || i.key_pressed(Key::Space)) {
                self.activate(self.cursor);
            }
            if ui.input(|i| i.key_pressed(Key::Escape)) {
                self.selected = None;
            }
            let description = position
                .board()
                .piece_at(self.cursor)
                .map_or("empty".into(), |p| {
                    format!(
                        "{} {}",
                        if p.color == Color::White {
                            "white"
                        } else {
                            "black"
                        },
                        role_name(p.role)
                    )
                });
            ui.label(format!("{}: {}", self.cursor, description));
        }
        ui.label(self.player_label(if self.flipped {
            Color::Black
        } else {
            Color::White
        }));
    }
    fn dialogs(&mut self, ctx: &egui::Context) {
        if self.settings {
            let modal = egui::Modal::new(egui::Id::new("Settings")).show(ctx, |ui| {
                ui.set_min_width(330.);
                ui.heading("Settings");
                ui.label("Omarchy Arcade · Chess");
                if ui.checkbox(&mut self.preferences.follow_omarchy, "Follow Omarchy colours").changed() {
                    self.theme = if self.preferences.follow_omarchy { Theme::load() } else { Theme::default() };
                    self.save_preferences();
                }
                let previous_style = self.preferences.piece_style;
                egui::ComboBox::from_label("Chess pieces")
                    .selected_text(self.preferences.piece_style.label())
                    .show_ui(ui, |ui| {
                        for style in [
                            crate::preferences::PieceStyle::AfterHours,
                            crate::preferences::PieceStyle::Chisel,
                        ] {
                            ui.selectable_value(
                                &mut self.preferences.piece_style,
                                style,
                                style.label(),
                            );
                        }
                    });
                if previous_style != self.preferences.piece_style {
                    self.pieces
                        .refresh(ctx, self.theme.accent, self.preferences.piece_style);
                    self.save_preferences();
                }
                if ui.checkbox(&mut self.preferences.sound, "Sound effects").changed() { self.save_preferences(); }
                if self.preferences.sound && !Sound::available() { ui.label("Sound needs paplay (libpulse package). Games remain playable without audio."); }
                ui.separator(); ui.strong("Computer opponent");
                ui.label(if self.engine_path().is_some() { "Stockfish configured" } else { "Stockfish not installed" });
                ui.label("The Chess Arch package includes Stockfish. For source builds, choose a downloaded executable below.");
                if ui.button("Choose Stockfish executable…").clicked() {
                    if let Some(path) = rfd::FileDialog::new().set_title("Choose Stockfish").pick_file() {
                        self.invalidate(); self.preferences.engine_path = Some(path); self.engine_error = false; self.save_preferences();
                    }
                }
                if self.preferences.engine_path.is_some() && ui.button("Use system Stockfish").clicked() { self.invalidate(); self.preferences.engine_path = None; self.engine_error = false; self.save_preferences(); }
                ui.label("Games and settings stay on this computer. Package updates preserve them.");
                if ui.button("Close").clicked() { self.settings = false; }
            });
            if modal.should_close() {
                self.settings = false;
            }
        }

        if self.new_dialog {
            let modal = egui::Modal::new(egui::Id::new("New game")).show(ctx, |ui| {
                ui.heading("New game");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.new_mode, Mode::Computer, "Computer");
                    ui.selectable_value(&mut self.new_mode, Mode::Local, "Friend");
                });
                ui.add_enabled_ui(self.new_mode == Mode::Computer, |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.new_white, true, "White");
                        ui.selectable_value(&mut self.new_white, false, "Black");
                    });
                    egui::ComboBox::from_label("Difficulty")
                        .selected_text(self.new_difficulty.label())
                        .show_ui(ui, |ui| {
                            for d in Difficulty::ALL {
                                ui.selectable_value(&mut self.new_difficulty, d, d.label());
                            }
                        });
                });
                ui.label("Untimed. Your current game will be archived.");
                ui.label("Gentle is reduced engine strength, not a rated beginner bot.");
                ui.horizontal(|ui| {
                    if ui.button("Start game").clicked() {
                        let game = Game {
                            mode: self.new_mode,
                            human_white: self.new_white,
                            difficulty: self.new_difficulty,
                            ..Game::default()
                        };
                        match self.replace_game(game) {
                            Ok(()) => self.new_dialog = false,
                            Err(e) => self.message = e,
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        self.new_dialog = false;
                    }
                });
            });
            if modal.should_close() {
                self.new_dialog = false;
            }
        }
        if !self.promotion.is_empty() {
            let modal = egui::Modal::new(egui::Id::new("Promote pawn")).show(ctx, |ui| {
                ui.heading("Promote pawn");
                for m in self.promotion.clone() {
                    if let Some(role) = m.promotion() {
                        if ui.button(role_name(role)).clicked() {
                            self.accept_move(m);
                        }
                    }
                }
                if ui.button("Cancel").clicked() {
                    self.promotion.clear();
                }
            });
            if modal.should_close() {
                self.promotion.clear();
            }
        }
        if self.resign_dialog {
            let modal = egui::Modal::new(egui::Id::new("Resign game?")).show(ctx, |ui| {
                ui.heading("Resign game?");
                ui.horizontal(|ui| {
                    if ui.button("Resign").clicked() {
                        self.invalidate();
                        self.game.resign();
                        self.resign_dialog = false;
                        self.status_message();
                        self.persist();
                    }
                    if ui.button("Cancel").clicked() {
                        self.resign_dialog = false;
                    }
                });
            });
            if modal.should_close() {
                self.resign_dialog = false;
            }
        }
        if self.help {
            let modal = egui::Modal::new(egui::Id::new("Help / About")).show(ctx,|ui|{
            ui.add(egui::Image::new(egui::include_image!("../packaging/omarchy-chess.svg")).max_size(Vec2::splat(64.)));
            ui.heading("Chess"); ui.label(format!("Omarchy Arcade · Version {}", env!("CARGO_PKG_VERSION"))); ui.label("Powered by Stockfish, shakmaty and egui. GPL-3.0-or-later.");
            ui.label("Board: click or drag; arrow keys, Enter/Space to select, Escape to clear.\nMove field: e4, Nf3, O-O, e2e4 or e7e8n.\nCtrl+, Settings · Ctrl+M Sound · Ctrl+N New · Ctrl+O Import · Ctrl+S Export\nCtrl+Z Takeback · Ctrl+H Hint · Ctrl+F Flip · Ctrl+L Live · Ctrl+Q Quit");
            ui.label("Independent community app. No accounts or network services.\nPieces: Cburnett, adapted by python-chess; GPL artwork, embedded SVGs.");
            if ui.button("Close").clicked(){self.help=false;}
        });
            if modal.should_close() {
                self.help = false;
            }
        }
    }
}
impl eframe::App for ChessApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        arcade_presentation::apply(ctx);
        self.draw(ctx);
    }
    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        self.engine.cancel();
        self.persist();
    }
}
pub fn endpoints(m: &Move) -> (Square, Square) {
    match UciMove::from_move(m, CastlingMode::Standard) {
        UciMove::Normal { from, to, .. } => (from, to),
        _ => unreachable!("standard chess has no drops/null moves"),
    }
}
pub fn square_rect(board: Rect, square: Square, flipped: bool) -> Rect {
    let mut col = square.file() as u8;
    let mut row = 7 - square.rank() as u8;
    if flipped {
        col = 7 - col;
        row = 7 - row;
    }
    let size = board.width() / 8.;
    Rect::from_min_size(
        board.min + Vec2::new(col as f32 * size, row as f32 * size),
        Vec2::splat(size),
    )
}
pub fn square_at(board: Rect, p: Pos2, flipped: bool) -> Option<Square> {
    if !board.contains(p) || p.x == board.right() || p.y == board.bottom() {
        return None;
    }
    let mut col = ((p.x - board.left()) / (board.width() / 8.)) as u8;
    let mut row = ((p.y - board.top()) / (board.width() / 8.)) as u8;
    if flipped {
        col = 7 - col;
        row = 7 - row;
    }
    Some(Square::new(((7 - row) * 8 + col) as u32))
}
fn role_name(role: Role) -> &'static str {
    match role {
        Role::Pawn => "Pawn",
        Role::Knight => "Knight",
        Role::Bishop => "Bishop",
        Role::Rook => "Rook",
        Role::Queen => "Queen",
        Role::King => "King",
    }
}
