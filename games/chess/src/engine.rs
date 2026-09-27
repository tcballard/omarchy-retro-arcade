use crate::game::{Difficulty, Game};
use std::{
    env, fs,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

pub fn find_engine() -> Option<PathBuf> {
    if let Some(path) = env::var_os("OMARCHY_CHESS_ENGINE") {
        return find_executable(Path::new(&path));
    }
    env::current_exe()
        .ok()
        .and_then(|p| {
            p.parent()
                .map(|d| d.join("../libexec/omarchy-retro-arcade/stockfish"))
        })
        .and_then(|p| find_executable(&p))
        .or_else(|| find_executable(Path::new("stockfish")))
        .or_else(|| find_executable(Path::new("/usr/games/stockfish")))
        .or_else(|| find_executable(Path::new("/usr/lib/omarchy-chess/stockfish")))
}
fn executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}
fn find_executable(path: &Path) -> Option<PathBuf> {
    if path.components().count() > 1 {
        return executable(path).then(|| path.to_owned());
    }
    env::split_paths(&env::var_os("PATH").unwrap_or_default())
        .map(|p| p.join(path))
        .find(|p| executable(p))
}
#[derive(Debug)]
pub struct Answer {
    pub revision: u64,
    pub hint: bool,
    pub result: Result<String, String>,
}
struct Job {
    cancel: Arc<AtomicBool>,
    thread: JoinHandle<()>,
}
#[derive(Default)]
pub struct Engine {
    active: Option<Job>,
    retired: Vec<Job>,
    pub busy: bool,
    tx: Option<Sender<Answer>>,
    rx: Option<Receiver<Answer>>,
}
impl Engine {
    pub fn start(&mut self, game: &Game, revision: u64, hint: bool) {
        self.start_with_path(game, revision, hint, find_engine());
    }
    pub fn start_with_path(
        &mut self,
        game: &Game,
        revision: u64,
        hint: bool,
        path: Option<PathBuf>,
    ) {
        self.cancel();
        if self.tx.is_none() {
            let (tx, rx) = mpsc::channel();
            self.tx = Some(tx);
            self.rx = Some(rx);
        }
        let tx = self.tx.as_ref().unwrap().clone();
        let game = game.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        self.busy = true;
        let thread = thread::spawn(move || {
            let result=path.ok_or_else(||"Stockfish not found. Install it or set OMARCHY_CHESS_ENGINE to its executable; local play still works.".to_string())
                .and_then(|path|search(&path,&game,if hint{Difficulty::Strong}else{game.difficulty},&worker_cancel));
            if !worker_cancel.load(Ordering::Relaxed) {
                let _ = tx.send(Answer {
                    revision,
                    hint,
                    result,
                });
            }
        });
        self.active = Some(Job { cancel, thread });
    }
    pub fn cancel(&mut self) {
        if let Some(job) = self.active.take() {
            job.cancel.store(true, Ordering::Relaxed);
            self.retired.push(job);
        }
        self.busy = false;
        self.reap();
    }
    fn reap(&mut self) {
        let mut i = 0;
        while i < self.retired.len() {
            if self.retired[i].thread.is_finished() {
                let job = self.retired.swap_remove(i);
                let _ = job.thread.join();
            } else {
                i += 1;
            }
        }
    }
    pub fn poll(&mut self) -> Vec<Answer> {
        self.reap();
        let mut results = vec![];
        if let Some(rx) = &self.rx {
            while let Ok(answer) = rx.try_recv() {
                results.push(answer);
            }
        }
        if self.active.as_ref().is_some_and(|j| j.thread.is_finished()) {
            let j = self.active.take().unwrap();
            let _ = j.thread.join();
        }
        results
    }
    pub fn complete(&mut self) {
        self.busy = false;
    }
}
impl Drop for Engine {
    fn drop(&mut self) {
        self.cancel();
        for job in self.retired.drain(..) {
            let _ = job.thread.join();
        }
    }
}
struct Process {
    child: Child,
    input: ChildStdin,
    lines: Receiver<Result<String, String>>,
    reader: Option<JoinHandle<()>>,
}
impl Process {
    fn spawn(path: &Path) -> Result<Self, String> {
        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Could not start Stockfish: {e}"))?;
        let input = child.stdin.take().ok_or("Engine stdin unavailable")?;
        let output = child.stdout.take().ok_or("Engine stdout unavailable")?;
        let (tx, lines) = mpsc::channel();
        let reader = thread::spawn(move || {
            let mut reader = BufReader::new(output);
            let mut options = 0;
            loop {
                let mut bytes = Vec::new();
                let read = std::io::Read::by_ref(&mut reader)
                    .take(65537)
                    .read_until(b'\n', &mut bytes);
                match read {
                    Ok(0) => {
                        let _ = tx.send(Err("Stockfish exited before returning a move.".into()));
                        break;
                    }
                    Ok(_) if bytes.len() > 65536 => {
                        let _ = tx.send(Err("Engine response is too large.".into()));
                        break;
                    }
                    Ok(_) => {
                        let line = String::from_utf8_lossy(&bytes).trim().to_string();
                        if line.starts_with("option name ") {
                            options += 1;
                            if options > 256 {
                                let _ = tx.send(Err("Too many engine options.".into()));
                                break;
                            }
                        }
                        if (line == "uciok"
                            || line == "readyok"
                            || line.starts_with("bestmove ")
                            || line.starts_with("option name "))
                            && tx.send(Ok(line)).is_err()
                        {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e.to_string()));
                        break;
                    }
                }
            }
        });
        let process = Self {
            child,
            input,
            lines,
            reader: Some(reader),
        };
        let flags = rustix::fs::fcntl_getfl(&process.input).map_err(|e| e.to_string())?;
        rustix::fs::fcntl_setfl(&process.input, flags | rustix::fs::OFlags::NONBLOCK)
            .map_err(|e| e.to_string())?;
        Ok(process)
    }
    fn send(&mut self, line: &str, cancel: &AtomicBool) -> Result<(), String> {
        let command = format!("{line}\n");
        let mut bytes = command.as_bytes();
        let deadline = Instant::now() + Duration::from_secs(3);
        while !bytes.is_empty() {
            if cancel.load(Ordering::Relaxed) {
                return Err("Cancelled".into());
            }
            if Instant::now() >= deadline {
                return Err("Stockfish stopped reading commands.".into());
            }
            match self.input.write(bytes) {
                Ok(0) => return Err("Engine input closed.".into()),
                Ok(n) => bytes = &bytes[n..],
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10))
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => (),
                Err(e) => return Err(e.to_string()),
            }
        }
        Ok(())
    }
    fn until(
        &self,
        prefix: &str,
        timeout: Duration,
        cancel: &AtomicBool,
    ) -> Result<Vec<String>, String> {
        let deadline = Instant::now() + timeout;
        let mut lines = vec![];
        loop {
            if cancel.load(Ordering::Relaxed) {
                return Err("Cancelled".into());
            }
            if Instant::now() >= deadline {
                return Err("Stockfish timed out. Choose Retry engine.".into());
            }
            match self.lines.recv_timeout(Duration::from_millis(20)) {
                Ok(Ok(line)) => {
                    let done = line == prefix || line.starts_with(&format!("{prefix} "));
                    lines.push(line);
                    if done {
                        return Ok(lines);
                    }
                }
                Ok(Err(e)) => return Err(e),
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err("Engine connection closed.".into())
                }
                Err(mpsc::RecvTimeoutError::Timeout) => (),
            }
        }
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}
pub fn search(
    path: &Path,
    game: &Game,
    difficulty: Difficulty,
    cancel: &AtomicBool,
) -> Result<String, String> {
    if cancel.load(Ordering::Relaxed) {
        return Err("Cancelled".into());
    }
    let mut p = Process::spawn(path)?;
    p.send("uci", cancel)?;
    // Loading Stockfish's evaluation network can exceed the move deadline on
    // a busy package builder. Keep the handshake bounded independently of play.
    const STARTUP_TIMEOUT: Duration = Duration::from_secs(10);
    let options = p.until("uciok", STARTUP_TIMEOUT, cancel)?;
    let (skill, millis) = difficulty.settings();
    for (name, value) in [("Threads", 1), ("Hash", 32), ("Skill Level", skill as u64)] {
        if options
            .iter()
            .any(|line| line.starts_with(&format!("option name {name} type ")))
        {
            p.send(&format!("setoption name {name} value {value}"), cancel)?;
        }
    }
    p.send("ucinewgame", cancel)?;
    p.send("isready", cancel)?;
    p.until("readyok", STARTUP_TIMEOUT, cancel)?;
    p.send(&game.uci_position(), cancel)?;
    p.send(&format!("go movetime {millis}"), cancel)?;
    let reply = p.until("bestmove", Duration::from_millis(millis + 3000), cancel)?;
    let text = reply
        .last()
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or("Engine returned no move")?;
    game.parse_move(text)?;
    Ok(text.into())
}
