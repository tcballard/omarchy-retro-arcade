//! Write a resumable save for native checks: `fixture MAP WAVE TICKS PATH`.
//! It replays that map's winning Normal replay through the production engine
//! up to WAVE (1-based), then runs TICKS ticks of that wave. TICKS = 0 stops
//! while waiting for WAVE, before the replay spends anything on it. Earlier maps are
//! marked won so the chosen map is unlocked. Never point this at real saves.
use omarchy_ridgeline::{
    battle::{Command, Phase, Replay},
    data::Difficulty,
    storage::{self, Medal, Record, Save},
};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [map, wave, ticks, path] = &args[..] else {
        eprintln!("usage: fixture MAP WAVE TICKS PATH");
        std::process::exit(2);
    };
    let (map, wave, ticks): (usize, usize, u32) = (
        map.parse::<usize>().unwrap() - 1,
        wave.parse::<usize>().unwrap() - 1,
        ticks.parse().unwrap(),
    );
    let file = format!(
        "{}/tests/replays/{:02}-normal.json",
        env!("CARGO_MANIFEST_DIR"),
        map + 1
    );
    let replay: Replay = serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
    let mut save = Save::default();
    for m in 0..map {
        save.progress.records[m][Difficulty::Normal.index()] = Record {
            medal: Medal::Clear,
            best_health: 15,
            wins: 1,
        };
    }
    let mut b = omarchy_ridgeline::battle::Battle::new(map, Difficulty::Normal);
    for (tick, c) in &replay.commands {
        while b.tick < *tick && b.phase == Phase::Running {
            b.step();
        }
        if ticks == 0 && b.wave == wave && b.phase == Phase::Waiting {
            break;
        }
        if *c == Command::StartWave && b.wave == wave {
            b.apply(*c).unwrap();
            break;
        }
        b.apply(*c).unwrap();
    }
    for _ in 0..ticks {
        if b.phase == Phase::Running {
            b.step();
        }
    }
    save.battle = Some(b);
    storage::write(std::path::Path::new(path), &save).unwrap();
}
