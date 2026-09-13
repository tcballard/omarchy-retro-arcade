use crate::{
    game::{self, Dir, Game, Phase, Pos, H, MAP, W},
    packs::{self, Artwork, Choice, PursuerState},
    sound::Sound,
    storage::{self, Appearance, Save},
    theme::{self, Palette},
};
use eframe::egui::{
    self, Align2, Color32, FontId, Key, Modifiers, Pos2, Rect, RichText, Shape, Stroke, Vec2,
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

const PAD: f32 = 24.;
#[derive(Clone, Copy, PartialEq, Eq)]
enum Dialog {
    Settings,
    Help,
    About,
    NewGame,
}
pub struct ScramApp {
    pub save: Save,
    dir: PathBuf,
    palette: Palette,
    paused: bool,
    dialog: Option<Dialog>,
    menu_open: bool,
    notice: String,
    writable: bool,
    clock: Instant,
    save_clock: Instant,
    theme_clock: Instant,
    accumulator: f32,
    sound: Sound,
    screenshot: Option<PathBuf>,
    frames: u32,
    capture_after: Instant,
    capture_sent: bool,
    artwork: Option<Artwork>,
    pack_choices: Vec<Choice>,
    pack_message: String,
}
impl ScramApp {
    pub fn prepare_to_leave(&mut self) -> Result<(), String> {
        self.paused = true;
        self.accumulator = 0.;
        if !self.writable {
            return Err("The original Scram save needs recovery before saving is allowed.".into());
        }
        self.save.write(&self.dir).map_err(|e| e.to_string())
    }
    pub fn new(ctx: &egui::Context, dir: PathBuf, screenshot: Option<PathBuf>) -> Self {
        egui_extras::install_image_loaders(ctx);
        let loaded = storage::load(&dir);
        let palette = Palette::load(loaded.save.settings.appearance);
        let paused = loaded.save.game.phase.active();
        let mut app = Self {
            save: loaded.save,
            dir,
            palette,
            paused,
            dialog: None,
            menu_open: false,
            notice: loaded.notice,
            writable: loaded.writable,
            clock: Instant::now(),
            save_clock: Instant::now(),
            theme_clock: Instant::now(),
            accumulator: 0.,
            sound: Sound::default(),
            screenshot,
            frames: 0,
            capture_after: Instant::now() + Duration::from_millis(1500),
            capture_sent: false,
            artwork: None,
            pack_choices: packs::choices_at(&packs::directory()),
            pack_message: String::new(),
        };
        let selected = app.save.settings.character_pack.clone();
        if !app.apply_pack(ctx, &selected) {
            let error = app.pack_message.clone();
            app.apply_pack(ctx, packs::LATCH);
            app.notice = format!("{} Characters: {error} Using Latch.", app.notice)
                .trim()
                .into();
        }
        app.style(ctx);
        app.persist();
        app
    }
    fn apply_pack(&mut self, ctx: &egui::Context, key: &str) -> bool {
        match packs::load_choice_at(key, &packs::directory()) {
            Ok(data) => {
                let artwork = data.map(|data| Artwork::upload(ctx, data));
                if let Some(art) = &artwork {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Icon(Some(std::sync::Arc::new(
                        art.icon_data.clone(),
                    ))));
                } else if let Ok(image) = egui_extras::image::load_svg_bytes_with_size(
                    include_bytes!("../assets/original.svg"),
                    Some(egui::load::SizeHint::Size(128, 128)),
                ) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Icon(Some(std::sync::Arc::new(
                        egui::IconData {
                            rgba: image.pixels.iter().flat_map(|p| p.to_array()).collect(),
                            width: image.width() as u32,
                            height: image.height() as u32,
                        },
                    ))));
                }
                self.artwork = artwork;
                self.save.settings.character_pack = key.into();
                self.pack_message.clear();
                true
            }
            Err(error) => {
                self.pack_message = error;
                false
            }
        }
    }
    fn character_settings(&mut self, ui: &mut egui::Ui) {
        ui.label("Characters");
        let current = self.save.settings.character_pack.clone();
        let mut selected = current.clone();
        let label = self
            .artwork
            .as_ref()
            .map_or("Original", |a| a.name.as_str());
        // A new list length resets egui's cached popup size after pack discovery.
        egui::ComboBox::from_id_salt(("character-pack", self.pack_choices.len()))
            .selected_text(label)
            .width(260.)
            .truncate()
            .show_ui(ui, |ui| {
                ui.set_max_width(320.);
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                for choice in &self.pack_choices {
                    let response = ui.add_enabled(
                        choice.error.is_none(),
                        egui::SelectableLabel::new(selected == choice.key, &choice.name),
                    );
                    if response.clicked() {
                        selected = choice.key.clone();
                    }
                    if let Some(error) = &choice.error {
                        response.on_hover_text(error);
                    }
                }
            });
        if selected != current {
            self.apply_pack(ui.ctx(), &selected);
        }
        if let Some(art) = &self.artwork {
            art.preview(ui);
        } else {
            let (r, _) = ui.allocate_exact_size(Vec2::new(240., 56.), egui::Sense::hover());
            draw_player(
                ui.painter(),
                r.left_center() + Vec2::new(26., 0.),
                18.,
                Dir::Right,
                0.42,
                self.palette.player,
                self.palette.surface,
            );
            for (i, ghost) in self.save.game.pursuers.iter().enumerate() {
                draw_pursuer(
                    ui.painter(),
                    r.left_center() + Vec2::new(78. + i as f32 * 45., 0.),
                    16.,
                    i,
                    false,
                    ghost,
                    &self.palette,
                );
            }
        }
        ui.horizontal(|ui| {
            if ui
                .button("Reload packs")
                .on_hover_text("Find new packs and reload the selected artwork.")
                .clicked()
            {
                self.pack_choices = packs::choices_at(&packs::directory());
                let key = self.save.settings.character_pack.clone();
                if self.apply_pack(ui.ctx(), &key) {
                    self.pack_message = "Artwork reloaded.".into();
                }
            }
            if ui
                .button("Create editable copy")
                .on_hover_text("Export the Latch PNGs and pack.toml to a new folder.")
                .clicked()
            {
                match packs::export_template(&packs::directory()) {
                    Ok(path) => {
                        self.pack_choices = packs::choices_at(&packs::directory());
                        self.pack_message = format!(
                            "Created {}. Replace its PNGs, then reload packs.",
                            path.display()
                        );
                    }
                    Err(error) => self.pack_message = error,
                }
            }
        });
        if ui.button("Open packs folder").clicked() {
            let result = std::fs::create_dir_all(packs::directory()).and_then(|()| {
                std::process::Command::new("xdg-open")
                    .arg(packs::directory())
                    .spawn()
                    .map(|mut child| {
                        std::thread::spawn(move || {
                            let _ = child.wait();
                        });
                    })
            });
            if let Err(error) = result {
                self.pack_message = format!(
                    "Open {} in your file manager. {error}",
                    packs::directory().display()
                );
            }
        }
        ui.label(RichText::new("Switch anytime. Your run and scores stay.").small());
        if !self.pack_message.is_empty() {
            ui.label(RichText::new(&self.pack_message).small());
        }
        ui.separator();
    }
    fn style(&self, ctx: &egui::Context) {
        let mut visuals = if theme::contrast(self.palette.background, Color32::WHITE)
            > theme::contrast(self.palette.background, Color32::BLACK)
        {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.panel_fill = self.palette.background;
        visuals.window_fill = self.palette.surface;
        visuals.override_text_color = Some(self.palette.text);
        visuals.selection.bg_fill = self.palette.accent;
        visuals.selection.stroke = Stroke::new(
            1_f32,
            theme::readable(self.palette.text, self.palette.accent, 4.5),
        );
        visuals.widgets.inactive.bg_fill =
            theme::blend(self.palette.background, self.palette.text, 0.08);
        visuals.widgets.inactive.weak_bg_fill = visuals.widgets.inactive.bg_fill;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1_f32, self.palette.text);
        visuals.widgets.hovered.bg_fill =
            theme::blend(self.palette.background, self.palette.text, 0.17);
        visuals.widgets.active.bg_fill =
            theme::blend(self.palette.background, self.palette.text, 0.24);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1_f32, self.palette.text);
        visuals.widgets.active.fg_stroke = Stroke::new(1_f32, self.palette.text);
        visuals.window_stroke = Stroke::new(1_f32, self.palette.wall);
        ctx.set_visuals(visuals);
        arcade_presentation::apply(ctx);
        ctx.style_mut(|s| {
            s.spacing.item_spacing = Vec2::new(10., 8.);
            s.spacing.button_padding = Vec2::new(12., 7.);
            s.spacing.interact_size.y = 30.;
        });
    }
    fn persist(&mut self) {
        if self.writable {
            if let Err(e) = self.save.write(&self.dir) {
                self.notice=format!("Your game could not be saved: {e}. Free space or check folder permissions; saving will retry.");
            }
        }
        self.save_clock = Instant::now();
    }
    fn restart(&mut self) {
        if self.writable {
            if let Err(e) = self.save.restart(&self.dir) {
                self.notice=format!("Could not preserve the previous game: {e}. Your current game is still available.");
                return;
            }
        } else {
            self.save.record();
            self.save.game = Game::new(self.save.settings.relaxed);
        }
        self.paused = false;
        self.accumulator = 0.;
        self.dialog = None;
        self.clock = Instant::now();
    }
    fn show_dialog(&mut self, dialog: Dialog) {
        self.dialog = Some(dialog);
        if self.save.game.phase.active() {
            self.paused = true;
        }
        self.persist();
    }
    fn toggle_pause(&mut self) {
        if matches!(
            self.save.game.phase,
            Phase::Playing | Phase::Dying | Phase::Cleared
        ) {
            self.paused = !self.paused;
            self.accumulator = 0.;
            self.clock = Instant::now();
            self.persist();
        }
    }
    fn keyboard(&mut self, ctx: &egui::Context) {
        let command = |key| ctx.input_mut(|i| i.consume_key(Modifiers::CTRL, key));
        if command(Key::Q) {
            self.persist();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if command(Key::N) {
            self.show_dialog(Dialog::NewGame);
        }
        if command(Key::Comma) {
            self.show_dialog(Dialog::Settings);
        }
        if command(Key::M) {
            self.save.settings.sound = !self.save.settings.sound;
            self.persist();
        }
        if ctx.input(|i| i.key_pressed(Key::F1)) {
            self.show_dialog(Dialog::Help);
        }
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            if self.dialog.take().is_none() && !self.menu_open {
                self.toggle_pause();
            }
            return;
        }
        if self.dialog.is_some() || self.menu_open || ctx.memory(|m| m.any_popup_open()) {
            return;
        }
        if ctx.input(|i| i.key_pressed(Key::Space) || i.key_pressed(Key::P)) {
            self.toggle_pause();
        }
        if self.paused {
            return;
        }
        let mut wanted = None;
        ctx.input(|input| {
            for event in &input.events {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                {
                    if modifiers.ctrl || modifiers.alt || modifiers.command {
                        continue;
                    }
                    wanted = match key {
                        Key::ArrowUp | Key::W | Key::K => Some(Dir::Up),
                        Key::ArrowLeft | Key::A | Key::H => Some(Dir::Left),
                        Key::ArrowDown | Key::S | Key::J => Some(Dir::Down),
                        Key::ArrowRight | Key::D | Key::L => Some(Dir::Right),
                        _ => wanted,
                    };
                }
            }
        });
        if let Some(dir) = wanted {
            self.save.game.input(dir);
        }
    }
    fn menu(&mut self, ctx: &egui::Context) {
        let mut menu_open = false;
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Game", |ui| {
                    menu_open = true;
                    if matches!(
                        self.save.game.phase,
                        Phase::Playing | Phase::Dying | Phase::Cleared
                    ) {
                        self.paused = true;
                    }
                    if ui.button("New game                 Ctrl+N").clicked() {
                        self.show_dialog(Dialog::NewGame);
                        ui.close_menu();
                    }
                    if ui
                        .add_enabled(
                            matches!(
                                self.save.game.phase,
                                Phase::Playing | Phase::Dying | Phase::Cleared
                            ),
                            egui::Button::new(if self.paused {
                                "Resume                         Space"
                            } else {
                                "Pause                            Space"
                            }),
                        )
                        .clicked()
                    {
                        self.toggle_pause();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .button("Quit                            Ctrl+Q")
                        .clicked()
                    {
                        self.persist();
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Settings", |ui| {
                    menu_open = true;
                    if matches!(
                        self.save.game.phase,
                        Phase::Playing | Phase::Dying | Phase::Cleared
                    ) {
                        self.paused = true;
                    }
                    if ui
                        .checkbox(&mut self.save.settings.sound, "Sound cues     Ctrl+M")
                        .changed()
                    {
                        self.persist();
                    }
                    if ui.button("Appearance & play…   Ctrl+,").clicked() {
                        self.show_dialog(Dialog::Settings);
                        ui.close_menu();
                    }
                });
                ui.menu_button("Help", |ui| {
                    menu_open = true;
                    if matches!(
                        self.save.game.phase,
                        Phase::Playing | Phase::Dying | Phase::Cleared
                    ) {
                        self.paused = true;
                    }
                    if ui.button("How to play             F1").clicked() {
                        self.show_dialog(Dialog::Help);
                        ui.close_menu();
                    }
                    if ui.button("About Omarchy Scram").clicked() {
                        self.show_dialog(Dialog::About);
                        ui.close_menu();
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("New game").clicked() {
                        self.show_dialog(Dialog::NewGame);
                    }
                    if ui
                        .add_enabled(
                            matches!(
                                self.save.game.phase,
                                Phase::Playing | Phase::Dying | Phase::Cleared
                            ),
                            egui::Button::new(if self.paused { "Resume" } else { "Pause" }),
                        )
                        .clicked()
                    {
                        self.toggle_pause();
                    }
                });
            });
        });
        self.menu_open = menu_open;
    }
    fn board(&mut self, ui: &mut egui::Ui) {
        let p = self.palette.clone();
        let available = ui.available_rect_before_wrap();
        let tile = ((available.width() - PAD * 2.) / W as f32)
            .min((available.height() - 186.) / H as f32)
            .max(10.);
        let board_size = Vec2::new(tile * W as f32, tile * H as f32);
        let left = available.center().x - board_size.x / 2.;
        let top = available.top() + 123.;
        let board = Rect::from_min_size(Pos2::new(left, top), board_size);
        let painter = ui.painter();
        painter.text(
            Pos2::new(left, available.top() + 8.),
            Align2::LEFT_TOP,
            "SCRAM",
            FontId::proportional(30.),
            p.text,
        );
        painter.text(
            Pos2::new(left, available.top() + 43.),
            Align2::LEFT_TOP,
            "OMARCHY ARCADE",
            FontId::monospace(10.),
            p.muted,
        );
        let mode = if self.save.game.relaxed {
            "RELAXED"
        } else {
            "STANDARD"
        };
        painter.text(
            Pos2::new(board.right(), available.top() + 21.),
            Align2::RIGHT_CENTER,
            mode,
            FontId::monospace(11.),
            p.accent,
        );
        let stats = [
            ("SCORE", format!("{:06}", self.save.game.score)),
            (
                "BEST",
                format!(
                    "{:06}",
                    self.save.best[self.save.game.relaxed as usize].max(self.save.game.score)
                ),
            ),
            ("LEVEL", format!("{:02}", self.save.game.level)),
            ("LIVES", self.save.game.lives.to_string()),
        ];
        for (i, (label, value)) in stats.iter().enumerate() {
            let x = left + i as f32 * board_size.x / 4.;
            painter.text(
                Pos2::new(x, available.top() + 72.),
                Align2::LEFT_TOP,
                label,
                FontId::monospace(10.),
                p.muted,
            );
            painter.text(
                Pos2::new(x, available.top() + 88.),
                Align2::LEFT_TOP,
                value,
                FontId::monospace((board_size.x / 24.).clamp(13., 22.)),
                if i == 0 { p.accent } else { p.text },
            );
        }
        arcade_presentation::bezel(painter, board, p.accent);
        painter.rect_filled(board, 0., p.field);
        let cell = |pos: Pos| {
            Rect::from_min_size(
                board.min + Vec2::new(pos.x as f32 * tile, pos.y as f32 * tile),
                Vec2::splat(tile),
            )
        };
        let maze_painter = painter.with_clip_rect(board);
        for y in 0..H {
            for x in 0..W {
                let pos = Pos { x, y };
                let r = cell(pos);
                if MAP[y as usize].as_bytes()[x as usize] == b'#' {
                    maze_painter.rect_filled(r, 0., p.wall_fill);
                    arcade_presentation::grain(&maze_painter, r, Color32::from_black_alpha(12), 4.);
                    for dir in Dir::ALL {
                        let (dx, dy) = dir.delta();
                        let neighbor = Pos {
                            x: x + dx,
                            y: y + dy,
                        };
                        if neighbor.open() {
                            let points = match dir {
                                Dir::Up => [r.left_top(), r.right_top()],
                                Dir::Down => [r.left_bottom(), r.right_bottom()],
                                Dir::Left => [r.left_top(), r.left_bottom()],
                                Dir::Right => [r.right_top(), r.right_bottom()],
                            };
                            maze_painter.line_segment(
                                points,
                                Stroke::new((tile * 0.26).max(2.), p.wall.gamma_multiply(0.22)),
                            );
                            maze_painter
                                .line_segment(points, Stroke::new((tile * 0.10).max(1.), p.wall));
                            maze_painter.line_segment(
                                points,
                                Stroke::new(0.6_f32, p.wall.lerp_to_gamma(Color32::WHITE, 0.35)),
                            );
                        }
                    }
                } else {
                    match self.save.game.pellets[pos.index()] {
                        1 => {
                            maze_painter.rect_filled(
                                Rect::from_center_size(
                                    r.center(),
                                    Vec2::splat((tile * 0.105).max(2.)),
                                ),
                                0.4,
                                p.dot,
                            );
                        }
                        2 => {
                            let radius = tile * 0.24;
                            let c = r.center();
                            maze_painter.add(Shape::convex_polygon(
                                vec![
                                    c + Vec2::new(0., -radius),
                                    c + Vec2::new(radius, 0.),
                                    c + Vec2::new(0., radius),
                                    c + Vec2::new(-radius, 0.),
                                ],
                                p.accent,
                                Stroke::NONE,
                            ));
                            maze_painter.circle_stroke(
                                c,
                                tile * 0.37,
                                Stroke::new(1_f32, p.accent),
                            );
                        }
                        _ => (),
                    }
                }
            }
        }
        // Portal markers show that the side corridor wraps, without extra UI.
        for x in [0., W as f32 * tile] {
            let c = board.min + Vec2::new(x, (game::TUNNEL as f32 + 0.5) * tile);
            maze_painter.line_segment(
                [
                    c - Vec2::new(0., tile * 0.35),
                    c + Vec2::new(0., tile * 0.35),
                ],
                Stroke::new(3_f32, p.accent),
            );
        }
        for (i, ghost) in self.save.game.pursuers.iter().enumerate() {
            let (x, y) = ghost.mover.xy();
            let c = board.min + Vec2::new((x + 0.5) * tile, (y + 0.5) * tile);
            for dx in [-W as f32 * tile, 0., W as f32 * tile] {
                if let Some(art) = &self.artwork {
                    let state = if ghost.returning {
                        PursuerState::Returning
                    } else if self.save.game.power > 0. {
                        PursuerState::Vulnerable
                    } else {
                        PursuerState::Normal
                    };
                    art.pursuer(&maze_painter, c + Vec2::new(dx, 0.), tile, i, state);
                } else {
                    draw_pursuer(
                        &maze_painter,
                        c + Vec2::new(dx, 0.),
                        tile * 0.40,
                        i,
                        self.save.game.power > 0. && !ghost.returning,
                        ghost,
                        &p,
                    );
                }
            }
        }
        let (x, y) = self.save.game.player.xy();
        let c = board.min + Vec2::new((x + 0.5) * tile, (y + 0.5) * tile);
        let moving = self.save.game.phase == Phase::Playing
            && !self.paused
            && !self.save.settings.reduce_motion;
        let mouth = if moving {
            0.22 + (self.save.game.elapsed * 18.).sin().abs() * 0.43
        } else {
            0.42
        };
        let size = if self.save.game.phase == Phase::Dying && !self.save.settings.reduce_motion {
            (self.save.game.phase_time / 1.3).clamp(0., 1.)
        } else {
            1.
        };
        for dx in [-W as f32 * tile, 0., W as f32 * tile] {
            if let Some(art) = &self.artwork {
                art.player(
                    &maze_painter,
                    c + Vec2::new(dx, 0.),
                    tile * size,
                    self.save.game.player.dir,
                    self.save.game.elapsed,
                    moving,
                );
            } else {
                draw_player(
                    &maze_painter,
                    c + Vec2::new(dx, 0.),
                    tile * 0.42 * size,
                    self.save.game.player.dir,
                    mouth,
                    p.player,
                    p.field,
                );
            }
        }
        let power = self.save.game.power;
        let foot = board.bottom() + 17.;
        if power > 0. {
            let label = format!("POWER   {:.1}s", power);
            painter.text(
                Pos2::new(left, foot),
                Align2::LEFT_TOP,
                &label,
                FontId::monospace(12.),
                p.accent,
            );
            let bar = Rect::from_min_size(
                Pos2::new(left + 140., foot + 4.),
                Vec2::new((board_size.x - 142.).max(30.), 5.),
            );
            painter.rect_filled(bar, 0., p.wall_fill);
            painter.rect_filled(
                Rect::from_min_size(
                    bar.min,
                    Vec2::new(
                        bar.width() * (power / self.save.game.power_duration()),
                        bar.height(),
                    ),
                ),
                0.,
                p.accent,
            );
        } else {
            painter.text(
                Pos2::new(left, foot),
                Align2::LEFT_TOP,
                format!("{} DOTS LEFT", self.save.game.remaining()),
                FontId::monospace(11.),
                p.muted,
            );
            painter.text(
                Pos2::new(board.right(), foot),
                Align2::RIGHT_TOP,
                "ARROWS / WASD    SPACE PAUSES",
                FontId::monospace(if tile < 18. { 9. } else { 11. }),
                p.muted,
            );
        }
        let overlay = if self.paused {
            Some(("PAUSED", "Space to resume. Your place is safe."))
        } else {
            match self.save.game.phase {
                Phase::Ready => Some((
                    if self.save.game.level == 1
                        && self.save.game.lives == 3
                        && self.save.game.score == 0
                    {
                        "READY?"
                    } else {
                        "READY AGAIN?"
                    },
                    "Press a direction to move.",
                )),
                Phase::Dying => Some((
                    "CAUGHT",
                    if self.save.game.lives > 0 {
                        "Take a breath. Another life is ready."
                    } else {
                        "One last dot too far."
                    },
                )),
                Phase::Cleared => Some(("MAZE CLEAR", "Next level. A little quicker.")),
                Phase::GameOver => Some(("GAME OVER", "One more run?")),
                _ => None,
            }
        };
        if let Some((title, subtitle)) = overlay {
            let box_size = Vec2::new(
                (board_size.x - 24.).min(360.),
                if self.save.game.phase == Phase::GameOver {
                    152.
                } else {
                    106.
                },
            );
            let box_rect = Rect::from_center_size(board.center(), box_size);
            painter.rect_filled(box_rect, 3., p.background);
            painter.rect_stroke(
                box_rect,
                3.,
                Stroke::new(1.5_f32, p.accent),
                egui::StrokeKind::Inside,
            );
            painter.text(
                box_rect.center_top() + Vec2::new(0., 21.),
                Align2::CENTER_TOP,
                title,
                FontId::monospace(24.),
                p.accent,
            );
            painter.text(
                box_rect.center_top() + Vec2::new(0., 60.),
                Align2::CENTER_TOP,
                subtitle,
                FontId::proportional(13.),
                p.text,
            );
            if self.save.game.phase == Phase::GameOver && !self.paused {
                let r = Rect::from_center_size(
                    box_rect.center_bottom() - Vec2::new(0., 32.),
                    Vec2::new(140., 32.),
                );
                if ui.put(r, egui::Button::new("Play again")).clicked() {
                    self.restart();
                }
            }
        }
        // A semantic summary supplements the graphical playfield and standard controls.
        let label = format!(
            "Maze. Score {}. Level {}. {} lives. {} dots remaining. {:?}.",
            self.save.game.score,
            self.save.game.level,
            self.save.game.lives,
            self.save.game.remaining(),
            self.save.game.phase
        );
        ui.interact(board, ui.id().with("maze"), egui::Sense::hover())
            .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, &label));
        ui.allocate_rect(available, egui::Sense::hover());
    }
    fn dialogs(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.dialog else {
            return;
        };
        let title = match dialog {
            Dialog::Settings => "Appearance & play",
            Dialog::Help => "How to play",
            Dialog::About => "About Omarchy Scram",
            Dialog::NewGame => "Start a new game?",
        };
        let mut open = true;
        let mut close = false;
        let mut restart = false;
        let before = self.save.settings.clone();
        egui::Window::new(title).open(&mut open).collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER,Vec2::ZERO).default_width(380.).max_height(ctx.screen_rect().height()-80.).vscroll(true).show(ctx,|ui|{
            match dialog{
                Dialog::Settings=>{
                    self.character_settings(ui);
                    ui.label("Appearance");
                    ui.radio_value(&mut self.save.settings.appearance,Appearance::Omarchy,"Follow Omarchy");
                    ui.horizontal(|ui|{ui.radio_value(&mut self.save.settings.appearance,Appearance::Charcoal,"Charcoal");ui.radio_value(&mut self.save.settings.appearance,Appearance::Ivory,"Ivory");});
                    ui.separator();ui.checkbox(&mut self.save.settings.sound,"Sound cues");
                    if !Sound::available(){ui.label(RichText::new("Sound needs paplay (libpulse on Arch). Play remains silent until it is available.").small());}
                    ui.checkbox(&mut self.save.settings.reduce_motion,"Reduce animation");
                    ui.label(RichText::new("Keeps movement readable; removes the chewing and shrinking animations.").small());
                    ui.separator();ui.checkbox(&mut self.save.settings.relaxed,"Relaxed pace for new games");
                    ui.label(RichText::new("25% slower. Separate high score. Takes effect with New game.").small());
                    ui.add_space(8.);ui.label(format!("Current palette: {}",self.palette.name));
                }
                Dialog::Help=>{
                    ui.label("Clear the dots. Avoid the four pursuers.");ui.add_space(8.);
                    egui::Grid::new("keys").spacing([24.,10.]).show(ui,|ui|{for (key,action) in [("Arrows / WASD / HJKL","Move; turns are buffered"),("Space / P / Esc","Pause or resume"),("Ctrl+N","New game"),("Ctrl+M","Toggle sound"),("Ctrl+,","Settings"),("Ctrl+Q","Save and quit")]{ui.monospace(key);ui.label(action);ui.end_row();}});
                    ui.separator();ui.label("Dots: 10 points. Power diamonds: 50 points and temporary capture power. Captures: 200, 400, 800, then 1,600 points.");
                    ui.label("A life every 10,000 points, up to nine. The side tunnel wraps. Clear a maze to advance; each level gets faster and power gets shorter.");
                    ui.label("The pursuers chase, ambush, flank or retreat nearby. Their silhouettes stay distinct when theme colours change. The power timer shows exactly how long you have.");
                    ui.label("Opening menus or switching windows pauses play. Resume explicitly with Space. A saved run also opens paused.");
                }
                Dialog::About=>{
                    ui.horizontal(|ui|{if let Some(art) = &self.artwork { ui.image((art.icon.id(),Vec2::splat(64.))); } else {ui.add(egui::Image::new(egui::include_image!("../assets/original.svg")).fit_to_exact_size(Vec2::splat(64.)));}ui.vertical(|ui|{ui.heading("Omarchy Scram");ui.label("Omarchy Arcade");ui.label(concat!("Version ",env!("CARGO_PKG_VERSION")));});});
                    ui.separator();ui.label("An original, offline maze chase for the community Arcade collection. Built in Rust, with original maze artwork and synthesized sound.");
                    ui.label("MIT licensed. Interface: egui / eframe (MIT or Apache-2.0). Full third-party notices ship with the source.");
                    if let Some(art) = &self.artwork {ui.label(format!("Characters: {}\n{}\nPack licence: {}",art.name,art.author,art.license));}
                    ui.label("Community project. Not an official Omarchy or PAC-MAN release.");
                }
                Dialog::NewGame=>{ui.label("The current run will be kept as your previous session. Your best scores and settings stay.");if ui.button("Start new game").clicked(){restart=true;}}
            }
            ui.add_space(8.);if ui.button(if dialog==Dialog::NewGame{"Keep playing"}else{"Done"}).clicked(){close=true;}
        });
        if self.save.settings != before {
            self.palette = Palette::load(self.save.settings.appearance);
            self.style(ctx);
            self.persist();
        }
        if close || !open {
            self.dialog = None;
        }
        if restart {
            self.restart();
        }
    }
    fn capture(&mut self, ctx: &egui::Context) {
        if self.screenshot.is_none() {
            return;
        }
        self.frames += 1;
        if self.frames >= 8 && Instant::now() >= self.capture_after && !self.capture_sent {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            self.capture_sent = true;
        }
        let screenshot = ctx.input(|i| {
            i.events.iter().find_map(|e| {
                if let egui::Event::Screenshot { image, .. } = e {
                    Some(image.clone())
                } else {
                    None
                }
            })
        });
        if let (Some(image), Some(path)) = (screenshot, self.screenshot.as_ref()) {
            let bytes: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
            if let Err(e) = image::save_buffer(
                path,
                &bytes,
                image.width() as u32,
                image.height() as u32,
                image::ColorType::Rgba8,
            ) {
                eprintln!("Screenshot failed: {e}");
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ctx.request_repaint();
    }
}
impl eframe::App for ScramApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        arcade_presentation::apply(ctx);
        let now = Instant::now();
        let dt = now.duration_since(self.clock).as_secs_f32().min(0.1);
        self.clock = now;
        if !ctx.input(|i| i.focused) && self.save.game.phase.active() && !self.paused {
            self.paused = true;
            self.persist();
        }
        self.keyboard(ctx);
        self.menu(ctx);
        if ctx.memory(|m| m.any_popup_open()) && self.save.game.phase == Phase::Playing {
            self.paused = true;
        }
        if !self.paused && self.dialog.is_none() {
            self.accumulator += dt;
            while self.accumulator >= game::STEP {
                self.accumulator -= game::STEP;
                let events = self.save.game.tick();
                if self.save.settings.sound {
                    for event in &events {
                        self.sound.play(*event);
                    }
                }
                if events.iter().any(|e| *e != game::Event::Dot) {
                    self.persist();
                }
            }
        } else {
            self.accumulator = 0.;
        }
        self.save.record();
        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(&self.palette.name)
                        .small()
                        .color(self.palette.muted),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .small_button(if self.save.settings.sound {
                            "Sound on"
                        } else {
                            "Sound off"
                        })
                        .clicked()
                    {
                        self.save.settings.sound = !self.save.settings.sound;
                        self.persist();
                    }
                });
            });
            if !self.notice.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    ui.label(&self.notice);
                    if ui.small_button("Dismiss").clicked() {
                        self.notice.clear();
                    }
                });
            }
        });
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(self.palette.background)
                    .inner_margin(8.),
            )
            .show(ctx, |ui| {
                arcade_presentation::backdrop(ui);
                self.board(ui);
            });
        self.dialogs(ctx);
        if self.save_clock.elapsed() > Duration::from_secs(2) {
            self.persist();
        }
        if self.theme_clock.elapsed() > Duration::from_secs(2) {
            let p = Palette::load(self.save.settings.appearance);
            if p != self.palette {
                self.palette = p;
                self.style(ctx);
            }
            self.theme_clock = Instant::now();
        }
        self.capture(ctx);
        if !self.paused
            && matches!(
                self.save.game.phase,
                Phase::Playing | Phase::Dying | Phase::Cleared
            )
        {
            ctx.request_repaint_after(Duration::from_millis(8));
        } else {
            ctx.request_repaint_after(Duration::from_millis(200));
        }
    }
    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        self.persist();
    }
}

fn draw_player(
    p: &egui::Painter,
    c: Pos2,
    r: f32,
    dir: Dir,
    mouth: f32,
    color: Color32,
    ink: Color32,
) {
    let angle = match dir {
        Dir::Right => 0.,
        Dir::Down => std::f32::consts::FRAC_PI_2,
        Dir::Left => std::f32::consts::PI,
        Dir::Up => -std::f32::consts::FRAC_PI_2,
    };
    let point = |a: f32| c + Vec2::angled(a + angle) * r;
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(c, color);
    for i in 0..=18 {
        let a = mouth + (std::f32::consts::TAU - 2. * mouth) * i as f32 / 18.;
        mesh.colored_vertex(point(a), color);
    }
    for i in 0..18 {
        mesh.add_triangle(0, i + 1, i + 2);
    }
    p.add(Shape::mesh(mesh));
    let eye = c + Vec2::angled(angle - 0.92) * r * 0.58;
    p.rect_filled(
        Rect::from_center_size(eye, Vec2::splat((r * 0.16).max(1.))),
        0.,
        ink,
    );
}
fn draw_pursuer(
    p: &egui::Painter,
    c: Pos2,
    r: f32,
    id: usize,
    powered: bool,
    ghost: &game::Pursuer,
    palette: &Palette,
) {
    let returning = ghost.returning;
    let dir = ghost.mover.dir;
    let color = if powered {
        palette.accent
    } else {
        palette.pursuers[id]
    };
    if !returning {
        // Four crowns identify the pursuers independently of their colours.
        let crown = match id {
            0 => vec![(-0.8, -0.55), (-0.4, -0.95), (0.4, -0.95), (0.8, -0.55)],
            1 => vec![(-0.8, -0.5), (0., -1.15), (0.8, -0.5)],
            2 => vec![(-0.9, -0.55), (-0.55, -0.95), (0.55, -0.95), (0.9, -0.55)],
            _ => vec![(-0.8, -0.55), (-0.8, -0.95), (0.8, -0.95), (0.8, -0.55)],
        };
        let mut points = vec![c + Vec2::new(-r, r * 0.65)];
        points.extend(crown.into_iter().map(|(x, y)| c + Vec2::new(x * r, y * r)));
        points.push(c + Vec2::new(r, r * 0.65));
        p.add(Shape::convex_polygon(points, color, Stroke::NONE));
        for dx in [-0.7, 0., 0.7] {
            p.rect_filled(
                Rect::from_center_size(
                    c + Vec2::new(dx * r, r * 0.7),
                    Vec2::new(r * 0.37, r * 0.35),
                ),
                0.,
                color,
            );
        }
        if id == 2 {
            p.line_segment(
                [
                    c + Vec2::new(-0.42 * r, -0.65 * r),
                    c + Vec2::new(0.42 * r, -0.65 * r),
                ],
                Stroke::new((r * 0.16).max(1.), palette.field),
            );
        }
    }
    let (dx, dy) = dir.delta();
    for side in [-1., 1.] {
        let eye = c + Vec2::new(side * r * 0.35, -r * 0.05);
        if powered && !returning {
            p.line_segment(
                [
                    eye + Vec2::new(-r * 0.14, -r * 0.10),
                    eye + Vec2::new(r * 0.14, r * 0.10),
                ],
                Stroke::new((r * 0.14).max(1.), palette.field),
            );
        } else {
            p.rect_filled(
                Rect::from_center_size(eye, Vec2::new(r * 0.45, r * 0.52)),
                r * 0.08,
                palette.dot,
            );
            p.rect_filled(
                Rect::from_center_size(
                    eye + Vec2::new(dx as f32 * r * 0.09, dy as f32 * r * 0.09),
                    Vec2::splat(r * 0.22),
                ),
                0.,
                palette.field,
            );
        }
    }
    if powered && !returning {
        p.line_segment(
            [
                c + Vec2::new(-r * 0.27, r * 0.39),
                c + Vec2::new(r * 0.27, r * 0.39),
            ],
            Stroke::new((r * 0.12).max(1.), palette.field),
        );
    }
}

#[cfg(test)]
mod pack_tests {
    use super::*;
    #[test]
    fn switching_and_failed_switch_keep_run_and_records() {
        let ctx = egui::Context::default();
        let dir = tempfile::tempdir().unwrap();
        let mut app = ScramApp::new(&ctx, dir.path().into(), None);
        app.save.game.score = 1230;
        app.save.best = [5000, 800];
        let game = app.save.game.clone();
        let best = app.save.best;
        assert!(app.apply_pack(&ctx, packs::ORIGINAL));
        assert!(app.artwork.is_none());
        assert_eq!(app.save.game, game);
        assert_eq!(app.save.best, best);
        assert!(app.apply_pack(&ctx, packs::LATCH));
        assert!(!app.apply_pack(&ctx, "folder:missing-pack-for-test"));
        assert!(app.artwork.is_some());
        assert_eq!(app.save.settings.character_pack, packs::LATCH);
        assert_eq!(app.save.game, game);
        assert_eq!(app.save.best, best);
    }
    #[test]
    fn legacy_save_and_missing_pack_fall_back_without_losing_run() {
        let ctx = egui::Context::default();
        let dir = tempfile::tempdir().unwrap();
        let mut save = Save::default();
        save.game.score = 450;
        save.best = [900, 500];
        let mut json = serde_json::to_value(&save).unwrap();
        json["settings"]
            .as_object_mut()
            .unwrap()
            .remove("character_pack");
        std::fs::write(
            dir.path().join("session.json"),
            serde_json::to_vec(&json).unwrap(),
        )
        .unwrap();
        let app = ScramApp::new(&ctx, dir.path().into(), None);
        assert_eq!(app.save.game, save.game);
        assert_eq!(app.save.best, save.best);
        assert_eq!(app.save.settings.character_pack, packs::LATCH);
        save.settings.character_pack = "folder:no-such-pack".into();
        save.write(dir.path()).unwrap();
        let app = ScramApp::new(&ctx, dir.path().into(), None);
        assert_eq!(app.save.game, save.game);
        assert_eq!(app.save.best, save.best);
        assert!(app.notice.contains("Using Latch"));
    }
}
