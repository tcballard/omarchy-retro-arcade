//! Winning replays for every map and difficulty, run through the production engine.
use omarchy_ridgeline::{
    battle::{Battle, Clock, Phase, Replay, RULES},
    data::{maps, Difficulty, LAYOUT},
};

fn replays() -> Vec<Replay> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/replays");
    let mut all = vec![];
    for (i, _) in maps().iter().enumerate() {
        for d in Difficulty::ALL {
            let name = format!("{:02}-{}.json", i + 1, d.name().to_lowercase());
            let text = std::fs::read_to_string(dir.join(&name)).expect(&name);
            let replay: Replay = serde_json::from_str(&text).expect(&name);
            assert_eq!((replay.map, replay.difficulty), (i, d), "{name}");
            assert_eq!(
                (replay.rules, replay.layout),
                (RULES, LAYOUT),
                "{name} is stale"
            );
            all.push(replay);
        }
    }
    all
}

#[test]
fn every_map_and_difficulty_has_a_winning_replay() {
    for replay in replays() {
        let b = replay.run().unwrap();
        assert_eq!(
            b.phase,
            Phase::Victory,
            "map {} {:?}",
            replay.map + 1,
            replay.difficulty
        );
        assert!(b.health > 0);
    }
}

/// The same tick-stamped commands through a frame clock at 2× and uneven frames.
#[test]
fn replays_are_identical_at_double_speed_and_uneven_frames() {
    for replay in replays().into_iter().step_by(7) {
        let reference = replay.run().unwrap();
        let mut b = Battle::new(replay.map, replay.difficulty);
        let mut clock = Clock::default();
        let frames = [1. / 30., 1. / 144., 1. / 61., 0.2, 1. / 240.];
        let mut f = 0;
        let mut commands = replay.commands.iter().peekable();
        loop {
            while let Some((tick, c)) = commands.peek() {
                if *tick != b.tick
                    || (b.phase == Phase::Running
                        && *c == omarchy_ridgeline::battle::Command::StartWave)
                {
                    break;
                }
                b.apply(*c).unwrap();
                commands.next();
            }
            if b.phase != Phase::Running {
                if commands.peek().is_none() {
                    break;
                }
                continue;
            }
            // Advance by frames, but never past the next scheduled command.
            let n = clock.advance(frames[f % frames.len()], 2).unwrap();
            f += 1;
            for _ in 0..n {
                if b.phase != Phase::Running || commands.peek().is_some_and(|(t, _)| *t == b.tick) {
                    break;
                }
                b.step();
            }
        }
        assert_eq!(b, reference, "map {}", replay.map + 1);
    }
}
