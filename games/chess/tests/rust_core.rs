use omarchy_chess::{
    engine,
    game::{Difficulty, Game, Mode},
    storage::{self, Session, SessionLock},
    theme::{parse_color, Theme},
};
use shakmaty::{fen::Fen, CastlingMode, Position, Role, Square};
use std::{
    fs,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
fn play(g: &mut Game, moves: &[&str]) {
    for text in moves {
        let m = g.parse_move(text).unwrap();
        g.play(m).unwrap();
    }
}
fn custom(fen: &str) -> Game {
    Game::new(
        fen.parse::<Fen>()
            .unwrap()
            .into_position(CastlingMode::Standard)
            .unwrap(),
    )
}
#[test]
fn mate_and_illegal_rejection() {
    let mut g = Game::default();
    play(&mut g, &["f3", "e5", "g4", "Qh4#"]);
    assert!(g.finished());
    assert_eq!(g.result_text(), "0-1");
    assert!(g.parse_move("a2a3").and_then(|m| g.play(m)).is_err());
}
#[test]
fn castle_uses_standard_uci() {
    let mut g = Game::default();
    play(&mut g, &["e4", "e5", "Nf3", "Nc6", "Bc4", "Nf6", "O-O"]);
    assert_eq!(
        g.position().board().king_of(shakmaty::Color::White),
        Some(Square::G1)
    );
    assert!(g.uci_position().ends_with("e1g1"));
    assert_eq!(Game::from_pgn(&g.pgn()).unwrap().fen(), g.fen());
}
#[test]
fn en_passant() {
    let mut g = Game::default();
    play(&mut g, &["e4", "a6", "e5", "d5", "exd6"]);
    assert!(g.position().board().piece_at(Square::D5).is_none());
    assert_eq!(
        g.position().board().piece_at(Square::D6).unwrap().role,
        Role::Pawn
    );
}
#[test]
fn all_promotions() {
    for (suffix, role) in [
        ("q", Role::Queen),
        ("r", Role::Rook),
        ("b", Role::Bishop),
        ("n", Role::Knight),
    ] {
        let mut g = custom("7k/P7/8/8/8/8/8/7K w - - 0 1");
        play(&mut g, &[&format!("a7a8{suffix}")]);
        assert_eq!(
            g.position().board().piece_at(Square::A8).unwrap().role,
            role
        );
    }
}
#[test]
fn stalemate() {
    assert!(custom("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1")
        .status()
        .contains("Stalemate"));
}
#[test]
fn insufficient_material() {
    assert!(custom("7k/8/6K1/8/8/8/8/8 w - - 0 1").finished());
}
#[test]
fn seventy_five_move_rule() {
    assert!(custom("7k/8/6K1/8/8/8/8/R7 w - - 150 90").finished());
}
#[test]
fn fifty_move_claim() {
    let mut g = custom("7k/8/6K1/8/8/8/8/R7 w - - 100 90");
    assert!(!g.finished());
    g.claim_draw().unwrap();
    assert_eq!(g.result_text(), "1/2-1/2");
}
#[test]
fn intended_move_draw_claim() {
    let mut g = custom("7k/8/6K1/8/8/8/8/R7 w - - 99 90");
    assert!(g.can_claim_draw());
    g.claim_draw().unwrap();
}
#[test]
fn repetition_survives_resume() {
    let dir = tempfile::tempdir().unwrap();
    let mut g = Game {
        mode: Mode::Local,
        ..Game::default()
    };
    play(
        &mut g,
        &["Nf3", "Nf6", "Ng1", "Ng8", "Nf3", "Nf6", "Ng1", "Ng8"],
    );
    assert!(!g.finished());
    storage::save(dir.path(), &g, true, false).unwrap();
    let (mut restored, flip, guides) = storage::load(dir.path()).unwrap();
    assert!(flip && !guides);
    restored.claim_draw().unwrap();
    restored.takeback();
    assert!(!restored.finished());
}
#[test]
fn fivefold_is_automatic() {
    let mut g = Game::default();
    for _ in 0..4 {
        play(&mut g, &["Nf3", "Nf6", "Ng1", "Ng8"]);
    }
    assert!(g.status().contains("Fivefold"));
}
#[test]
fn undo_computer_reply_and_pending_turn() {
    let mut g = Game::default();
    play(&mut g, &["e4", "e5"]);
    g.takeback();
    assert!(g.moves.is_empty());
    play(&mut g, &["d4"]);
    g.takeback();
    assert!(g.moves.is_empty());
}
#[test]
fn black_takeback_and_resignation() {
    let mut g = Game {
        human_white: false,
        ..Game::default()
    };
    play(&mut g, &["e4"]);
    assert!(!g.can_takeback());
    play(&mut g, &["e5", "Nf3"]);
    g.takeback();
    assert_eq!(g.moves.len(), 1);
    g.resign();
    assert_eq!(g.result_text(), "1-0");
}
#[test]
fn invalid_pgn_does_not_import() {
    for pgn in [
        "",
        "[FEN \"8/8/8/8/8/8/8/8 w - - 0 1\"]\n\n*",
        "[Variant \"Atomic\"]\n\n1. e4 *",
        "1. e4 e5 2. Bh6 *",
        "[Result \"nonsense\"]\n\n*",
        "1. e4 *\n\n[Event \"Other\"]\n\n1. d4 *",
        "[Result \"1-0\"]\n\n1. e4 0-1",
    ] {
        assert!(Game::from_pgn(pgn).is_err(), "accepted {pgn}");
    }
}
#[test]
fn custom_fen_roundtrip() {
    let mut g = custom("7k/8/6K1/8/8/8/8/R7 w - - 0 1");
    play(&mut g, &["Ra2"]);
    let import = Game::from_pgn(&g.pgn()).unwrap();
    assert_eq!(import.fen(), g.fen());
    assert_eq!(import.positions[0], g.positions[0]);
}
#[test]
fn reject_move_after_result() {
    assert!(Game::from_pgn("1. f3 e5 2. g4 Qh4# 3. a3 0-1").is_err());
}
#[test]
fn comments_and_variations_ignore_mainline() {
    let g = Game::from_pgn("1. e4 {note} (1. d4 d5) e5 *").unwrap();
    assert_eq!(g.moves.len(), 2);
}
#[test]
fn session_validation() {
    let mut s = Session::capture(&Game::default(), false, true);
    s.version = 999;
    assert!(s.game().is_err());
    let mut s = Session::capture(&Game::default(), false, true);
    s.moves = vec!["e2e5".into()];
    assert!(s.game().is_err());
}
#[test]
fn corrupt_save_stays_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("session.json");
    fs::write(&p, b"bad json\xff").unwrap();
    assert!(storage::load(dir.path()).is_err());
    storage::archive(dir.path(), &Game::default(), true).unwrap();
    let backup = fs::read_dir(dir.path().join("archive"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert_eq!(fs::read(backup).unwrap(), b"bad json\xff");
    assert_eq!(fs::read(p).unwrap(), b"bad json\xff");
}
#[test]
fn legacy_python_save_migrates_and_backs_up() {
    let dir = tempfile::tempdir().unwrap();
    let mut g = Game::default();
    play(&mut g, &["e4", "e5"]);
    let old = serde_json::json!({"version":1,"pgn":g.pgn(),"mode":"computer","human":true,"difficulty":"Club","ending":"","flipped":true,"guides":false});
    fs::write(
        dir.path().join("session.json"),
        serde_json::to_vec(&old).unwrap(),
    )
    .unwrap();
    let (g2, flipped, guides) = storage::load(dir.path()).unwrap();
    assert_eq!(g.fen(), g2.fen());
    assert_eq!(g2.difficulty, Difficulty::Club);
    assert!(flipped && !guides);
    assert_eq!(fs::read_dir(dir.path().join("archive")).unwrap().count(), 1);
    storage::save(dir.path(), &g2, flipped, guides).unwrap();
    assert_eq!(storage::load(dir.path()).unwrap().0.fen(), g.fen());
}
#[test]
fn session_lock_is_exclusive() {
    let dir = tempfile::tempdir().unwrap();
    let lock = SessionLock::acquire(dir.path()).unwrap();
    assert!(SessionLock::acquire(dir.path()).is_err());
    drop(lock);
    assert!(SessionLock::acquire(dir.path()).is_ok());
}
#[test]
fn block_concurrent_python_preview() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("session.lock"), "legacy").unwrap();
    assert!(SessionLock::acquire(dir.path()).is_err());
}
#[test]
fn theme_validation() {
    assert!(parse_color("#123456").is_some());
    assert!(parse_color("#ééé").is_none());
    let t = Theme::parse("accent = '#123456'\nbackground='url(evil)'");
    assert_eq!(t.background, Theme::default().background);
    assert_ne!(t.accent, Theme::default().accent);
    assert_eq!(Theme::parse("not toml"), Theme::default());
}
#[test]
fn bounded_import() {
    assert!(Game::from_pgn(&" ".repeat(1_000_001)).is_err());
}
#[cfg(unix)]
fn engine_fixture(name: &str) -> std::path::PathBuf {
    // Do not write executables while parallel tests are spawning processes:
    // an inherited writable descriptor can make exec fail with ETXTBSY.
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}
#[cfg(unix)]
#[test]
fn engine_timeout_is_bounded() {
    let p = engine_fixture("silent-engine.sh");
    let start = Instant::now();
    let result = engine::search(
        &p,
        &Game::default(),
        Difficulty::Gentle,
        &AtomicBool::new(false),
    );
    let error = result.expect_err("an unresponsive engine must time out");
    assert!(
        error.contains("timed out"),
        "unexpected engine error: {error}"
    );
    let elapsed = start.elapsed();
    // Startup is allowed ten seconds for Stockfish to load its network on a
    // busy runner; an unresponsive engine must still fail in bounded time.
    assert!(elapsed < Duration::from_secs(12), "timeout took {elapsed:?}");
}
#[cfg(unix)]
#[test]
fn engine_cancel_is_bounded() {
    let p = engine_fixture("silent-engine.sh");
    let cancel = Arc::new(AtomicBool::new(false));
    let c = cancel.clone();
    let handle =
        std::thread::spawn(move || engine::search(&p, &Game::default(), Difficulty::Strong, &c));
    std::thread::sleep(Duration::from_millis(80));
    let start = Instant::now();
    cancel.store(true, Ordering::Relaxed);
    assert_eq!(handle.join().unwrap().unwrap_err(), "Cancelled");
    assert!(start.elapsed() < Duration::from_secs(1));
}
#[cfg(unix)]
#[test]
fn engine_illegal_move_rejected() {
    let p = engine_fixture("illegal-move-engine.sh");
    assert!(engine::search(
        &p,
        &Game::default(),
        Difficulty::Gentle,
        &AtomicBool::new(false)
    )
    .is_err());
}
#[test]
fn real_stockfish_move() {
    let Some(path) = engine::find_engine() else {
        assert!(
            std::env::var_os("REQUIRE_STOCKFISH").is_none(),
            "CI requires Stockfish"
        );
        return;
    };
    let g = Game::default();
    let text = engine::search(&path, &g, Difficulty::Gentle, &AtomicBool::new(false)).unwrap();
    assert!(g.parse_move(&text).is_ok());
}

#[test]
fn preferences_survive_updates_and_bad_settings_are_preserved() {
    use omarchy_chess::preferences::Preferences;
    let dir = tempfile::tempdir().unwrap();
    let legacy: Preferences = serde_json::from_str(r#"{"version":1,"sound":true}"#).unwrap();
    assert_eq!(
        legacy.piece_style,
        omarchy_chess::preferences::PieceStyle::AfterHours
    );
    let p = Preferences {
        piece_style: omarchy_chess::preferences::PieceStyle::Chisel,
        sound: true,
        follow_omarchy: false,
        ..Default::default()
    };
    p.save(dir.path()).unwrap();
    assert_eq!(Preferences::load(dir.path()).unwrap(), p);
    std::fs::write(dir.path().join("settings.json"), b"broken").unwrap();
    assert!(Preferences::load(dir.path()).is_err());
    assert_eq!(
        std::fs::read(dir.path().join("settings.json")).unwrap(),
        b"broken"
    );
}
#[test]
fn current_theme_precedes_legacy_and_survives_replacement() {
    use omarchy_chess::theme::Theme;
    let dir = tempfile::tempdir().unwrap();
    let current = dir.path().join("current.toml");
    let legacy = dir.path().join("legacy.toml");
    std::fs::write(&legacy, "background='#000000'\nforeground='#ffffff'").unwrap();
    let paths = [current.clone(), legacy];
    assert!(!Theme::load_from(&paths).unwrap().light());
    std::fs::write(
        &current,
        "background='#ffffff'\nforeground='#101010'\naccent='#002255'",
    )
    .unwrap();
    let theme = Theme::load_from(&paths).unwrap();
    assert!(theme.light());
    assert_eq!(theme.accent_text(), eframe::egui::Color32::WHITE);
    std::fs::write(&current, "invalid").unwrap();
    assert!(!Theme::load_from(&paths).unwrap().light());
}
#[test]
fn original_sound_cues_are_valid_bounded_pcm() {
    for finished in [false, true] {
        let wav = omarchy_chess::sound::cue(finished);
        assert_eq!(&wav[..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        let len = u32::from_le_bytes(wav[40..44].try_into().unwrap()) as usize;
        assert_eq!(len + 44, wav.len());
        assert!(len < 16000);
    }
}
