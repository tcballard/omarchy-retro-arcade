mod catalog;
mod desktop;
mod input;
mod pinball;
mod session;
mod shelf;
use catalog::Game;
use eframe::egui::{self, Key};
use session::Active;
use std::{
    fs::File,
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, PartialEq)]
enum Closing {
    Home,
    Quit,
}

struct Arcade {
    closing: Option<Closing>,

    input: input::FocusInput,
    active: Option<Active>,
    selected: usize,
    error: Option<String>,
    about: bool,
    confirm_home: bool,
    theme: arcade_platform::theme::Theme,
    themed: Instant,
    capture: Option<PathBuf>,
    frames: usize,
    initial: Option<Game>,
    _lock: File,
}
impl Arcade {
    fn open(&mut self, game: Game, ctx: &egui::Context) {
        let result = Active::open(game, ctx);
        match result {
            Ok(active) => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
                    "{} - Omarchy Arcade",
                    game.name()
                )));
                ctx.request_repaint();
                self.selected = Game::ALL.iter().position(|g| *g == game).unwrap_or(0);
                self.active = Some(active);
                self.error = None;
            }
            Err(e) => self.error = Some(e),
        }
    }
    fn home(&mut self, ctx: &egui::Context) {
        self.begin_close(Closing::Home);
        self.poll_close(ctx);
    }
    fn begin_close(&mut self, target: Closing) {
        if self.closing.is_none() {
            if let Some(active) = self.active.as_mut() {
                active.app.begin_shutdown();
            }
        }
        if self.closing != Some(Closing::Quit) {
            self.closing = Some(target);
        }
    }
    fn poll_close(&mut self, ctx: &egui::Context) -> bool {
        let Some(target) = self.closing else {
            return false;
        };
        if self
            .active
            .as_mut()
            .is_none_or(|active| active.app.poll_shutdown())
        {
            self.active = None;
            self.closing = None;
            self.reset_shelf(ctx);
            ctx.request_repaint();
            if target == Closing::Quit {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.label("Closing Pinball…");
                });
            });
            ctx.request_repaint_after(Duration::from_millis(10));
        }
        true
    }
    fn reset_shelf(&mut self, ctx: &egui::Context) {
        self.confirm_home = false;
        ctx.memory_mut(|m| *m = egui::Memory::default());
        ctx.set_style(egui::Style::default());
        ctx.send_viewport_cmd(egui::ViewportCommand::Title("Omarchy Arcade".into()));
    }
}
impl eframe::App for Arcade {
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.input.filter(ctx, raw);
    }
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::F11)) {
            toggle_fullscreen(ctx);
        }
        if let Some(g) = self.initial.take() {
            self.open(g, ctx);
        }
        let mut home = ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL, Key::H))
        });
        if ctx.input(|i| i.viewport().close_requested())
            && (self.active.is_some() || self.closing.is_some())
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.begin_close(Closing::Quit);
        }
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL, Key::Q))
        }) {
            self.begin_close(Closing::Quit);
        }
        if self.poll_close(ctx) {
            return;
        }
        arcade_presentation::apply(ctx);
        if let Some(a) = self.active.as_mut() {
            a.app.prepare_style(ctx);
        }
        if let Some(a) = self.active.as_ref() {
            egui::TopBottomPanel::top("arcade-navigation")
                .frame(
                    egui::Frame::NONE
                        .fill(ctx.style().visuals.panel_fill)
                        .inner_margin(egui::Margin::symmetric(16, 8)),
                )
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        home |= ui.button("Arcade    Ctrl+H").clicked();
                        if ui
                            .button("Full screen")
                            .on_hover_text("Toggle fullscreen · F11")
                            .clicked()
                        {
                            toggle_fullscreen(ctx);
                        }
                        ui.separator();
                        ui.label(egui::RichText::new(a.game.name()).monospace().color(
                            if a.game == Game::Minesweeper {
                                ctx.style().visuals.text_color()
                            } else {
                                arcade_presentation::BRASS
                            },
                        ));
                    });
                });
        }
        if home && self.active.is_some() {
            if self
                .active
                .as_ref()
                .is_some_and(|a| a.game == Game::Pinball)
            {
                self.confirm_home = true;
                if let Some(a) = self.active.as_mut() {
                    a.app.suspend();
                }
            } else {
                self.home(ctx);
            }
        }
        let block_game_input = self.confirm_home;
        if self.confirm_home {
            let mut leave = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter));
            let mut cancel = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape));
            egui::Modal::new(egui::Id::new("return-to-arcade")).show(ctx, |ui| {
                ui.set_max_width(430.);
                ui.heading("Return to Arcade?");
                ui.label(
                    "This ends the current pinball game. Saved high scores and settings are kept.",
                );
                ui.horizontal(|ui| {
                    leave |= ui.button("End game and return (Enter)").clicked();
                    cancel |= ui.button("Keep playing (Esc)").clicked();
                });
            });
            if leave {
                self.home(ctx);
                return;
            } else if cancel {
                self.confirm_home = false;
            }
        }
        if let Some(a) = self.active.as_mut() {
            // Keep the worker rendering, but suppress input through the closing frame too.
            a.app.set_input_enabled(!block_game_input);
            a.app.update(ctx, frame);
        } else {
            self.shelf(ctx);
        }
        if self.active.as_mut().is_some_and(|a| a.app.finished()) {
            self.home(ctx);
        }
        if let Some(error) = self.error.clone() {
            egui::Window::new("Could not open game").show(ctx, |ui| {
                ui.label(error);
                if ui.button("Close").clicked() {
                    self.error = None;
                }
            });
        }
        if self.about {
            egui::Window::new("About Omarchy Arcade").open(&mut self.about).show(ctx,|ui|{
            ui.heading("Omarchy Arcade");ui.label(concat!("Version ",env!("CARGO_PKG_VERSION")));ui.label("Native games. A community project for Omarchy.");
            ui.label("Tanks: original Arcade artillery game. Preview; original synthesized sound.");
            ui.label("Shatter: original Arcade game, layouts and synthesized audio.");
            ui.hyperlink_to("2048: Avi Barit (avibarit)", "https://github.com/avibarit/2048");
            ui.hyperlink_to("Original 2048: Gabriele Cirulli", "https://github.com/gabrielecirulli/2048");
            ui.label("Ridgeline: original tower defence, ten maps, artwork and synthesized sound by Omarchy Arcade contributors. Canyon Defense was a design reference only.");
            ui.label("FreeSki: original downhill skiing, five Slalom courses, creature design and synthesized sound by Omarchy Arcade contributors.");
            ui.label("Original game artwork and engines; credits and licences are included with the app.");ui.label("Ctrl+H returns to Arcade. Each game keeps its own controls and saves.");
        });
        }
        self.frames += 1;
        let mut captured = false;
        if let Some(path) = &self.capture {
            if self.frames > 60 && self.active.as_ref().is_none_or(|a| a.app.ready()) {
                for event in ctx.input(|i| i.events.clone()) {
                    if let egui::Event::Screenshot { image, .. } = event {
                        let pixels: Vec<u8> =
                            image.pixels.iter().flat_map(|p| p.to_array()).collect();
                        match image::save_buffer(
                            path,
                            &pixels,
                            image.width() as u32,
                            image.height() as u32,
                            image::ColorType::Rgba8,
                        ) {
                            Ok(()) => captured = true,
                            Err(e) => {
                                eprintln!("Screenshot: {e}");
                            }
                        }
                    }
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            }
            ctx.request_repaint_after(Duration::from_millis(25));
        }
        if captured {
            self.begin_close(Closing::Quit);
            ctx.request_repaint();
        }
    }
    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        self.active = None;
    }
}
fn toggle_fullscreen(ctx: &egui::Context) {
    // A clicked fullscreen button must not retain Space/Enter from gameplay.
    ctx.memory_mut(|memory| {
        if let Some(id) = memory.focused() {
            memory.surrender_focus(id);
        }
    });
    let fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fullscreen));
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut capture = None;
    let mut initial = None;
    let mut size = [1280., 900.];
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--version" => {
                println!("Omarchy Arcade {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--help" | "-h" => {
                let games = Game::ALL.map(Game::id).join("|");
                println!("Omarchy Arcade\n--game {games}\n--screenshot PATH\n--compact\n--version\nCtrl+H: return to Arcade. Ctrl+Q: quit.");
                return Ok(());
            }
            "--game" => {
                let id = args.next().ok_or("Missing game")?;
                initial = Some(
                    Game::ALL
                        .into_iter()
                        .find(|g| g.id() == id)
                        .ok_or("Unknown game")?,
                );
            }
            "--screenshot" => {
                capture = Some(PathBuf::from(args.next().ok_or("Missing screenshot path")?))
            }
            "--compact" => size = [900., 760.],
            _ => return Err(format!("Unknown argument: {arg}").into()),
        }
    }
    let lock = desktop::acquire_session_lock()?;
    let options = desktop::native_options(size)?;
    eframe::run_native(
        "Omarchy Arcade",
        options,
        Box::new(move |cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(Arcade {
                closing: None,
                input: input::FocusInput::default(),
                active: None,
                selected: 0,
                error: None,
                about: false,
                confirm_home: false,
                theme: arcade_platform::theme::Theme::load(),
                themed: Instant::now(),
                capture,
                frames: 0,
                initial,
                _lock: lock,
            }))
        }),
    )?;
    Ok(())
}
