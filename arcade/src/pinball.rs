use eframe::egui::{self, Key};
use std::{
    collections::HashSet,
    fmt::Write as _,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
const WIDTH: usize = 1152;
const HEIGHT: usize = 790;
const MAX_DIMENSION: usize = 1600;
const MAX_PIXELS: usize = 1_000_000;
struct FramePixels {
    size: [usize; 2],
    bytes: Vec<u8>,
}
fn valid_size([width, height]: [usize; 2]) -> bool {
    (64..=MAX_DIMENSION).contains(&width)
        && (64..=MAX_DIMENSION).contains(&height)
        && width * height <= MAX_PIXELS
}
fn surface_size(size: egui::Vec2) -> [usize; 2] {
    let width = size.x.max(64.0);
    let height = size.y.max(64.0);
    let scale = 1.0_f32
        .min(MAX_DIMENSION as f32 / width.max(height))
        .min((MAX_PIXELS as f32 / (width * height)).sqrt());
    [
        ((width * scale) as usize / 8 * 8).max(64),
        ((height * scale) as usize / 8 * 8).max(64),
    ]
}
fn read_frame(reader: &mut impl Read) -> io::Result<FramePixels> {
    let mut header = [0; 12];
    reader.read_exact(&mut header)?;
    let size = [
        u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize,
        u32::from_le_bytes(header[8..12].try_into().unwrap()) as usize,
    ];
    if &header[..4] != b"OAR1" || !valid_size(size) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid Circuit frame",
        ));
    }
    let mut bytes = vec![0; size[0] * size[1] * 4];
    reader.read_exact(&mut bytes)?;
    Ok(FramePixels { size, bytes })
}
fn helper() -> io::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    let dir = exe
        .parent()
        .ok_or_else(|| io::Error::other("No executable directory"))?;
    let paths = [
        dir.join("../libexec/omarchy-retro-arcade/circuit"),
        dir.join("circuit"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../build/pinball/bin/omarchy-spacecadet-game"),
    ];
    paths.into_iter().find(|p| p.is_file()).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Circuit engine is missing. Reinstall Omarchy Arcade.",
        )
    })
}
/// Which table the worker runs. Circuit is bundled; the classic table needs
/// the player's own Space Cadet files, which are never distributed here.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Table {
    Circuit,
    Classic,
}
const DATA_FILES: [&str; 6] = [
    "PINBALL.DAT",
    "pinball.dat",
    "CADET.DAT",
    "cadet.dat",
    "DEMO.DAT",
    "demo.dat",
];
fn valid_data_dir(path: &Path) -> bool {
    path.is_dir() && DATA_FILES.iter().any(|name| path.join(name).is_file())
}
fn home_join(variable: &str, fallback: &str) -> PathBuf {
    std::env::var_os(variable)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(fallback)
        })
}
/// The standalone launcher's directory, so both remember the same folder.
fn config_dir() -> PathBuf {
    home_join("XDG_CONFIG_HOME", ".config").join("omarchy-spacecadet")
}
fn expand(text: &str) -> PathBuf {
    let text = text.trim();
    match text.strip_prefix("~/") {
        Some(rest) => PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(rest),
        None => PathBuf::from(text),
    }
}
fn read_line(path: &Path) -> Option<String> {
    let bytes = arcade_platform::storage::read_bounded(path, 4096).ok()?;
    let text = String::from_utf8(bytes).ok()?;
    let line = text.lines().next()?.trim();
    (!line.is_empty()).then(|| line.to_owned())
}
/// The saved folder first, then the places the standalone launcher searches.
fn find_data_dir(config: &Path, data_home: &Path) -> Option<PathBuf> {
    read_line(&config.join("data-directory"))
        .map(PathBuf::from)
        .into_iter()
        .chain([
            data_home.join("omarchy-spacecadet"),
            PathBuf::from("/usr/share/SpaceCadetPinball"),
            PathBuf::from("/usr/local/share/SpaceCadetPinball"),
        ])
        .find(|p| valid_data_dir(p))
}
fn saved_table(config: &Path) -> Table {
    match read_line(&config.join("arcade-table")).as_deref() {
        Some("classic") => Table::Classic,
        _ => Table::Circuit,
    }
}
fn save_choice(config: &Path, table: Table, data: Option<&Path>) -> Result<(), String> {
    if let (Table::Classic, Some(dir)) = (table, data) {
        let line = format!("{}\n", dir.display());
        arcade_platform::storage::atomic_write(&config.join("data-directory"), line.as_bytes())?;
    }
    let name = if table == Table::Classic {
        "classic\n"
    } else {
        "circuit\n"
    };
    arcade_platform::storage::atomic_write(&config.join("arcade-table"), name.as_bytes())
}
/// Classic mode is the unmodified engine started in the data folder, exactly
/// as the standalone launcher starts it.
fn worker_command(helper: &Path, classic: Option<&Path>) -> Command {
    let mut command = Command::new(helper);
    command.arg("--arcade-bridge");
    match classic {
        Some(dir) => {
            command.current_dir(dir);
        }
        None => {
            command.arg("--omarchy-table");
        }
    }
    command
}
struct TableDialog {
    choice: Table,
    folder: String,
    problem: Option<String>,
    browse: Option<mpsc::Receiver<Option<String>>>,
}
struct Link {
    worker: Worker,
    pixels: Arc<Mutex<Option<FramePixels>>>,
    errors: mpsc::Receiver<String>,
}
#[derive(Default)]
struct Shutdown {
    deadline: Option<Instant>,
    reaped: bool,
}
struct Worker {
    shutdown: Shutdown,
    child: Child,
    input: Option<mpsc::SyncSender<String>>,
    writer: Option<thread::JoinHandle<()>>,
    reader: Option<thread::JoinHandle<()>>,
}
impl Worker {
    fn begin_shutdown(&mut self) {
        if self.shutdown.deadline.is_some() {
            return;
        }
        self.shutdown.deadline = Some(Instant::now() + Duration::from_secs(2));
        if let Some(input) = self.input.take() {
            let _ = input.try_send("quit".into());
            // Disconnect even when the queue is full; the writer must reach EOF.
        }
    }
    /// Poll between UI frames. Never wait for the child or a running thread here.
    fn poll_shutdown(&mut self) -> bool {
        let Some(deadline) = self.shutdown.deadline else {
            return false;
        };
        if !self.shutdown.reaped {
            match self.child.try_wait() {
                Ok(Some(_)) => self.shutdown.reaped = true,
                _ if Instant::now() >= deadline => {
                    let _ = self.child.kill();
                }
                _ => {}
            }
        }
        if !self.shutdown.reaped {
            return false;
        }
        for handle in [&mut self.writer, &mut self.reader] {
            if handle.as_ref().is_some_and(|h| h.is_finished()) {
                let _ = handle.take().unwrap().join();
            }
        }
        self.writer.is_none() && self.reader.is_none()
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        // Fallback for forced teardown or construction errors. Normal navigation
        // and window close finish the same state machine before dropping us.
        self.begin_shutdown();
        while !self.poll_shutdown() {
            thread::sleep(Duration::from_millis(10));
        }
    }
}
fn write_commands(mut pipe: impl Write, commands: mpsc::Receiver<String>) {
    while let Ok(command) = commands.recv() {
        if writeln!(pipe, "{command}").is_err() || command == "quit" {
            break;
        }
    }
}
pub struct Pinball {
    worker: Worker,
    pixels: Arc<Mutex<Option<FramePixels>>>,
    errors: mpsc::Receiver<String>,
    texture: Option<egui::TextureHandle>,
    error: Option<String>,
    focused: bool,
    input_enabled: bool,
    mouse_down: bool,
    held_keys: HashSet<Key>,
    started: Instant,
    requested_size: [usize; 2],
    resize_candidate: ([usize; 2], Instant),
    table: Table,
    data_dir: Option<PathBuf>,
    config: PathBuf,
    dialog: Option<TableDialog>,
    switching: Option<(Table, Option<PathBuf>)>,
    /// A changed table is remembered only once it has drawn a frame, so a
    /// folder that cannot run is never what the next visit starts with.
    unsaved: bool,
    notice: Option<String>,
}
impl Pinball {
    pub fn begin_shutdown(&mut self) {
        self.worker.begin_shutdown();
    }
    pub fn poll_shutdown(&mut self) -> bool {
        self.worker.poll_shutdown()
    }

    pub fn ready(&self) -> bool {
        self.texture.is_some() || self.error.is_some()
    }
    pub fn finished(&mut self) -> bool {
        // The outgoing worker of a table change exits cleanly; that is not
        // the player finishing with pinball.
        if self.switching.is_some() {
            return false;
        }
        self.worker
            .child
            .try_wait()
            .ok()
            .flatten()
            .is_some_and(|s| s.success())
    }
    fn release_pointer(&mut self) {
        if self.mouse_down {
            // Release outside the worker UI so a blocked click cannot activate a menu item.
            self.send("mouse -1 -1 0".into());
            self.mouse_down = false;
        }
        self.send("mouse -1 -1 -1".into());
    }
    pub fn set_input_enabled(&mut self, enabled: bool) {
        if self.input_enabled && !enabled {
            self.release_pointer();
        }
        self.input_enabled = enabled;
    }
    pub fn pause(&mut self) {
        self.release_pointer();
        self.held_keys.clear();
        self.send("blur".into());
    }
    fn spawn(ctx: &egui::Context, classic: Option<&Path>) -> io::Result<Link> {
        let mut child = worker_command(&helper()?, classic)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .env_remove("OMARCHY_TEST_COMPACT")
            .env_remove("OMARCHY_TEST_TICKS")
            .env_remove("OMARCHY_TEST_SHOT")
            .env_remove("OMARCHY_TEST_SCREENSHOT")
            .spawn()?;
        let pipe = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("No Circuit input"))?;
        let (input, commands) = mpsc::sync_channel::<String>(256);
        let writer = thread::spawn(move || write_commands(pipe, commands));
        let mut output = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("No Circuit output"))?;
        let pixels = Arc::new(Mutex::new(None));
        let latest = pixels.clone();
        let ctx = ctx.clone();
        let (tx, errors) = mpsc::channel();
        let reader = thread::spawn(move || loop {
            match read_frame(&mut output) {
                Ok(bytes) => {
                    if let Ok(mut p) = latest.lock() {
                        *p = Some(bytes);
                    }
                    ctx.request_repaint();
                }
                Err(e) => {
                    let _ = tx.send(format!("Pinball stopped: {e}"));
                    ctx.request_repaint();
                    break;
                }
            }
        });
        Ok(Link {
            worker: Worker {
                shutdown: Shutdown::default(),
                child,
                input: Some(input),
                writer: Some(writer),
                reader: Some(reader),
            },
            pixels,
            errors,
        })
    }
    pub fn new(ctx: &egui::Context) -> io::Result<Self> {
        let config = config_dir();
        let data_dir = find_data_dir(&config, &home_join("XDG_DATA_HOME", ".local/share"));
        // Remembered classic play needs its folder to still be there.
        let table = match (saved_table(&config), &data_dir) {
            (Table::Classic, Some(_)) => Table::Classic,
            _ => Table::Circuit,
        };
        let classic = data_dir.as_deref().filter(|_| table == Table::Classic);
        let link = Self::spawn(ctx, classic)?;
        Ok(Self {
            worker: link.worker,
            pixels: link.pixels,
            errors: link.errors,
            texture: None,
            error: None,
            focused: true,
            input_enabled: true,
            mouse_down: false,
            held_keys: HashSet::new(),
            started: Instant::now(),
            requested_size: [WIDTH, HEIGHT],
            resize_candidate: ([WIDTH, HEIGHT], Instant::now()),
            table,
            data_dir,
            config,
            dialog: None,
            switching: None,
            unsaved: false,
            notice: None,
        })
    }
    /// Host navigation bar: the current table and the way to change it.
    pub fn toolbar(&mut self, ui: &mut egui::Ui) {
        let label = match self.table {
            Table::Circuit => "Table: Circuit",
            Table::Classic => "Table: Classic",
        };
        let button = ui
            .button(label)
            .on_hover_text("Choose the Circuit table or your own Space Cadet files");
        if button.has_focus() {
            // Space launches the ball; it must never re-press this button.
            button.surrender_focus();
        }
        if button.clicked() && self.dialog.is_none() && self.switching.is_none() {
            self.pause();
            self.dialog = Some(TableDialog {
                choice: self.table,
                folder: self
                    .data_dir
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default(),
                problem: None,
                browse: None,
            });
        }
        if let Some(notice) = &self.notice {
            ui.label(notice);
        }
    }
    fn begin_switch(&mut self, table: Table, data: Option<PathBuf>) {
        self.release_pointer();
        self.held_keys.clear();
        self.worker.begin_shutdown();
        self.switching = Some((table, data));
        self.texture = None;
        self.error = None;
    }
    /// Start the chosen table once the previous worker has fully stopped, so
    /// two engines never share the audio device or a settings file.
    fn poll_switch(&mut self, ctx: &egui::Context) {
        if self.switching.is_none() || !self.worker.poll_shutdown() {
            return;
        }
        let (table, data) = self.switching.take().expect("pending table");
        let classic = data.as_deref().filter(|_| table == Table::Classic);
        match Self::spawn(ctx, classic) {
            Ok(link) => {
                self.worker = link.worker;
                self.pixels = link.pixels;
                self.errors = link.errors;
                self.table = table;
                if table == Table::Classic {
                    self.data_dir = data;
                }
                self.started = Instant::now();
                self.requested_size = [WIDTH, HEIGHT];
                self.unsaved = true;
            }
            Err(e) => self.error = Some(format!("Could not start the table: {e}")),
        }
    }
    fn table_dialog(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.dialog.as_mut() else {
            return;
        };
        if let Some(Ok(result)) = dialog.browse.as_ref().map(|r| r.try_recv()) {
            if let Some(folder) = result {
                dialog.folder = folder;
            }
            dialog.browse = None;
        }
        let mut play = false;
        let mut cancel = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape));
        egui::Modal::new(egui::Id::new("pinball-table")).show(ctx, |ui| {
            ui.set_max_width(520.);
            ui.heading("Pinball table");
            ui.radio_value(&mut dialog.choice, Table::Circuit, "Circuit (included)");
            ui.radio_value(
                &mut dialog.choice,
                Table::Classic,
                "Classic Space Cadet (your own game files)",
            );
            if dialog.choice == Table::Classic {
                ui.label(
                    "Omarchy Arcade does not include the original game. Enter the folder \
                     that holds your copy: PINBALL.DAT (or CADET.DAT) with its sound files.",
                );
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut dialog.folder)
                            .hint_text("/path/to/your/Space Cadet folder")
                            .desired_width(360.),
                    );
                    if dialog.browse.is_none() && ui.button("Browse…").clicked() {
                        let (tx, rx) = mpsc::channel();
                        let ctx = ctx.clone();
                        thread::spawn(move || {
                            let chosen = Command::new("zenity")
                                .args(["--file-selection", "--directory"])
                                .arg("--title=Choose your Space Cadet folder")
                                .output()
                                .ok()
                                .filter(|o| o.status.success())
                                .and_then(|o| String::from_utf8(o.stdout).ok())
                                .map(|s| s.trim().to_owned())
                                .filter(|s| !s.is_empty());
                            let _ = tx.send(chosen);
                            ctx.request_repaint();
                        });
                        dialog.browse = Some(rx);
                    }
                });
                ui.small("Browse needs zenity; typing or pasting the path always works.");
            }
            if let Some(problem) = &dialog.problem {
                ui.colored_label(ui.visuals().warn_fg_color, problem);
            }
            ui.label("Changing table ends the current game. High scores and settings are kept.");
            ui.horizontal(|ui| {
                play = ui.button("Play this table").clicked();
                cancel |= ui.button("Cancel (Esc)").clicked();
            });
        });
        if cancel {
            self.dialog = None;
            return;
        }
        if !play {
            return;
        }
        let choice = dialog.choice;
        let folder = expand(&dialog.folder);
        if choice == Table::Classic && !valid_data_dir(&folder) {
            dialog.problem =
                Some("That folder has no PINBALL.DAT, CADET.DAT or DEMO.DAT in it.".into());
            return;
        }
        self.dialog = None;
        let unchanged = choice == self.table
            && (choice == Table::Circuit || self.data_dir.as_deref() == Some(folder.as_path()));
        if !unchanged {
            self.begin_switch(choice, (choice == Table::Classic).then_some(folder));
        }
    }
    fn send(&mut self, command: String) {
        if self.error.is_none() {
            if let Err(e) = self
                .worker
                .input
                .as_ref()
                .expect("live worker input")
                .try_send(command)
            {
                self.error = Some(format!("Circuit input failed: {e}"));
            }
        }
    }
}
fn keycode(key: Key) -> Option<i32> {
    Some(match key {
        Key::A | Key::Z => 97,
        Key::D | Key::Slash => 100,
        Key::X => 120,
        Key::Space => 32,
        Key::P => 112,
        Key::Escape => 27,
        Key::Enter => 13,
        Key::Tab => 9,
        Key::Backspace => 8,
        Key::Delete => 127,
        Key::Home => 1073741898,
        Key::End => 1073741901,
        Key::Y => 121,
        Key::ArrowLeft => 1073741904,
        Key::ArrowRight => 1073741903,
        Key::ArrowUp => 1073741906,
        Key::ArrowDown => 1073741905,
        Key::F1 => 1073741882,
        Key::F2 => 1073741883,
        Key::N => 110,
        Key::M => 109,
        Key::Comma => 44,
        Key::Period => 46,
        Key::F5 => 1073741886,
        Key::F6 => 1073741887,
        Key::F8 => 1073741889,
        _ => return None,
    })
}
// Text is distinct from gameplay key presses. Hex keeps UTF-8, spaces and
// punctuation intact without allowing pasted newlines to become bridge commands.
fn text_command(event: &egui::Event) -> Option<String> {
    let (operation, text) = match event {
        egui::Event::Text(text) => ("text", text),
        egui::Event::Paste(text) => ("paste", text),
        _ => return None,
    };
    let text: String = text.chars().filter(|c| !c.is_control()).take(256).collect();
    if text.is_empty() {
        return None;
    }
    let mut command = format!("{operation} ");
    for byte in text.bytes() {
        write!(command, "{byte:02x}").expect("writing to a string");
    }
    Some(command)
}
// Keep text-editing shortcuts distinct from the classic flipper aliases.
fn physical_keycode(key: Key) -> Option<i32> {
    match key {
        Key::Z => Some(122),
        Key::Slash => Some(47),
        _ => keycode(key),
    }
}
// Multiple physical keys may hold the same logical control. Only emit edges.
fn key_transition(held: &mut HashSet<Key>, key: Key, pressed: bool) -> Option<i32> {
    let code = keycode(key)?;
    let was_down = held.iter().any(|k| keycode(*k) == Some(code));
    if pressed {
        held.insert(key);
    } else {
        held.remove(&key);
    }
    let is_down = held.iter().any(|k| keycode(*k) == Some(code));
    (was_down != is_down).then_some(code)
}
impl eframe::App for Pinball {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.poll_switch(ctx);
        self.table_dialog(ctx);
        if let Ok(e) = self.errors.try_recv() {
            // The outgoing worker's end of stream is expected during a change.
            if self.switching.is_none() {
                self.error = Some(e);
            }
        }
        if self.switching.is_some() {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(5, 8, 8)))
                .show(ctx, |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label("Changing table…");
                    });
                });
            ctx.request_repaint_after(Duration::from_millis(10));
            return;
        }
        if self.texture.is_none() && self.started.elapsed() > Duration::from_secs(15) {
            self.error.get_or_insert(
                "Pinball did not produce a frame. Return to Arcade and try again.".into(),
            );
        }
        if let Some(bytes) = self.pixels.lock().ok().and_then(|mut p| p.take()) {
            let image = egui::ColorImage::from_rgba_unmultiplied(bytes.size, &bytes.bytes);
            if let Some(t) = &mut self.texture {
                t.set(image, egui::TextureOptions::LINEAR);
            } else {
                self.texture =
                    Some(ctx.load_texture("circuit-live", image, egui::TextureOptions::LINEAR));
            }
            if std::mem::take(&mut self.unsaved) {
                self.notice = save_choice(&self.config, self.table, self.data_dir.as_deref())
                    .err()
                    .map(|e| format!("Table choice was not saved: {e}"));
            }
        }
        let focused = ctx.input(|i| i.focused) && self.input_enabled && self.dialog.is_none();
        if self.focused && !focused {
            self.pause();
        }
        self.focused = focused;
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(5, 8, 8)))
            .show(ctx, |ui| {
                if let Some(error) = self.error.clone() {
                    ui.heading("Pinball needs attention");
                    ui.label(error);
                    if self.table == Table::Classic {
                        ui.label(
                            "The classic table could not run from the chosen folder. Check \
                             that it holds a complete copy of the original game.",
                        );
                        if ui.button("Play Circuit instead").clicked() {
                            self.begin_switch(Table::Circuit, None);
                        }
                    }
                    ui.label("Use Back to Arcade to choose another game.");
                    return;
                }
                let desired = surface_size(ui.available_size());
                if desired != self.resize_candidate.0 {
                    self.resize_candidate = (desired, Instant::now());
                }
                if desired != self.requested_size
                    && self.resize_candidate.1.elapsed() >= Duration::from_millis(100)
                {
                    self.send(format!("resize {} {}", desired[0], desired[1]));
                    self.requested_size = desired;
                }
                let Some(texture) = &self.texture else {
                    ui.centered_and_justified(|ui| {
                        ui.spinner();
                    });
                    return;
                };
                let size = texture.size_vec2();
                let scale = (ui.available_width() / size.x).min(ui.available_height() / size.y);
                let available = ui.available_rect_before_wrap();
                let rect = egui::Rect::from_center_size(available.center(), size * scale);
                let image = ui.put(
                    rect,
                    egui::Image::new(texture)
                        .fit_to_exact_size(rect.size())
                        .sense(egui::Sense::click_and_drag()),
                );
                let rect = image.rect;
                if !focused {
                    return;
                }
                // Ownership matters: coordinates alone include overlapping host dialogs.
                let hovered = image.contains_pointer();
                // Modifier keys have no egui Key event of their own. In particular,
                // release Ctrl after Select All even when the next letter isn't a game key.
                let mods = ctx.input(|i| i.modifiers);
                self.send(format!(
                    "modifiers {}",
                    if mods.ctrl { 0x40 } else { 0 }
                        | if mods.shift { 1 } else { 0 }
                        | if mods.alt { 0x100 } else { 0 }
                ));
                for event in ctx.input(|i| i.events.clone()) {
                    match event {
                        egui::Event::Text(_) | egui::Event::Paste(_) => {
                            if let Some(command) = text_command(&event) {
                                self.send(command);
                            }
                        }
                        egui::Event::Key {
                            key,
                            pressed,
                            repeat: false,
                            modifiers,
                            ..
                        } if focused => {
                            if let Some(code) = key_transition(&mut self.held_keys, key, pressed) {
                                let mods = if modifiers.ctrl { 0x40 } else { 0 }
                                    | if modifiers.shift { 1 } else { 0 }
                                    | if modifiers.alt { 0x100 } else { 0 };
                                let physical = physical_keycode(key).expect("mapped key");
                                self.send(format!(
                                    "key {code} {} {mods} {physical}",
                                    i32::from(pressed)
                                ));
                            }
                        }
                        egui::Event::PointerMoved(pos) if hovered => {
                            self.send(format!(
                                "mouse {} {} -1",
                                ((pos.x - rect.min.x) / scale) as i32,
                                ((pos.y - rect.min.y) / scale) as i32
                            ));
                        }
                        egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed,
                            ..
                        } if (pressed && hovered) || (!pressed && self.mouse_down) => {
                            self.mouse_down = pressed;
                            self.send(format!(
                                "mouse {} {} {}",
                                ((pos.x - rect.min.x) / scale) as i32,
                                ((pos.y - rect.min.y) / scale) as i32,
                                i32::from(pressed)
                            ));
                        }
                        egui::Event::PointerMoved(_) | egui::Event::PointerGone
                            if !self.mouse_down =>
                        {
                            self.send("mouse -1 -1 -1".into());
                        }
                        egui::Event::MouseWheel { delta, .. } if hovered => {
                            let direction = delta.y.signum() as i32;
                            if direction != 0 {
                                self.send(format!("wheel {direction}"));
                            }
                        }
                        _ => {}
                    }
                }
            });
        ctx.request_repaint_after(Duration::from_millis(33));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn classic_data_folder_is_validated_found_and_remembered() {
        let root = tempfile::tempdir().unwrap();
        let (config, data_home) = (root.path().join("config"), root.path().join("data"));
        let (empty, game) = (root.path().join("empty"), root.path().join("game"));
        std::fs::create_dir_all(&empty).unwrap();
        std::fs::create_dir_all(&game).unwrap();
        std::fs::write(game.join("pinball.dat"), b"x").unwrap();
        assert!(!valid_data_dir(&empty) && !valid_data_dir(&root.path().join("missing")));
        assert!(valid_data_dir(&game));
        // Nothing saved and nothing installed: Circuit, no folder.
        assert_eq!(saved_table(&config), Table::Circuit);
        assert_eq!(find_data_dir(&config, &data_home), None);
        // Choosing Circuit never records a folder.
        save_choice(&config, Table::Circuit, Some(&game)).unwrap();
        assert_eq!(find_data_dir(&config, &data_home), None);
        save_choice(&config, Table::Classic, Some(&game)).unwrap();
        assert_eq!(saved_table(&config), Table::Classic);
        assert_eq!(find_data_dir(&config, &data_home), Some(game.clone()));
        // Returning to Circuit keeps the folder for next time.
        save_choice(&config, Table::Circuit, None).unwrap();
        assert_eq!(saved_table(&config), Table::Circuit);
        assert_eq!(find_data_dir(&config, &data_home), Some(game.clone()));
        // A saved folder that has since lost its data is not offered.
        std::fs::remove_file(game.join("pinball.dat")).unwrap();
        assert_eq!(find_data_dir(&config, &data_home), None);
        // The standalone launcher's default location is found without saving.
        let installed = data_home.join("omarchy-spacecadet");
        std::fs::create_dir_all(&installed).unwrap();
        std::fs::write(installed.join("CADET.DAT"), b"x").unwrap();
        assert_eq!(find_data_dir(&config, &data_home), Some(installed));
    }
    #[test]
    fn classic_worker_runs_the_unmodified_engine_in_the_data_folder() {
        let args = |c: &Command| {
            c.get_args()
                .map(|a| a.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
        };
        let circuit = worker_command(Path::new("/engine"), None);
        assert_eq!(args(&circuit), ["--arcade-bridge", "--omarchy-table"]);
        assert_eq!(circuit.get_current_dir(), None);
        let classic = worker_command(Path::new("/engine"), Some(Path::new("/games/cadet")));
        assert_eq!(args(&classic), ["--arcade-bridge"]);
        assert_eq!(classic.get_current_dir(), Some(Path::new("/games/cadet")));
    }
    #[test]
    fn names_preserve_utf8_and_cannot_inject_bridge_commands() {
        assert_eq!(
            text_command(&egui::Event::Text("Riél O'Neil".into())),
            Some("text 5269c3a96c204f274e65696c".into())
        );
        assert_eq!(
            text_command(&egui::Event::Paste("Riél O'Neil".into())),
            Some("paste 5269c3a96c204f274e65696c".into())
        );
        assert_eq!(
            text_command(&egui::Event::Paste("A\nquit\0".into())),
            Some("paste 4171756974".into())
        );
        assert_eq!(text_command(&egui::Event::Paste("\n\0".into())), None);
        assert_eq!(
            text_command(&egui::Event::Paste("é".repeat(300))).unwrap(),
            format!("paste {}", "c3a9".repeat(256))
        );
        assert_eq!(physical_keycode(Key::Z), Some(122));
        assert_eq!(keycode(Key::Z), Some(97));
        assert_eq!(physical_keycode(Key::Slash), Some(47));
    }
    #[test]
    fn worker_shutdown_is_bounded() {
        for scenario in ["full", "blocked-write", "exited", "graceful", "fallback"] {
            let mut fixture = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "pinball::tests::worker_shutdown_fixture",
                    "--ignored",
                ])
                .env("ARCADE_SHUTDOWN_SCENARIO", scenario)
                .spawn()
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(8);
            loop {
                if let Some(status) = fixture.try_wait().unwrap() {
                    assert!(status.success(), "shutdown scenario {scenario}");
                    break;
                }
                if Instant::now() >= deadline {
                    let _ = fixture.kill();
                    let _ = fixture.wait();
                    panic!("shutdown exceeded deadline: {scenario}");
                }
                thread::sleep(Duration::from_millis(20));
            }
        }
    }

    // Only the supervising test invokes this, so a regression cannot hang CI.
    #[test]
    #[ignore]
    fn worker_shutdown_fixture() {
        let scenario = std::env::var("ARCADE_SHUTDOWN_SCENARIO").unwrap();
        let mut command = if scenario == "graceful" {
            let mut command = Command::new("sh");
            command.args([
                "-c",
                "while read -r line; do [ \"$line\" = quit ] && exit 0; done; exit 0",
            ]);
            command
        } else {
            let mut command = Command::new("sleep");
            command.arg("30");
            command
        };
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let pipe = child.stdin.take().unwrap();
        let mut output = child.stdout.take().unwrap();
        let reader = thread::spawn(move || {
            let _ = output.read_to_end(&mut Vec::new());
        });
        if scenario == "exited" {
            child.kill().unwrap();
            child.wait().unwrap();
        }
        let (input, commands) = mpsc::sync_channel(1);
        input
            .send(if scenario == "blocked-write" {
                "x".repeat(1_000_000)
            } else {
                "blur".into()
            })
            .unwrap();
        let writer = thread::spawn(move || {
            // Keep the queue full until Drop has attempted its nonblocking quit.
            thread::sleep(Duration::from_millis(200));
            write_commands(pipe, commands);
        });
        let mut worker = Worker {
            shutdown: Shutdown::default(),
            child,
            input: Some(input),
            writer: Some(writer),
            reader: Some(reader),
        };
        if scenario == "fallback" {
            drop(worker);
            return;
        }
        let start = Instant::now();
        worker.begin_shutdown();
        assert!(start.elapsed() < Duration::from_millis(500));
        let deadline = worker.shutdown.deadline;
        worker.begin_shutdown();
        assert_eq!(
            worker.shutdown.deadline, deadline,
            "repeated close must not extend grace"
        );
        let mut pending_frames = 0;
        loop {
            let frame = Instant::now();
            let done = worker.poll_shutdown();
            assert!(
                frame.elapsed() < Duration::from_millis(500),
                "poll blocked a frame"
            );
            if done {
                break;
            }
            pending_frames += 1;
            thread::sleep(Duration::from_millis(10));
        }
        assert!(
            pending_frames > 0,
            "fixture should allow UI frames during cleanup"
        );
        let status = worker.child.try_wait().unwrap().unwrap();
        if scenario == "graceful" {
            assert!(status.success(), "cooperative child should not be killed");
            assert!(start.elapsed() < Duration::from_secs(1));
        }
        assert!(worker.writer.is_none() && worker.reader.is_none());
        drop(worker);
    }

    #[test]
    fn classic_flippers_share_press_release_state() {
        for (modern, classic, code) in [(Key::A, Key::Z, 97), (Key::D, Key::Slash, 100)] {
            let mut held = HashSet::new();
            assert_eq!(key_transition(&mut held, classic, true), Some(code));
            assert_eq!(key_transition(&mut held, classic, true), None);
            assert_eq!(key_transition(&mut held, modern, true), None);
            assert_eq!(key_transition(&mut held, classic, false), None);
            assert_eq!(key_transition(&mut held, modern, false), Some(code));
            assert_eq!(key_transition(&mut held, classic, false), None);
            // Focus loss clears the host state along with the engine's blur.
            held.clear();
            assert_eq!(key_transition(&mut held, classic, true), Some(code));
        }
    }
    #[test]
    fn surface_budget_preserves_tall_and_wide_aspects() {
        for size in [
            egui::vec2(941.0, 984.0),
            egui::vec2(3840.0, 2160.0),
            egui::vec2(640.0, 1200.0),
            egui::vec2(8000.0, 64.0),
        ] {
            let actual = surface_size(size);
            assert!(valid_size(actual), "{actual:?}");
        }
        let tall = surface_size(egui::vec2(941.0, 984.0));
        assert!(tall[1] > tall[0]);
        assert!(!valid_size([1600, 1600]));
        assert!(!valid_size([0, 790]));
    }
    #[test]
    fn accepts_resized_frame() {
        let mut data = b"OAR1".to_vec();
        data.extend(640_u32.to_le_bytes());
        data.extend(960_u32.to_le_bytes());
        data.resize(12 + 640 * 960 * 4, 255);
        assert_eq!(read_frame(&mut data.as_slice()).unwrap().size, [640, 960]);
    }
    #[test]
    fn rejects_unbounded_or_unknown_frames() {
        assert!(read_frame(&mut &b"OAR1\xff\xff\xff\xff\xff\xff\xff\xff"[..]).is_err());
        assert!(read_frame(&mut &b"bad!\x80\x04\0\0\x16\x03\0\0"[..]).is_err());
    }
    #[test]
    fn complete_and_truncated_frames() {
        let mut data = b"OAR1\x80\x04\0\0\x16\x03\0\0".to_vec();
        data.resize(12 + WIDTH * HEIGHT * 4, 127);
        assert_eq!(
            read_frame(&mut data.as_slice()).unwrap().bytes.len(),
            WIDTH * HEIGHT * 4
        );
        data.pop();
        assert!(read_frame(&mut data.as_slice()).is_err());
    }
}
