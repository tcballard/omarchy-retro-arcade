use crate::{
    cards::{self, DeckStyle},
    game::Card,
    storage::{self, Session},
    theme::{self, Art, Theme},
};
use eframe::egui::{
    self, pos2, vec2, Align2, Color32, Context, FontId, Id, Key, Rect, Sense, Stroke, StrokeKind,
    TextureHandle,
};
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    time::Duration,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dialog {
    New,
    Deck,
    Help,
    About,
}
#[derive(Clone, Copy, Debug)]
pub enum Action {
    Draw,
    Move(usize, usize, usize),
    Choose(usize, usize),
    Foundation(usize, usize),
    Undo,
    Hint,
    Deal(usize, bool),
    Finish,
    Clear,
}
#[derive(Clone, Copy)]
struct Visual {
    from: Rect,
    to: Rect,
    current: Rect,
    was_up: bool,
    up: bool,
    start: f64,
}
#[derive(Clone, Copy)]
struct Placed {
    card: Card,
    pile: usize,
    row: usize,
    rect: Rect,
    hit: Rect,
}
struct Drag {
    source: usize,
    row: usize,
    anchor: egui::Pos2,
    offset: egui::Vec2,
}
pub struct SolitaireApp {
    pub session: Session,
    pub theme: Theme,
    pub message: String,
    pub dialog: Option<Dialog>,
    pub selection: Option<(usize, usize)>,
    pub hint_target: Option<usize>,
    pub completing: bool,
    dir: PathBuf,
    writable: bool,
    save_error: String,
    logo: TextureHandle,
    art: Option<TextureHandle>,
    deck: crate::deck::DeckArt,
    visuals: [Option<Visual>; 52],
    drag: Option<Drag>,
    focus: (usize, usize),
    return_focus: bool,
    board_rect: Rect,
    now: f64,
    last_tick: Option<f64>,
    fraction: f64,
    last_save: f64,
    last_theme: f64,
    last_finish: f64,
    victory: Option<f64>,
    import: Option<Receiver<Option<PathBuf>>>,
    screenshot: Option<PathBuf>,
    frames: u32,
    capture_start: Option<f64>,
}
impl SolitaireApp {
    pub fn prepare_to_leave(&mut self) -> Result<(), String> {
        self.selection = None;
        self.completing = false;
        if !self.writable {
            return Err(
                "The original Solitaire save needs recovery before saving is allowed.".into(),
            );
        }
        self.persist();
        if self.save_error.is_empty() {
            Ok(())
        } else {
            Err(self.save_error.clone())
        }
    }
    pub fn new(ctx: &Context, dir: PathBuf, screenshot: Option<PathBuf>) -> Self {
        let loaded = storage::load(&dir);
        let session = loaded.session;
        let logo = egui_extras::image::load_svg_bytes_with_size(
            include_bytes!("../assets/omarchy-logo.svg"),
            Some(egui::load::SizeHint::Size(160, 160)),
        )
        .expect("bundled Omarchy SVG");
        let mut theme = Theme::default();
        theme.name = "House green".into();
        if session.preferences.follow_theme {
            theme.refresh();
        } else {
            theme.palette = theme::Palette::restore(&session.theme_snapshot);
            theme.name = session.theme_name.clone();
            if !session.locked_art.is_empty() {
                match theme::validate_art(&dir.join(&session.locked_art)) {
                    Ok(a) => theme.art = Some(a),
                    Err(e) => theme.error = e,
                }
            }
        }
        let deck = crate::deck::DeckArt::new(ctx, &theme.palette);
        let mut app = Self {
            deck,
            session,
            theme,
            message: loaded.message.clone(),
            dialog: None,
            selection: None,
            hint_target: None,
            completing: false,
            dir,
            writable: loaded.writable,
            save_error: if loaded.writable {
                String::new()
            } else {
                loaded.message
            },
            logo: ctx.load_texture("omarchy-logo", logo, Default::default()),
            art: None,
            visuals: [None; 52],
            drag: None,
            focus: (0, 0),
            return_focus: false,
            board_rect: Rect::NOTHING,
            now: 0.,
            last_tick: None,
            fraction: 0.,
            last_save: 0.,
            last_theme: 0.,
            last_finish: 0.,
            victory: None,
            import: None,
            screenshot,
            frames: 0,
            capture_start: None,
        };
        if std::env::var("SOLITAIRE_REDUCED_MOTION").as_deref() == Ok("1") {
            app.session.preferences.reduced_motion = true;
        }
        app.refresh_art(ctx);
        app
    }
    fn texture(ctx: &Context, art: &Art) -> Result<TextureHandle, String> {
        let image = if art.suffix == "svg" {
            egui_extras::image::load_svg_bytes_with_size(
                &art.bytes,
                Some(egui::load::SizeHint::Size(500, 700)),
            )?
        } else {
            let i = image::load_from_memory_with_format(&art.bytes, image::ImageFormat::Png)
                .map_err(|e| e.to_string())?
                .into_rgba8();
            egui::ColorImage::from_rgba_unmultiplied(
                [i.width() as usize, i.height() as usize],
                i.as_raw(),
            )
        };
        Ok(ctx.load_texture("special-card-back", image, Default::default()))
    }
    fn refresh_art(&mut self, ctx: &Context) {
        self.deck.refresh_backs(&self.theme.palette);
        self.art = None;
        let result = if self.session.preferences.pattern == 3
            && !self.session.preferences.custom_art.is_empty()
        {
            theme::validate_art(&self.dir.join(&self.session.preferences.custom_art)).map(Some)
        } else {
            Ok(self.theme.art.clone())
        };
        match result {
            Ok(Some(art)) => match Self::texture(ctx, &art) {
                Ok(t) => self.art = Some(t),
                Err(e) => self.theme.error = e,
            },
            Ok(None) => {}
            Err(e) => self.theme.error = e,
        }
    }
    pub fn persist(&mut self) {
        if !self.writable {
            return;
        }
        self.session.theme_snapshot = self.theme.palette.snapshot();
        self.session.theme_name = self.theme.name.clone();
        self.save_error = match storage::save(&self.dir, &self.session) {
            Ok(()) => String::new(),
            Err(e) => format!("Couldn't save this game: {e}"),
        };
        self.last_save = self.now;
    }
    fn changed(&mut self, record: bool) {
        self.selection = None;
        self.hint_target = None;
        if record {
            self.session.record_move();
        }
        if self.session.game.won() {
            self.message = "All home. Beautifully played.".into();
            self.victory = Some(self.now);
            self.completing = false;
        } else {
            self.victory = None;
        }
        self.persist();
    }
    pub fn act(&mut self, a: Action) {
        if self.completing
            && !matches!(
                a,
                Action::Undo | Action::Finish | Action::Deal(..) | Action::Clear
            )
        {
            return;
        }
        match a {
            Action::Draw => {
                if self.session.game.draw() {
                    self.message = "Build down in alternating colours. Aces go above.".into();
                    self.changed(true);
                }
            }
            Action::Move(s, i, t) => {
                if self.session.game.play(s, i, t) {
                    self.message = "Nicely placed.".into();
                    self.changed(true);
                } else {
                    self.message = "That move doesn't fit. Only kings fill empty columns.".into();
                }
            }
            Action::Choose(s, i) => {
                if s == 0 {
                    self.act(Action::Draw);
                    return;
                }
                if let Some((from, row)) = self.selection {
                    if self.session.game.legal(from, row, s) {
                        self.act(Action::Move(from, row, s));
                        return;
                    }
                }
                if self.selection == Some((s, i)) {
                    self.selection = None;
                } else if self.session.game.movable(s, i) {
                    self.selection = Some((s, i));
                    self.message = format!(
                        "{} selected. Choose a destination, or press F.",
                        self.session.game.state.piles[s][i].name()
                    );
                } else {
                    self.selection = None;
                }
                self.hint_target = None;
            }
            Action::Foundation(s, i) => {
                if let Some(t) = (2..6).find(|t| self.session.game.legal(s, i, *t)) {
                    self.act(Action::Move(s, i, t));
                } else {
                    self.message = "That card isn't ready for a foundation yet.".into();
                }
            }
            Action::Undo => {
                self.completing = false;
                if self.session.game.undo() {
                    self.message = "One move back.".into();
                    self.changed(false);
                }
            }
            Action::Hint => match self.session.game.hint() {
                Some((0, _, _)) => {
                    self.selection = None;
                    self.hint_target = Some(0);
                    self.message = "Draw from the stock, or recycle the waste.".into();
                }
                Some((s, i, t)) => {
                    self.selection = Some((s, i));
                    self.hint_target = Some(t);
                    self.message = format!(
                        "Try {} to {}.",
                        self.session.game.state.piles[s][i].name(),
                        pile_name(t)
                    );
                }
                None => {
                    self.message="No useful move found. Try Undo, or restart this deal. Hints aren't a solver.".into();
                    self.selection = None;
                    self.hint_target = None;
                }
            },
            Action::Deal(draw, restart) => {
                self.completing = false;
                self.session.deal(draw, restart);
                self.visuals = [None; 52];
                self.drag = None;
                self.fraction = 0.;
                self.focus = (0, 0);
                self.message = if restart {
                    "Same deal, fresh start."
                } else {
                    "A fresh deck. Draw a card to begin."
                }
                .into();
                self.changed(false);
            }
            Action::Finish => {
                self.completing = !self.completing && self.session.game.can_complete();
            }
            Action::Clear => {
                self.selection = None;
                self.hint_target = None;
                self.drag = None;
                self.completing = false;
            }
        }
    }
    fn style(&self, ctx: &Context) {
        let p = &self.theme.palette;
        let mut v = if theme::contrast(Color32::WHITE, p.table)
            > theme::contrast(Color32::BLACK, p.table)
        {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        v.panel_fill = p.table;
        v.window_fill = p.surface;
        v.override_text_color = Some(p.text);
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, p.line);
        v.widgets.inactive.weak_bg_fill = p.surface;
        v.widgets.inactive.bg_fill = p.surface;
        v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, p.line);
        v.widgets.hovered.weak_bg_fill = theme::blend(p.surface, p.accent, 0.12);
        v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, p.accent);
        v.widgets.active.weak_bg_fill = theme::blend(p.surface, p.accent, 0.20);
        v.widgets.active.bg_stroke = Stroke::new(1.5_f32, p.accent);
        v.selection.bg_fill = theme::blend(p.surface, p.accent, 0.25);
        v.selection.stroke = Stroke::new(1.5_f32, p.accent);
        v.window_corner_radius = egui::CornerRadius::same(5);
        ctx.set_visuals(v);
        arcade_presentation::apply(ctx);
        ctx.style_mut(|s| {
            s.spacing.button_padding = vec2(11., 7.);
            s.spacing.item_spacing = vec2(7., 7.);
            s.text_styles
                .insert(egui::TextStyle::Body, FontId::proportional(14.));
            s.text_styles
                .insert(egui::TextStyle::Button, FontId::proportional(14.));
        });
    }
    pub fn ui(&mut self, ctx: &Context) {
        self.now = ctx.input(|i| i.time);
        self.frames += 1;
        self.capture_start.get_or_insert(self.now);
        let delta = self
            .last_tick
            .replace(self.now)
            .map_or(0., |last| (self.now - last).max(0.));
        let active = ctx.input(|i| i.focused);
        if active
            && self.dialog.is_none()
            && self.import.is_none()
            && self.session.game.state.moves > 0
            && !self.session.game.won()
            && delta < 2.
        {
            self.fraction += delta;
            if self.fraction >= 1. {
                self.session.elapsed += self.fraction.floor() as u64;
                self.fraction = self.fraction.fract();
            }
        }
        if self.now - self.last_theme >= 1.5 || self.frames == 1 {
            self.last_theme = self.now;
            if self.session.preferences.follow_theme && self.theme.refresh() {
                self.refresh_art(ctx);
            }
        }
        if self.now - self.last_save >= 10. {
            self.persist();
        }
        if let Some(rx) = &self.import {
            if let Ok(path) = rx.try_recv() {
                self.import = None;
                if let Some(path) = path {
                    match theme::validate_art(&path).and_then(|art| {
                        let name = format!("custom-back.{}", art.suffix);
                        storage::atomic_write(&self.dir.join(&name), &art.bytes)?;
                        Ok(name)
                    }) {
                        Ok(name) => {
                            self.session.preferences.custom_art = name;
                            self.session.preferences.pattern = 3;
                            self.theme.error.clear();
                            self.refresh_art(ctx);
                            self.persist();
                            self.message = "Your special card back is ready.".into();
                        }
                        Err(e) => self.theme.error = e,
                    }
                }
            }
        }
        if active
            && self.dialog.is_none()
            && self.import.is_none()
            && self.completing
            && self.now - self.last_finish >= 0.16
        {
            self.last_finish = self.now;
            if let Some((s, i, t)) = self.session.game.completion_move() {
                self.session.game.play(s, i, t);
                self.changed(true);
            } else {
                self.completing = false;
            }
        }
        self.style(ctx);
        self.shortcuts(ctx);
        if self.dialog.is_some() || self.import.is_some() {
            self.drag = None;
        }
        egui::TopBottomPanel::top("toolbar")
            .frame(
                egui::Frame::new()
                    .fill(self.theme.palette.surface)
                    .inner_margin(14.),
            )
            .show(ctx, |ui| {
                ui.add_enabled_ui(self.dialog.is_none() && self.import.is_none(), |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(&self.logo).fit_to_exact_size(vec2(25., 25.)));
                        ui.label(
                            egui::RichText::new("SOLITAIRE")
                                .monospace()
                                .strong()
                                .size(19.),
                        );
                        ui.add_space(10.);
                        if ui
                            .button("New game")
                            .on_hover_text("New or restart · Ctrl+N")
                            .clicked()
                        {
                            self.dialog = Some(Dialog::New);
                        }
                        if ui
                            .add_enabled(
                                !self.session.game.history.is_empty(),
                                egui::Button::new("Undo"),
                            )
                            .on_hover_text("Ctrl+Z")
                            .clicked()
                        {
                            self.act(Action::Undo);
                        }
                        if ui
                            .add_enabled(
                                !self.session.game.won() && !self.completing,
                                egui::Button::new("Hint"),
                            )
                            .on_hover_text("H")
                            .clicked()
                        {
                            self.act(Action::Hint);
                        }
                        if ui
                            .add_enabled(
                                self.session.game.can_complete() || self.completing,
                                egui::Button::new(if self.completing { "Stop" } else { "Finish" }),
                            )
                            .clicked()
                        {
                            self.act(Action::Finish);
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Help").clicked() {
                                self.dialog = Some(Dialog::Help);
                            }
                            if ui.button("Deck").clicked() {
                                self.dialog = Some(Dialog::Deck);
                            }
                        });
                    });
                });
            });
        egui::TopBottomPanel::bottom("status")
            .frame(
                egui::Frame::new()
                    .fill(self.theme.palette.surface)
                    .inner_margin(egui::Margin::symmetric(16, 10)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "{:02}:{:02}",
                            self.session.elapsed / 60,
                            self.session.elapsed % 60
                        ))
                        .monospace(),
                    );
                    ui.separator();
                    ui.label(format!("{} moves", self.session.game.state.moves));
                    ui.separator();
                    ui.label(format!("Score {}", self.session.game.state.score));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("{}/52 home", self.session.game.home()));
                        for n in (0..13).rev() {
                            let (r, _) = ui.allocate_exact_size(vec2(5., 12.), Sense::hover());
                            ui.painter().rect_filled(
                                r,
                                1,
                                if self.session.game.home() >= 4 * (n + 1) {
                                    self.theme.palette.accent
                                } else {
                                    self.theme.palette.line
                                },
                            );
                        }
                    });
                });
                let message = if !self.save_error.is_empty() {
                    &self.save_error
                } else if !self.theme.error.is_empty() {
                    &self.theme.error
                } else {
                    &self.message
                };
                ui.label(egui::RichText::new(message).size(12.));
            });
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(self.theme.palette.table)
                    .inner_margin(16.),
            )
            .show(ctx, |ui| {
                let felt = ui.max_rect();
                ui.painter().rect_filled(felt, 0., self.theme.palette.table);
                arcade_presentation::grain(ui.painter(), felt, Color32::from_black_alpha(13), 3.);
                ui.painter().rect_stroke(
                    felt.shrink(5.),
                    0.,
                    Stroke::new(1_f32, arcade_presentation::BRASS.gamma_multiply(0.4)),
                    StrokeKind::Inside,
                );
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "KLONDIKE / DRAW {}",
                            self.session.game.state.draw
                        ))
                        .monospace()
                        .size(11.),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(&self.theme.name).size(12.));
                    });
                });
                ui.add_space(12.);
                ui.add_enabled_ui(self.dialog.is_none() && self.import.is_none(), |ui| {
                    self.board(ui);
                });
            });
        if let Some(dialog) = self.dialog {
            self.show_dialog(ctx, dialog);
        }
        if self.import.is_some() {
            egui::Window::new("Choose a special card back")
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, vec2(0., 0.))
                .show(ctx, |ui| {
                    ui.label("Select an SVG or PNG in the file chooser.");
                    if ui.button("Cancel import").clicked() {
                        self.import = None;
                    }
                });
        }
        self.capture(ctx);
        ctx.request_repaint_after(Duration::from_millis(500));
    }
    fn shortcuts(&mut self, ctx: &Context) {
        if self.dialog.is_some() || self.import.is_some() {
            if ctx.input(|i| i.key_pressed(Key::Escape)) {
                self.dialog = None;
                self.import = None;
                self.return_focus = true;
                ctx.memory_mut(|m| m.request_focus(Id::new("card-table")));
            }
            return;
        }
        let mut action = None;
        let mut quit = false;
        ctx.input_mut(|i| {
            if i.consume_key(egui::Modifiers::CTRL, Key::Q) {
                quit = true;
            }
            if i.consume_key(egui::Modifiers::CTRL, Key::N) {
                self.dialog = Some(Dialog::New);
            }
            if i.consume_key(egui::Modifiers::CTRL, Key::Comma) {
                self.dialog = Some(Dialog::Deck);
            }
            if i.consume_key(egui::Modifiers::NONE, Key::F1) {
                self.dialog = Some(Dialog::Help);
            }
            if i.consume_key(egui::Modifiers::CTRL, Key::Z) {
                action = Some(Action::Undo);
            }
            if i.consume_key(egui::Modifiers::NONE, Key::H) {
                action = Some(Action::Hint);
            }
        });
        if quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if let Some(a) = action {
            self.act(a);
        }
    }
    fn board(&mut self, ui: &mut egui::Ui) {
        let mut available = ui.available_size();
        available.x -= 12.;
        let w = ((available.x - 6. * 13.) / 7.).min(124.);
        let h = w * 1.4;
        let gap = (available.x - 7. * w) / 6.;
        let area_h = available.y.max(h * 2. + 40.);
        let natural = self.session.game.state.piles[6..]
            .iter()
            .map(|pile| {
                pile.iter()
                    .take(pile.len().saturating_sub(1))
                    .map(|c| if c.1 { w * 0.28 } else { w * 0.12 })
                    .sum::<f32>()
            })
            .fold(0., f32::max);
        let hidden_max = self.session.game.state.piles[6..]
            .iter()
            .map(|pile| pile.iter().filter(|c| !c.1).count())
            .max()
            .unwrap_or(0) as f32;
        let overflow = (h * 2. + 40. + natural - area_h).max(0.);
        let down = (w * 0.12 - overflow / hidden_max.max(1.)).max(w * 0.055);
        let longest = self.session.game.state.piles[6..]
            .iter()
            .map(|pile| {
                pile.iter()
                    .take(pile.len().saturating_sub(1))
                    .map(|c| if c.1 { w * 0.28 } else { down })
                    .sum::<f32>()
            })
            .fold(0., f32::max);
        let content_h = (h * 2. + 40. + longest).max(available.y);
        let board_id = Id::new("card-table");
        let mut action = None;
        egui::ScrollArea::vertical().id_salt("table-scroll").auto_shrink([false,false]).show(ui,|ui| {
            let (board,resp)=ui.allocate_exact_size(vec2(available.x,content_h),Sense::click());self.board_rect=board;
            if self.frames==1{resp.request_focus();}
            if resp.clicked(){resp.request_focus();}
            resp.widget_info(||egui::WidgetInfo::labeled(egui::WidgetType::Other,true,"Card table. Arrow keys navigate; Enter selects; Space draws; F moves to foundation."));
            // A stable explicit focus widget permits arrow navigation after mouse or Tab entry.
            let keyboard=ui.interact(board,board_id,Sense::focusable_noninteractive());
            if resp.has_focus(){keyboard.request_focus();}
            if self.return_focus && self.dialog.is_none() && self.import.is_none() {
                keyboard.request_focus();
                if keyboard.has_focus(){self.return_focus=false;}else{ui.ctx().request_repaint();}
            }
            let mut board_focus=keyboard.has_focus();
            let slots:[Rect;13]=std::array::from_fn(|p| {
                let col=if p==0{0}else if p==1{1}else if p<6{p+1}else{p-6};
                Rect::from_min_size(board.min+vec2(col as f32*(w+gap),if p<6{0.}else{h+40.}),vec2(w,h))
            });
            let mut placed=Vec::new();
            for (p,pile) in self.session.game.state.piles.iter().enumerate() {
                let mut y=0.;
                for (row,&card) in pile.iter().enumerate() {
                    let fan=if p==1{row.saturating_sub(pile.len().saturating_sub(self.session.game.state.draw)) as f32*w*0.20}else{0.};
                    let rect=slots[p].translate(vec2(fan,y));
                    let shown=p>=6 || row+1==pile.len() || (p==1 && row>=pile.len().saturating_sub(self.session.game.state.draw));
                    let mut hit=rect;
                    if row+1<pile.len() {if p>=6{hit.max.y=hit.min.y+if card.1{w*0.28}else{down};} else if p==1{hit.max.x=hit.min.x+w*0.20;}}
                    if shown {placed.push(Placed{card,pile:p,row,rect,hit});}
                    if p>=6{y+=if card.1{w*0.28}else{down};}
                    if !shown {self.visuals[card.0 as usize]=Some(Visual{from:rect,to:rect,current:rect,was_up:card.1,up:card.1,start:self.now});}
                }
            }
            for (p,slot) in slots.iter().enumerate() {
                let legal=self.selection.is_some_and(|(s,i)|self.session.game.legal(s,i,p));
                let marked=legal||self.hint_target==Some(p);
                ui.painter().rect_stroke(*slot,4,Stroke::new(if marked{2.0_f32}else{1.0_f32},if marked{self.theme.palette.accent}else{self.theme.palette.line}),StrokeKind::Inside);
                let label=if p==0{"↻"}else if p==1{"WASTE"}else if p<6{"A"}else{"K"};
                ui.painter().text(slot.center(),Align2::CENTER_CENTER,label,FontId::monospace(if p==1{11.}else{24.}),self.theme.palette.text.gamma_multiply(0.65));
                if p>=6 {ui.painter().text(slot.min-vec2(0.,8.),Align2::LEFT_BOTTOM,format!("{:02}",p-5),FontId::monospace(10.),self.theme.palette.text);}
                if marked {ui.painter().text(slot.right_top()+vec2(-5.,-5.),Align2::RIGHT_BOTTOM,"↓",FontId::proportional(20.),self.theme.palette.accent);}
                if self.session.game.state.piles[p].is_empty() {
                    let r=ui.interact(*slot,Id::new(("empty",p)),Sense::click());
                    r.widget_info(||egui::WidgetInfo::labeled(egui::WidgetType::Button,true,pile_name(p)));
                    if r.has_focus(){board_focus=true;self.focus=(p,0);}
                    if r.clicked(){action=Some(Action::Choose(p,0));self.focus=(p,0);keyboard.request_focus();}
                }
            }
            let mut drag_started=None;
            for c in &placed {
                let response=ui.interact(c.hit,Id::new(("card",c.card.0)),Sense::click_and_drag());
                if response.has_focus(){board_focus=true;self.focus=(c.pile,c.row);}
                let selected=self.selection.is_some_and(|(s,i)|s==c.pile&&c.row>=i);
                let label=if c.card.1{format!("{}, {}",c.card.name(),pile_name(c.pile))}else{format!("Face-down card, {}",pile_name(c.pile))};
                response.widget_info(||egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel,ui.is_enabled(),selected,&label));
                if response.double_clicked()||response.secondary_clicked(){action=Some(Action::Foundation(c.pile,c.row));keyboard.request_focus();}
                else if response.clicked(){action=Some(Action::Choose(c.pile,c.row));self.focus=(c.pile,c.row);keyboard.request_focus();}
                if response.drag_started() && self.session.game.movable(c.pile,c.row) && !self.completing {
                    if let Some(pointer)=response.interact_pointer_pos(){drag_started=Some(Drag{source:c.pile,row:c.row,anchor:pointer-response.drag_delta(),offset:vec2(0.,0.)});self.selection=Some((c.pile,c.row));}
                }
                if response.hovered() && self.session.game.movable(c.pile,c.row){ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);}
            }
            if let Some(drag)=drag_started{self.drag=Some(drag);}
            if let Some(d)=&mut self.drag {
                if let Some(pointer)=ui.ctx().pointer_interact_pos(){d.offset=pointer-d.anchor;}
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            }
            let mut overlay=Vec::new();
            for c in &placed {
                let dragging=self.drag.as_ref().filter(|d|d.source==c.pile&&c.row>=d.row);
                if let Some(d)=dragging {overlay.push((*c,d.offset));continue;}
                self.draw_card(ui,c,slots[0],None);
            }
            for (c,offset) in overlay {self.draw_card(ui,&c,slots[0],Some(offset));}
            if ui.input(|i|i.pointer.any_released()) {
                if let Some(d)=self.drag.take() {
                    let target=ui.ctx().pointer_interact_pos().and_then(|pos| (2..13).find(|&t|{
                        let rect=if t>=6 {Rect::from_min_max(slots[t].min,pos2(slots[t].right(),board.bottom()))}else{slots[t]};rect.contains(pos)
                    }));
                    if let Some(t)=target{action=Some(Action::Move(d.source,d.row,t));}else{self.message="Move returned. Drop onto a column or foundation.".into();}
                }
            }
            if board_focus {
                let keys=ui.input(|i|(i.key_pressed(Key::ArrowLeft),i.key_pressed(Key::ArrowRight),i.key_pressed(Key::ArrowUp),i.key_pressed(Key::ArrowDown),i.key_pressed(Key::Enter),i.key_pressed(Key::Space),i.key_pressed(Key::F),i.key_pressed(Key::Escape)));
                if keys.0||keys.1 {
                    self.focus.0=if keys.0{(self.focus.0+12)%13}else{(self.focus.0+1)%13};self.focus.1=self.session.game.state.piles[self.focus.0].len().saturating_sub(1);
                }
                if keys.2{self.focus.1=self.focus.1.saturating_sub(1);}
                if keys.3{self.focus.1+=1;}
                self.focus.1=self.focus.1.min(self.session.game.state.piles[self.focus.0].len().saturating_sub(1));
                if keys.4 {action=Some(Action::Choose(self.focus.0,self.focus.1));}
                if keys.5 {action=Some(Action::Draw);}
                if keys.6 {let (s,i)=self.selection.unwrap_or(self.focus);action=Some(Action::Foundation(s,i));}
                if keys.7 {action=Some(Action::Clear);}
                let r=placed.iter().find(|c|(c.pile,c.row)==self.focus).map_or(slots[self.focus.0],|c|c.hit);
                ui.painter().rect_stroke(r.expand(3.),5,Stroke::new(2.0_f32,self.theme.palette.accent),StrokeKind::Outside);
                if keys.0||keys.1||keys.2||keys.3 {keyboard.request_focus();ui.scroll_to_rect(r,Some(egui::Align::Center));}
            }
            if let Some(start)=self.victory {
                let elapsed=self.now-start;
                if elapsed<3. && !self.session.preferences.reduced_motion {
                    for n in 0..16 {
                        let t=elapsed-n as f64*0.06;
                        if t<0. {continue;}
                        let x=board.left()+((n as f32*61.+t as f32*130.)%board.width());
                        let y=board.top()+h+(t as f32*250.).rem_euclid((board.height()-h).max(1.));
                        let r=Rect::from_min_size(pos2(x,y),vec2(w*0.45,h*0.45));
                        let style=self.deck_style(0.5);cards::paint(ui.painter(),r,Card((n*3) as u8,true),&style,false);
                    }
                    ui.ctx().request_repaint();
                }
            }
        });
        if let Some(a) = action {
            self.act(a);
        }
    }
    fn deck_style(&self, sheen: f32) -> DeckStyle<'_> {
        DeckStyle {
            palette: &self.theme.palette,
            pattern: self.session.preferences.pattern.min(2),
            holographic: self.session.preferences.finish == "holographic",
            sheen,
            deck: &self.deck,
            art: self.art.as_ref(),
        }
    }
    fn draw_card(&mut self, ui: &egui::Ui, c: &Placed, stock: Rect, offset: Option<egui::Vec2>) {
        let id = c.card.0 as usize;
        let v = self.visuals[id].get_or_insert(Visual {
            from: stock,
            to: c.rect,
            current: stock,
            was_up: false,
            up: c.card.1,
            start: self.now
                + if c.pile >= 6 {
                    c.row as f64 * 0.025 + c.pile as f64 * 0.012
                } else {
                    0.
                },
        });
        if v.to != c.rect || v.up != c.card.1 {
            v.from = v.current;
            v.to = c.rect;
            v.was_up = v.up;
            v.up = c.card.1;
            v.start = self.now;
        }
        let t = if self.session.preferences.reduced_motion {
            1.
        } else {
            ((self.now - v.start) / 0.18).clamp(0., 1.) as f32
        };
        let ease = 1. - (1. - t).powi(3);
        let mut rect = Rect::from_min_max(
            v.from.min.lerp(v.to.min, ease),
            v.from.max.lerp(v.to.max, ease),
        );
        let face = if v.was_up != v.up && t < 0.5 {
            v.was_up
        } else {
            v.up
        };
        if v.was_up != v.up {
            let scale = (2. * t - 1.).abs().max(0.04);
            rect = Rect::from_center_size(rect.center(), vec2(rect.width() * scale, rect.height()));
        }
        if let Some(offset) = offset {
            rect = c.rect.translate(offset);
            v.from = rect;
            v.current = rect;
            v.start = self.now;
        } else {
            v.current = rect;
        }
        if t < 1. {
            ui.ctx().request_repaint();
        }
        ui.painter().rect_filled(
            rect.translate(vec2(1., 4.)),
            4.,
            Color32::from_black_alpha(55),
        );
        ui.painter().rect_filled(
            rect.translate(vec2(0., 2.)),
            4.,
            Color32::from_black_alpha(45),
        );
        let selected = self
            .selection
            .is_some_and(|(s, i)| s == c.pile && c.row >= i);
        let hovered = ui
            .ctx()
            .pointer_hover_pos()
            .is_some_and(|p| c.hit.contains(p));
        let sheen = if hovered && !self.session.preferences.reduced_motion {
            ui.ctx().request_repaint_after(Duration::from_millis(33));
            (self.now as f32 * 0.6).fract()
        } else {
            0.5
        };
        let style = self.deck_style(sheen);
        cards::paint(ui.painter(), rect, Card(c.card.0, face), &style, selected);
    }
    fn show_dialog(&mut self, ctx: &Context, dialog: Dialog) {
        let mut close = false;
        let title = match dialog {
            Dialog::New => "A fresh deck",
            Dialog::Deck => "Deck & settings",
            Dialog::Help => "How to play",
            Dialog::About => "About Omarchy Solitaire",
        };
        // Modal intercepts pointer input outside the window and owns keyboard focus.
        egui::Modal::new(Id::new("solitaire-dialog")).show(ctx,|ui| {
            ui.set_width(440_f32.min(ctx.screen_rect().width()-64.));
            ui.horizontal(|ui|{ui.heading(title);ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{if ui.button("Close").clicked(){close=true;}});});ui.separator();
            egui::ScrollArea::vertical().max_height((ctx.screen_rect().height()-160.).max(200.)).show(ui,|ui| {
                match dialog {
                    Dialog::New=>{
                        ui.label("Deal again, or try the same cards from the beginning.");ui.add_space(10.);
                        ui.horizontal(|ui|{
                            if ui.button("Draw one").clicked(){self.act(Action::Deal(1,false));close=true;}
                            if ui.button("Draw three").clicked(){self.act(Action::Deal(3,false));close=true;}
                            if ui.button("Restart this deal").clicked(){self.act(Action::Deal(self.session.game.state.draw,true));close=true;}
                        });
                        ui.add_space(8.);ui.label(format!("Current deal: {}",self.session.game.state.seed));
                        ui.label("Random deals aren't guaranteed winnable. The current table saves automatically until you choose a new deal.");
                    }
                    Dialog::Deck=>{
                        ui.label(egui::RichText::new("CARD BACK").monospace().size(11.));
                        ui.horizontal(|ui| {
                            for (n,name) in ["Tilework","Engraved","Foil","Special"].iter().enumerate() {
                                let enabled=n!=3||!self.session.preferences.custom_art.is_empty();
                                if ui.add_enabled(enabled,egui::Button::new(*name).selected(self.session.preferences.pattern==n)).clicked(){self.session.preferences.pattern=n;self.refresh_art(ctx);self.persist();}
                            }
                        });
                        let (r,_)=ui.allocate_exact_size(vec2(90.,126.),Sense::hover());cards::paint(ui.painter(),r,Card(0,false),&self.deck_style(0.5),false);
                        let mut holo=self.session.preferences.finish=="holographic";
                        if ui.checkbox(&mut holo,"Holographic finish").changed(){self.session.preferences.finish=if holo{"holographic"}else{"matte"}.into();self.persist();}
                        if ui.checkbox(&mut self.session.preferences.reduced_motion,"Reduce motion").changed(){self.persist();}
                        let mut follow=self.session.preferences.follow_theme;
                        if ui.checkbox(&mut follow,"Follow the Omarchy theme").changed(){
                            let result=if !follow {
                                if let Some(art)=&self.theme.art {
                                    let name=format!("locked-theme.{}",art.suffix);
                                    storage::atomic_write(&self.dir.join(&name),&art.bytes).map(|_|self.session.locked_art=name)
                                }else{self.session.locked_art.clear();Ok(())}
                            }else{Ok(())};
                            match result {
                                Ok(())=>{self.session.preferences.follow_theme=follow;if follow{self.session.locked_art.clear();self.theme.invalidate();self.theme.refresh();self.refresh_art(ctx);}self.persist();}
                                Err(e)=>self.theme.error=format!("Couldn't lock the card artwork: {e}"),
                            }
                        }
                        ui.label(format!("Theme: {}",self.theme.name));
                        ui.separator();
                        if ui.button("Import a special card back…").clicked(){
                            let (tx,rx)=mpsc::channel();self.import=Some(rx);
                            std::thread::spawn(move||{let file=rfd::FileDialog::new().set_title("Import a special card back").add_filter("Card artwork",&["svg","png"]).pick_file();let _=tx.send(file);});
                        }
                        if !self.session.preferences.custom_art.is_empty()&&ui.button("Use theme artwork again").clicked(){self.session.preferences.custom_art.clear();self.session.preferences.pattern=0;self.refresh_art(ctx);self.persist();}
                        ui.label("SVG or PNG · up to 2 MB · recommended 500 × 700. Imported art replaces the full back and stays on this computer.");
                        if !self.theme.error.is_empty(){ui.label(&self.theme.error);}
                        ui.separator();ui.label(egui::RichText::new("YOUR GAMES").monospace().size(11.));
                        for mode in ["1","3"] {let record=self.session.stats.get(mode).cloned().unwrap_or_default();ui.label(format!("Draw {mode}: {} played · {} won · best {}",record.played,record.won,if record.best==0{"—".into()}else{format!("{}:{:02}",record.best/60,record.best%60)}));}
                        if ui.button("About").clicked(){self.dialog=Some(Dialog::About);}
                    }
                    Dialog::Help=>{
                        ui.label("Build tableau columns down in alternating red and black. Move complete face-up sequences together. Only kings can fill empty columns.");
                        ui.label("Build each foundation up from ace to king in one suit. Draw one or three cards; recycle the stock as often as you like.");ui.separator();
                        egui::Grid::new("controls").spacing(vec2(24.,8.)).show(ui,|ui|{
                            for (action,key) in [("Select / place","Click or Enter"),("Move a sequence","Drag its first face-up card"),("Send to foundation","Double-click, right-click or F"),("Draw / recycle","Stock or Space on table"),("Navigate table","Arrow keys"),("Undo","Ctrl+Z"),("Hint","H"),("New / restart","Ctrl+N"),("Deck settings","Ctrl+,"),("Help / quit","F1 / Ctrl+Q"),("Clear / stop","Escape")] {ui.label(action);ui.label(key);ui.end_row();}
                        });
                        ui.separator();ui.label("Finish becomes available when stock and waste are empty and all tableau cards are face up. Each move remains undoable. Hints suggest legal moves; they aren't a solver.");
                        ui.label("Scoring: +10 to foundations, +5 waste to table, +5 reveal, −15 foundation to table, −20 recycle. Scores stop at zero. Undo restores the score; the clock keeps running.");
                        ui.label("The clock pauses in dialogs and inactive windows. Restarts and undo don't count the same deal or win twice. Tab moves between controls; the table has a visible keyboard focus outline.");
                    }
                    Dialog::About=>{
                        ui.label(egui::RichText::new(format!("Omarchy Solitaire {}",env!("CARGO_PKG_VERSION"))).strong());
                        ui.label("A little time well spent. Part of Tom's retro Omarchy Arcade collection.");
                        ui.label("Native Rust / egui. Offline, local saves, no accounts or telemetry.");
                        ui.label("Original code and card illustrations: MIT. Official Omarchy mark: separate rights retained. Community application; no endorsement claimed. No Microsoft game artwork is included.");
                        ui.hyperlink_to("Source and credits","https://github.com/tcballard/omarchy-solitaire");
                    }
                }
            });
        });
        if close {
            self.dialog = None;
            self.return_focus = true;
            ctx.memory_mut(|m| m.request_focus(Id::new("card-table")));
        }
    }
    fn capture(&mut self, ctx: &Context) {
        if self.screenshot.is_none() {
            return;
        }
        let capture = ctx.input(|i| {
            i.events.iter().find_map(|e| {
                if let egui::Event::Screenshot { image, .. } = e {
                    Some(image.clone())
                } else {
                    None
                }
            })
        });
        if let Some(image) = capture {
            if let Some(path) = self.screenshot.take() {
                let result = (|| -> Result<(), String> {
                    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                    }
                    let rgba: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                    use image::ImageEncoder;
                    let mut bytes = Vec::new();
                    image::codecs::png::PngEncoder::new(&mut bytes)
                        .write_image(
                            &rgba,
                            image.width() as u32,
                            image.height() as u32,
                            image::ExtendedColorType::Rgba8,
                        )
                        .map_err(|e| e.to_string())?;
                    storage::atomic_write(&path, &bytes)
                })();
                if let Err(e) = result {
                    eprintln!("Screenshot failed: {e}");
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        } else if self.now - self.capture_start.unwrap_or(self.now) > 1.2 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        }
        ctx.request_repaint_after(Duration::from_millis(150));
    }
}
impl eframe::App for SolitaireApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        arcade_presentation::apply(ctx);
        self.ui(ctx);
    }
    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        self.persist();
    }
}
fn pile_name(p: usize) -> String {
    match p {
        0 => "stock".into(),
        1 => "waste".into(),
        2..=5 => format!("foundation {}", p - 1),
        _ => format!("column {}", p - 5),
    }
}
