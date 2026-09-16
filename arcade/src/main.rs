mod pinball;
mod shelf;
use eframe::egui::{self, Key};
use fs2::FileExt;
use std::{
    fs::{File, OpenOptions},
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Game {
    Chess,
    Solitaire,
    Scram,
    Invaders,
    Pinball,
    Stack,
    Snake,
    Bubble,
    Blast,
    TwentyFortyEight,
    Shatter,
    Tanks,
    Minesweeper,
}
impl Game {
    const ALL: [Self; 13] = [
        Self::Pinball,
        Self::Solitaire,
        Self::Scram,
        Self::Invaders,
        Self::Chess,
        Self::Stack,
        Self::Snake,
        Self::Bubble,
        Self::Blast,
        Self::TwentyFortyEight,
        Self::Shatter,
        Self::Tanks,
        Self::Minesweeper,
    ];
    fn id(self) -> &'static str {
        match self {
            Self::Stack => "stack",
            Self::Chess => "chess",
            Self::Solitaire => "solitaire",
            Self::Scram => "scram",
            Self::Invaders => "invaders",
            Self::Pinball => "pinball",
            Self::Snake => "snake",
            Self::Bubble => "bubble",
            Self::Blast => "blast",
            Self::TwentyFortyEight => "2048",
            Self::Tanks => "tanks",
            Self::Minesweeper => "minesweeper",
            Self::Shatter => "shatter",
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Stack => "Stack",
            Self::Chess => "Chess",
            Self::Solitaire => "Solitaire",
            Self::Scram => "Scram",
            Self::Invaders => "Invaders",
            Self::Pinball => "Circuit Pinball",
            Self::Snake => "Snake",
            Self::Bubble => "Bubble",
            Self::Blast => "Blast",
            Self::TwentyFortyEight => "2048",
            Self::Tanks => "Tanks",
            Self::Minesweeper => "Minesweeper",
            Self::Shatter => "Shatter",
        }
    }
    fn line(self) -> &'static str {
        match self {
            Self::Stack => "Make room. Go again.",
            Self::Chess => "Take your time. Make your move.",
            Self::Solitaire => "A quiet hand of Klondike.",
            Self::Scram => "Keep moving. They are behind you.",
            Self::Invaders => "Hold the line. Clear the sky.",
            Self::Pinball => "One more ball. One more high score.",
            Self::Snake => "Eat. Grow. Leave yourself a way out.",
            Self::Bubble => "Make three. Clear your head.",
            Self::Blast => "Make room. Leave an exit.",
            Self::TwentyFortyEight => "Slide together. Make something bigger.",
            Self::Tanks => "Read the wind. Change the landscape.",
            Self::Minesweeper => "Read the field. Trust your next move.",
            Self::Shatter => "Find your angle. Break through.",
        }
    }
    fn image(self) -> egui::ImageSource<'static> {
        match self {
            Self::Stack => egui::include_image!("../../games/stack/docs/stack-game.png"),
            Self::Snake => egui::include_image!("../../games/snake/docs/shelf.svg"),
            Self::Bubble => egui::include_image!("../../games/bubble/docs/game.png"),
            Self::Blast => egui::include_image!("../../games/blast/docs/game.png"),
            Self::Minesweeper => egui::include_image!("../../games/minesweeper/docs/shelf.svg"),
            Self::Tanks => egui::include_image!("../../games/tanks/shelf.svg"),
            Self::Shatter => egui::include_image!("../../games/shatter/docs/shelf.svg"),
            Self::TwentyFortyEight => egui::include_image!("../../games/2048/docs/shelf.svg"),
            Self::Chess => egui::include_image!("../../games/chess/docs/preview.png"),
            Self::Solitaire => {
                egui::include_image!("../../games/solitaire/docs/screenshots/table.png")
            }
            Self::Scram => egui::include_image!("../../games/scram/docs/screenshots/charcoal.png"),
            Self::Invaders => egui::include_image!("../../games/invaders/docs/orbit-opening.png"),
            Self::Pinball => egui::include_image!("../../games/pinball/docs/upstream-circuit.png"),
        }
    }
}
trait ArcadeGame: eframe::App {
    fn prepare_style(&mut self, _: &egui::Context) {}
    fn ready(&self) -> bool {
        true
    }
    fn suspend(&mut self) {}
    fn set_input_enabled(&mut self, _: bool) {}
    fn finished(&mut self) -> bool {
        false
    }
}
impl ArcadeGame for omarchy_stack::app::StackApp {
    fn suspend(&mut self) {
        self.suspend();
    }
    fn finished(&mut self) -> bool {
        omarchy_stack::app::StackApp::finished(self)
    }
}
impl ArcadeGame for omarchy_bubble::app::BubbleApp {
    fn suspend(&mut self) {
        self.suspend();
    }
    fn finished(&mut self) -> bool {
        omarchy_bubble::app::BubbleApp::finished(self)
    }
}
impl ArcadeGame for omarchy_blast::app::App {
    fn suspend(&mut self) {
        self.suspend();
    }
    fn finished(&mut self) -> bool {
        omarchy_blast::app::App::finished(self)
    }
}
impl ArcadeGame for omarchy_shatter::app::App {
    fn suspend(&mut self) {
        self.suspend();
    }
    fn set_input_enabled(&mut self, enabled: bool) {
        self.set_input_enabled(enabled);
    }
    fn finished(&mut self) -> bool {
        omarchy_shatter::app::App::finished(self)
    }
}
impl ArcadeGame for omarchy_tanks::app::App {
    fn suspend(&mut self) {
        self.suspend();
    }
    fn set_input_enabled(&mut self, enabled: bool) {
        self.set_input_enabled(enabled);
    }
    fn finished(&mut self) -> bool {
        omarchy_tanks::app::App::finished(self)
    }
}
impl ArcadeGame for omarchy_minesweeper::app::App {
    fn prepare_style(&mut self, ctx: &egui::Context) {
        self.prepare_style(ctx);
    }
    fn suspend(&mut self) {
        self.suspend();
    }
    fn set_input_enabled(&mut self, enabled: bool) {
        self.set_input_enabled(enabled);
    }
}
impl ArcadeGame for omarchy_2048::app::App {}
impl ArcadeGame for omarchy_chess::ui::ChessApp {}
impl ArcadeGame for omarchy_solitaire::app::SolitaireApp {}
impl ArcadeGame for omarchy_scram::app::ScramApp {}
impl ArcadeGame for omarchy_invaders::App {}
impl ArcadeGame for pinball::Pinball {
    fn set_input_enabled(&mut self, enabled: bool) {
        self.set_input_enabled(enabled);
    }
    fn ready(&self) -> bool {
        self.ready()
    }
    fn finished(&mut self) -> bool {
        self.finished()
    }
    fn suspend(&mut self) {
        self.pause();
    }
}
impl ArcadeGame for omarchy_snake::app::SnakeApp {
    fn finished(&mut self) -> bool {
        omarchy_snake::app::SnakeApp::finished(self)
    }
    fn suspend(&mut self) {
        omarchy_snake::app::SnakeApp::suspend(self);
    }
}
struct Active {
    game: Game,
    app: Box<dyn ArcadeGame>,
    _lock: Option<Box<dyn std::any::Any>>,
}
impl Drop for Active {
    fn drop(&mut self) {
        self.app.on_exit(None);
    }
}
struct Arcade {
    active: Option<Active>,
    selected: usize,
    error: Option<String>,
    about: bool,
    confirm_home: bool,
    theme: omarchy_chess::theme::Theme,
    themed: Instant,
    capture: Option<PathBuf>,
    frames: usize,
    initial: Option<Game>,
    _lock: File,
}
impl Arcade {
    fn open(&mut self, game: Game, ctx: &egui::Context) {
        let result = (|| -> Result<Active, String> {
            let mut lock: Option<Box<dyn std::any::Any>> = None;
            let app: Box<dyn ArcadeGame> = match game {
                Game::Stack => Box::new(omarchy_stack::app::StackApp::new()),
                Game::Snake => Box::new(omarchy_snake::app::SnakeApp::new()),
                Game::Bubble => Box::new(omarchy_bubble::app::BubbleApp::new()),
                Game::Blast => Box::new(omarchy_blast::app::App::new()),
                Game::Minesweeper => Box::new(omarchy_minesweeper::app::App::new()?),
                Game::Tanks => Box::new(omarchy_tanks::app::App::new()),
                Game::Shatter => Box::new(omarchy_shatter::app::App::new()),
                Game::TwentyFortyEight => Box::new(omarchy_2048::app::App::new()?),
                Game::Chess => {
                    let dir = omarchy_chess::storage::state_dir();
                    lock = Some(Box::new(omarchy_chess::storage::SessionLock::acquire(
                        &dir,
                    )?));
                    Box::new(omarchy_chess::ui::ChessApp::new(dir))
                }
                Game::Solitaire => {
                    let dir = omarchy_solitaire::storage::state_dir();
                    lock = Some(Box::new(omarchy_solitaire::storage::SessionLock::acquire(
                        &dir,
                    )?));
                    Box::new(omarchy_solitaire::app::SolitaireApp::new(ctx, dir, None))
                }
                Game::Scram => {
                    let dir = omarchy_scram::storage::state_dir();
                    lock = Some(Box::new(
                        omarchy_scram::storage::SessionLock::acquire(&dir)
                            .map_err(|e| e.to_string())?,
                    ));
                    Box::new(omarchy_scram::app::ScramApp::new(ctx, dir, None))
                }
                Game::Invaders => Box::new(omarchy_invaders::App::new(
                    omarchy_invaders::storage::Store::open().map_err(|e| e.to_string())?,
                )),
                Game::Pinball => Box::new(pinball::Pinball::new(ctx).map_err(|e| e.to_string())?),
            };
            Ok(Active {
                game,
                app,
                _lock: lock,
            })
        })();
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
        self.active = None;
        self.confirm_home = false;
        ctx.memory_mut(|m| *m = egui::Memory::default());
        ctx.set_style(egui::Style::default());
        ctx.send_viewport_cmd(egui::ViewportCommand::Title("Omarchy Arcade".into()));
    }
}
impl eframe::App for Arcade {
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
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL, Key::Q))
        }) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
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
            ui.label("Original game artwork and engines; credits and licences are included with the app.");ui.label("Ctrl+H returns to Arcade. Each game keeps its own controls and saves.");
        });
        }
        self.frames += 1;
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
                            Ok(()) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
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
                println!("Omarchy Arcade\n--game chess|solitaire|scram|invaders|pinball|stack|snake|bubble|blast|2048|shatter|tanks|minesweeper\n--screenshot PATH\n--compact\n--version\nCtrl+H: return to Arcade. Ctrl+Q: quit.");
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
    let state = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/state")))
        .ok_or("No state directory")?
        .join("omarchy-retro-arcade");
    std::fs::create_dir_all(&state)?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(state.join("session.lock"))?;
    lock.try_lock_exclusive()
        .map_err(|_| "Omarchy Arcade is already running")?;
    let image = egui_extras::image::load_svg_bytes_with_size(
        include_bytes!("../../packaging/omarchy-retro-arcade.svg"),
        Some(egui::load::SizeHint::Size(128, 128)),
    )?;
    let icon = egui::IconData {
        rgba: image.pixels.iter().flat_map(|p| p.to_array()).collect(),
        width: 128,
        height: 128,
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(size)
            .with_min_inner_size([900., 760.])
            .with_app_id("io.github.tcballard.omarchy-retro-arcade")
            .with_icon(icon),
        ..Default::default()
    };
    eframe::run_native(
        "Omarchy Arcade",
        options,
        Box::new(move |cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(Arcade {
                active: None,
                selected: 0,
                error: None,
                about: false,
                confirm_home: false,
                theme: omarchy_chess::theme::Theme::load(),
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
