mod art;
mod bitmap;
mod game;
mod render;
mod sound;
pub mod storage;
mod theme;
use eframe::egui::{self, Key};
use game::Game;
use std::time::{Duration, Instant};
use storage::{Saved, Store};
pub struct App {
    art: art::Art,
    s: Saved,
    store: Store,
    paused: bool,
    settings: bool,
    help: bool,
    confirm: bool,
    error: Option<String>,
    save_ok: bool,
    theme: theme::Theme,
    sound: sound::Sound,
    last: Instant,
    saved: Instant,
    themed: Instant,
    acc: f32,
    had_focus: bool,
    pointer_target: Option<f32>,
}
impl App {
    pub fn prepare_to_leave(&mut self) -> Result<(), String> {
        self.paused = true;
        self.pointer_target = None;
        self.acc = 0.;
        if !self.save_ok {
            return Err(
                "The original Invaders save needs recovery before saving is allowed.".into(),
            );
        }
        self.store.save(&self.s).map_err(|e| e.to_string())
    }
    pub fn new(store: Store) -> Self {
        let (s, paused, error, save_ok) = match store.load() {
            Ok(Some(s)) => (s, true, None, true),
            Ok(None) => (
                Saved {
                    version: 1,
                    follow: true,
                    ..Default::default()
                },
                false,
                None,
                true,
            ),
            Err(e) => (
                Saved {
                    version: 1,
                    follow: true,
                    ..Default::default()
                },
                true,
                Some(e.to_string()),
                false,
            ),
        };
        let mut art = art::Art::default();
        if s.custom_art {
            art.reload();
        }
        Self {
            art,
            s,
            store,
            paused,
            settings: false,
            help: false,
            confirm: false,
            error,
            save_ok,
            theme: theme::Theme::load(),
            sound: Default::default(),
            last: Instant::now(),
            saved: Instant::now(),
            themed: Instant::now(),
            acc: 0.,
            had_focus: false,
            pointer_target: None,
        }
    }
    fn save(&mut self) {
        if self.save_ok {
            if let Err(e) = self.store.save(&self.s) {
                self.error = Some(format!("Could not save: {e}"));
            }
        }
        self.saved = Instant::now();
    }
    fn restart(&mut self) {
        if self.save_ok {
            self.save();
            if self.error.is_some() {
                return;
            }
        }
        if let Err(e) = self.store.archive() {
            self.error = Some(format!("Could not archive previous session: {e}"));
            return;
        }
        self.s.game = Game::default();
        self.save_ok = true;
        self.error = None;
        self.paused = false;
        self.confirm = false;
        self.acc = 0.;
        self.save();
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last).as_secs_f32().min(0.1);
        self.last = now;
        if self.themed.elapsed() > Duration::from_secs(2) {
            self.theme = if self.s.follow {
                theme::Theme::load()
            } else {
                theme::Theme::default()
            };
            self.themed = now;
        }
        let t = &self.theme;
        let mut visuals = if t.light() {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };
        visuals.panel_fill = t.background;
        visuals.window_fill = t.background;
        visuals.override_text_color = Some(t.foreground);
        visuals.selection.bg_fill = t.accent;
        visuals.selection.stroke = egui::Stroke::new(1_f32, t.accent_text());
        ctx.set_visuals(visuals);
        ctx.style_mut(|s| {
            s.text_styles
                .insert(egui::TextStyle::Body, egui::FontId::proportional(16.));
            s.text_styles
                .insert(egui::TextStyle::Button, egui::FontId::proportional(16.));
        });
        let focused = ctx.input(|i| i.focused);
        if !focused && self.had_focus {
            self.paused = true;
        }
        self.had_focus = focused;
        let popup = ctx.memory(|m| m.any_popup_open());
        let mut quit = false;
        let retry = self.s.game.over
            && !self.settings
            && !self.help
            && !self.confirm
            && self.error.is_none()
            && !popup
            && focused
            && ctx.input(|i| i.key_pressed(Key::Enter));
        if retry {
            self.restart();
        }
        ctx.input_mut(|i| {
            if i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL, Key::Q)) {
                quit = true;
            }
            if i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL, Key::N)) {
                self.confirm = true;
                self.paused = true;
            }
            if i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL, Key::M)) {
                self.s.sound = !self.s.sound;
            }
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                Key::Comma,
            )) {
                self.settings = true;
                self.paused = true;
            }
            if i.key_pressed(Key::F1) {
                self.help = true;
                self.paused = true;
            }
            if i.key_pressed(Key::Escape) {
                if self.settings || self.help || self.confirm || popup {
                    self.settings = false;
                    self.help = false;
                    self.confirm = false;
                } else {
                    self.paused = !self.paused;
                }
            }
            if i.key_pressed(Key::P) && !self.settings && !self.help && !self.confirm {
                self.paused = !self.paused;
            }
        });
        if quit {
            self.save();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        let mut menu_open = false;
        egui::TopBottomPanel::top("menus").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                menu_open |= ui
                    .menu_button("Game", |ui| {
                        if ui.button("New game     Ctrl+N").clicked() {
                            self.confirm = true;
                            self.paused = true;
                            ui.close_menu();
                        }
                        if ui.button("Pause / resume      P").clicked() {
                            self.paused = !self.paused;
                            ui.close_menu();
                        }
                        if ui.button("Quit     Ctrl+Q").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    })
                    .inner
                    .is_some();
                if ui.button("Settings").clicked() {
                    self.settings = true;
                    self.paused = true;
                }
                if ui.button("Help").clicked() {
                    self.help = true;
                    self.paused = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("OMARCHY ARCADE").small().weak());
                });
            });
        });
        let blocked = !focused
            || self.paused
            || self.settings
            || self.help
            || self.confirm
            || menu_open
            || self.error.is_some();
        egui::TopBottomPanel::bottom("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("MOUSE / A/D  MOVE    HOLD CLICK / SPACE  FIRE").size(14.),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(if self.s.sound {
                            "SOUND ON"
                        } else {
                            "SOUND OFF"
                        })
                        .clicked()
                    {
                        self.s.sound = !self.s.sound;
                    }
                });
            });
        });
        let pointer = render::board(self, ctx, blocked);
        if !blocked {
            ctx.memory_mut(|m| {
                if let Some(id) = m.focused() {
                    m.surrender_focus(id);
                }
            });
            let (axis, fire) = ctx.input(|i| {
                (
                    (i.key_down(Key::ArrowRight) || i.key_down(Key::D)) as i32 as f32
                        - (i.key_down(Key::ArrowLeft) || i.key_down(Key::A)) as i32 as f32,
                    i.key_down(Key::Space) && !i.modifiers.ctrl,
                )
            });
            if axis != 0. || !pointer.inside {
                self.pointer_target = None;
            } else if let Some(target) = pointer.target {
                self.pointer_target = Some(target);
            }
            self.acc += elapsed;
            let mut impact = false;
            while self.acc >= 1. / 120. {
                let axis = self.pointer_target.map_or(axis, |target| {
                    // Follow at normal ship speed; never teleport or overshoot.
                    ((target - self.s.game.ship) / (340. / 120.)).clamp(-1., 1.)
                });
                impact |= self.s.game.step(1. / 120., axis, fire || pointer.fire);
                self.acc -= 1. / 120.;
            }
            self.s.high = self.s.high.max(self.s.game.score);
            if impact && self.s.sound {
                self.sound.play(self.s.game.over);
            }
        } else {
            self.acc = 0.;
            self.pointer_target = None;
        }

        if self.settings {
            egui::Window::new("Settings")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.checkbox(&mut self.s.follow, "Follow Omarchy colours");
                    ui.checkbox(&mut self.s.sound, "Sound effects");
                    if !sound::Sound::available() {
                        ui.label("Audio needs paplay (libpulse on Arch).");
                    }
                    ui.label("The playfield stays dark for clear projectiles.");
                    ui.separator();
                    ui.label("Orbit artwork");
                    ui.label("Edit the sprite PNG independently of game rules.");
                    ui.horizontal(|ui| {
                        if ui.button("Create editable copy").clicked() {
                            self.art.editable_copy();
                        }
                        if ui.button("Reload artwork").clicked() && self.art.reload() {
                            self.s.custom_art = true;
                        }
                    });
                    if ui.button("Use built-in artwork").clicked() {
                        self.art.builtin();
                        self.s.custom_art = false;
                    }
                    ui.label(art::path().display().to_string());
                    ui.label(&self.art.message);
                    if ui.button("Done").clicked() {
                        self.settings = false;
                        self.save();
                    }
                });
        }
        if self.help {
            egui::Window::new("About Omarchy Invaders")
                .collapsible(false)
                .show(ctx, |ui| {
                    render::about_icon(ui, &self.art);
                    ui.heading("OMARCHY INVADERS");
                    ui.label(concat!(
                        "Omarchy Arcade · ",
                        env!("CARGO_PKG_VERSION"),
                        " · Community project"
                    ));
                    ui.separator();
                    ui.label("Clear the formation before it reaches your ship.");
                    ui.label("Bunkers erode under fire. The bonus craft is worth 150.");
                    ui.label("Move the mouse over the field to steer; hold left click to fire.");
                    ui.label("← → or A/D: move · Space: fire · P/Esc: pause");
                    ui.label("Arrow keys take over immediately. Move the mouse to steer again.");
                    ui.label("Ctrl+N: new · Ctrl+M: sound · Ctrl+Q: quit");
                    ui.label("Ctrl+,: settings · F1: help");
                    ui.separator();
                    ui.label("Original Rust gameplay. Orbit artwork developed with ImageGen.");
                    ui.label("Audio helper adapted from Omarchy Chess.");
                    if ui.button("Close").clicked() {
                        self.help = false;
                    }
                });
        }
        if self.confirm {
            egui::Window::new("Start a new game?")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Your previous saved run will be archived.");
                    ui.horizontal(|ui| {
                        if ui.button("New game").clicked() {
                            self.restart();
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm = false;
                        }
                    });
                });
        }
        if let Some(e) = self.error.clone() {
            egui::Window::new("Save needs attention")
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label(e);
                    if !self.save_ok {
                        if ui.button("Archive original and start new game").clicked() {
                            self.restart();
                        }
                    } else if ui.button("Retry save").clicked() {
                        self.error = None;
                        self.save();
                    }
                });
        }
        if self.saved.elapsed() > Duration::from_secs(5) {
            self.save();
        }
        ctx.request_repaint_after(Duration::from_millis(8));
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save();
    }
}
