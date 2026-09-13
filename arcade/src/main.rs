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
}
impl Game {
    const ALL: [Self; 11] = [
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
            Self::Shatter => "Find your angle. Break through.",
        }
    }
    fn image(self) -> egui::ImageSource<'static> {
        match self {
            Self::Stack => egui::include_image!("../../games/stack/docs/stack-game.png"),
            Self::Snake => egui::include_image!("../../games/snake/docs/shelf.svg"),
            Self::Bubble => egui::include_image!("../../games/bubble/docs/game.png"),
            Self::Blast => egui::include_image!("../../games/blast/docs/game.png"),
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
    fn prepare_to_leave(&mut self) -> Result<(), String>;

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
    fn prepare_to_leave(&mut self) -> Result<(), String> {
        omarchy_stack::app::StackApp::prepare_to_leave(self)
    }

    fn suspend(&mut self) {
        self.suspend();
    }
    fn finished(&mut self) -> bool {
        omarchy_stack::app::StackApp::finished(self)
    }
}
impl ArcadeGame for omarchy_bubble::app::BubbleApp {
    fn prepare_to_leave(&mut self) -> Result<(), String> {
        omarchy_bubble::app::BubbleApp::prepare_to_leave(self)
    }

    fn suspend(&mut self) {
        self.suspend();
    }
    fn finished(&mut self) -> bool {
        omarchy_bubble::app::BubbleApp::finished(self)
    }
}
impl ArcadeGame for omarchy_blast::app::App {
    fn prepare_to_leave(&mut self) -> Result<(), String> {
        omarchy_blast::app::App::prepare_to_leave(self)
    }

    fn suspend(&mut self) {
        self.suspend();
    }
    fn finished(&mut self) -> bool {
        omarchy_blast::app::App::finished(self)
    }
}
impl ArcadeGame for omarchy_shatter::app::App {
    fn prepare_to_leave(&mut self) -> Result<(), String> {
        omarchy_shatter::app::App::prepare_to_leave(self)
    }

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
impl ArcadeGame for omarchy_2048::app::App {
    fn prepare_to_leave(&mut self) -> Result<(), String> {
        omarchy_2048::app::App::prepare_to_leave(self)
    }
}
impl ArcadeGame for omarchy_chess::ui::ChessApp {
    fn prepare_to_leave(&mut self) -> Result<(), String> {
        omarchy_chess::ui::ChessApp::prepare_to_leave(self)
    }
}
impl ArcadeGame for omarchy_solitaire::app::SolitaireApp {
    fn prepare_to_leave(&mut self) -> Result<(), String> {
        omarchy_solitaire::app::SolitaireApp::prepare_to_leave(self)
    }
}
impl ArcadeGame for omarchy_scram::app::ScramApp {
    fn prepare_to_leave(&mut self) -> Result<(), String> {
        omarchy_scram::app::ScramApp::prepare_to_leave(self)
    }
}
impl ArcadeGame for omarchy_invaders::App {
    fn prepare_to_leave(&mut self) -> Result<(), String> {
        omarchy_invaders::App::prepare_to_leave(self)
    }
}
impl ArcadeGame for pinball::Pinball {
    fn prepare_to_leave(&mut self) -> Result<(), String> {
        // The worker owns high-score/settings writes; its protocol has no save ACK.
        self.pause();
        Ok(())
    }

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
    fn prepare_to_leave(&mut self) -> Result<(), String> {
        omarchy_snake::app::SnakeApp::prepare_to_leave(self)
    }

    fn finished(&mut self) -> bool {
        omarchy_snake::app::SnakeApp::finished(self)
    }
    fn suspend(&mut self) {
        omarchy_snake::app::SnakeApp::suspend(self);
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LeaveTarget {
    Shelf,
    Quit,
}
struct LeaveRequest {
    target: LeaveTarget,
    error: Option<String>,
}
struct Active {
    game: Game,
    app: Box<dyn ArcadeGame>,
    _lock: Option<Box<dyn std::any::Any>>,
    leave_handled: bool,
}
impl Drop for Active {
    fn drop(&mut self) {
        if !self.leave_handled {
            self.app.on_exit(None);
        }
    }
}
struct Arcade {
    active: Option<Active>,
    selected: usize,
    error: Option<String>,
    about: bool,
    pending_leave: Option<LeaveRequest>,
    closing: bool,
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
                leave_handled: false,
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
    fn request_leave(&mut self, target: LeaveTarget, ctx: &egui::Context) {
        if self.pending_leave.is_some() || self.closing {
            return;
        }
        if target == LeaveTarget::Shelf
            && self
                .active
                .as_ref()
                .is_some_and(|a| a.game == Game::Pinball)
        {
            self.active.as_mut().unwrap().app.suspend();
            self.pending_leave = Some(LeaveRequest {
                target,
                error: None,
            });
        } else {
            self.attempt_leave(target, ctx);
        }
    }
    fn attempt_leave(&mut self, target: LeaveTarget, ctx: &egui::Context) {
        let result = self
            .active
            .as_mut()
            .map_or(Ok(()), |a| a.app.prepare_to_leave());
        match result {
            Ok(()) => self.complete_leave(target, ctx),
            Err(error) => {
                self.pending_leave = Some(LeaveRequest {
                    target,
                    error: Some(error),
                })
            }
        }
    }
    fn complete_leave(&mut self, target: LeaveTarget, ctx: &egui::Context) {
        // A successful save or explicit discard is final: Drop must not write again.
        if let Some(active) = &mut self.active {
            active.leave_handled = true;
        }
        self.home(ctx);
        if target == LeaveTarget::Quit {
            self.closing = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
    fn leave_dialog(&mut self, ctx: &egui::Context) {
        if let Some(request) = &self.pending_leave {
            let target = request.target;
            let error = request.error.clone();
            let mut retry = false;
            let mut discard = false;
            let mut stay = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape));
            egui::Modal::new(egui::Id::new("leave-game")).show(ctx, |ui| {
                ui.set_max_width(460.);
                if let Some(error) = &error {
                    ui.heading("Could not save");
                    ui.label(error);
                    ui.label("Your game is still open. Fix the save problem and retry, or leave without saving the latest progress.");
                    ui.horizontal(|ui| {
                        retry = ui.button("Retry save").clicked();
                        stay |= ui.button("Stay (Esc)").clicked();
                        discard = ui.button("Leave anyway (Alt+L)").clicked();
                    });
                } else {
                    ui.heading(if target == LeaveTarget::Quit { "Quit Arcade?" } else { "Return to Arcade?" });
                    ui.label("This ends the current pinball game. Pinball keeps high scores and settings, but cannot resume the table.");
                    ui.horizontal(|ui| {
                        retry = ui.button("End game and return (Enter)").clicked();
                        stay |= ui.button("Keep playing (Esc)").clicked();
                    });
                }
            });
            if error.is_some() {
                discard |= ctx.input_mut(|i| i.consume_key(egui::Modifiers::ALT, Key::L));
                retry |= ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter));
            }
            if error.is_none() {
                retry |= ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter));
            }
            if discard {
                self.complete_leave(target, ctx);
            } else if retry {
                self.attempt_leave(target, ctx);
            } else if stay {
                self.pending_leave = None;
            }
        }
    }
    fn home(&mut self, ctx: &egui::Context) {
        self.active = None;
        self.pending_leave = None;
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
        let quit = ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL, Key::Q))
        });
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        if !self.closing && (quit || close_requested) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.request_leave(LeaveTarget::Quit, ctx);
        }
        arcade_presentation::apply(ctx);
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
                        ui.label(
                            egui::RichText::new(a.game.name())
                                .monospace()
                                .color(arcade_presentation::BRASS),
                        );
                    });
                });
        }
        if home && self.active.is_some() {
            self.request_leave(LeaveTarget::Shelf, ctx);
        }
        // Freeze Rust games for the entire modal frame, including Stay's click/key.
        let block_game_input = self.pending_leave.is_some() || self.closing;
        self.leave_dialog(ctx);
        if let Some(a) = self.active.as_mut() {
            a.app.set_input_enabled(!block_game_input);
            if !block_game_input || a.game == Game::Pinball {
                a.app.update(ctx, frame);
            }
        } else {
            self.shelf(ctx);
        }
        if self.pending_leave.is_none() && self.active.as_mut().is_some_and(|a| a.app.finished()) {
            if self
                .active
                .as_ref()
                .is_some_and(|a| a.game == Game::Pinball)
            {
                self.complete_leave(LeaveTarget::Shelf, ctx);
            } else {
                self.request_leave(LeaveTarget::Shelf, ctx);
            }
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
                println!("Omarchy Arcade\n--game chess|solitaire|scram|invaders|pinball|stack|snake|bubble|blast|2048|shatter\n--screenshot PATH\n--compact\n--version\nCtrl+H: return to Arcade. Ctrl+Q: quit.");
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
                pending_leave: None,
                closing: false,
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

#[cfg(test)]
mod leave_tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    struct FakeGame {
        fail: Rc<Cell<bool>>,
        writes: Rc<Cell<usize>>,
        exits: Rc<Cell<usize>>,
    }
    impl eframe::App for FakeGame {
        fn update(&mut self, _: &egui::Context, _: &mut eframe::Frame) {}
        fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
            self.exits.set(self.exits.get() + 1);
        }
    }
    impl ArcadeGame for FakeGame {
        fn prepare_to_leave(&mut self) -> Result<(), String> {
            self.writes.set(self.writes.get() + 1);
            if self.fail.get() {
                Err("Disk full".into())
            } else {
                Ok(())
            }
        }
    }
    fn host(app: Box<dyn ArcadeGame>) -> Arcade {
        Arcade {
            active: Some(Active {
                game: Game::Shatter,
                app,
                _lock: None,
                leave_handled: false,
            }),
            selected: 0,
            error: None,
            about: false,
            pending_leave: None,
            closing: false,
            theme: omarchy_chess::theme::Theme::default(),
            themed: Instant::now(),
            capture: None,
            frames: 0,
            initial: None,
            _lock: tempfile::tempfile().unwrap(),
        }
    }
    fn frame(host: &mut Arcade, ctx: &egui::Context, events: Vec<egui::Event>) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(900., 760.),
                )),
                focused: true,
                events,
                ..Default::default()
            },
            |ctx| host.leave_dialog(ctx),
        )
    }
    fn click(host: &mut Arcade, ctx: &egui::Context, label: &str) {
        frame(host, ctx, vec![]);
        let out = frame(host, ctx, vec![]);
        let pos = out
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::Shape::Text(t) if t.galley.job.text == label => {
                    Some(t.pos + t.galley.size() / 2.)
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("Missing action: {label}"));
        for pressed in [true, false] {
            frame(
                host,
                ctx,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
    }
    #[test]
    fn failed_leave_stay_retry_and_discard_use_real_dialog_actions() {
        for target in [LeaveTarget::Shelf, LeaveTarget::Quit] {
            let fail = Rc::new(Cell::new(true));
            let writes = Rc::new(Cell::new(0));
            let exits = Rc::new(Cell::new(0));
            let mut arcade = host(Box::new(FakeGame {
                fail: fail.clone(),
                writes: writes.clone(),
                exits: exits.clone(),
            }));
            let ctx = egui::Context::default();
            arcade.request_leave(target, &ctx);
            assert!(arcade.active.is_some());
            assert_eq!(
                arcade.pending_leave.as_ref().unwrap().error.as_deref(),
                Some("Disk full")
            );
            arcade.request_leave(LeaveTarget::Quit, &ctx);
            assert_eq!(writes.get(), 1, "Repeated requests must not retry silently");
            click(&mut arcade, &ctx, "Stay (Esc)");
            assert!(arcade.active.is_some());
            assert!(arcade.pending_leave.is_none());
            arcade.request_leave(target, &ctx);
            click(&mut arcade, &ctx, "Retry save");
            assert!(arcade.active.is_some());
            fail.set(false);
            click(&mut arcade, &ctx, "Retry save");
            assert!(arcade.active.is_none());
            assert_eq!(arcade.closing, target == LeaveTarget::Quit);
            assert_eq!(exits.get(), 0, "Drop must not repeat a completed save");

            fail.set(true);
            let mut arcade = host(Box::new(FakeGame {
                fail,
                writes: writes.clone(),
                exits: exits.clone(),
            }));
            arcade.request_leave(target, &ctx);
            let before = writes.get();
            click(&mut arcade, &ctx, "Leave anyway (Alt+L)");
            assert!(arcade.active.is_none());
            assert_eq!(
                writes.get(),
                before,
                "Explicit discard must not write again"
            );
            assert_eq!(exits.get(), 0);
        }
    }
    #[test]
    fn real_chess_write_failure_keeps_app_until_retry_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let app = omarchy_chess::ui::ChessApp::new(dir.path().to_owned());
        let file = dir.path().join("session.json");
        std::fs::create_dir(&file).unwrap();
        let neighbour = dir.path().join("other-game.json");
        std::fs::write(&neighbour, b"preserve").unwrap();
        let mut arcade = host(Box::new(app));
        let ctx = egui::Context::default();
        arcade.request_leave(LeaveTarget::Shelf, &ctx);
        assert!(arcade.pending_leave.as_ref().unwrap().error.is_some());
        assert!(arcade.active.is_some());
        std::fs::remove_dir(&file).unwrap();
        arcade.attempt_leave(LeaveTarget::Shelf, &ctx);
        assert!(arcade.active.is_none());
        assert!(omarchy_chess::storage::load(dir.path()).is_ok());
        assert_eq!(std::fs::read(neighbour).unwrap(), b"preserve");
    }
}
