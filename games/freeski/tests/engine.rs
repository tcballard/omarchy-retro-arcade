use omarchy_freeski::{
    collision::circle_interval,
    engine::*,
    storage::{self, Save},
    world::{self, Kind, Obstacle},
};
fn running() -> Sim {
    let mut s = Sim::default();
    s.start();
    s
}
fn obstacle(kind: Kind, x: f64, y: f64) -> Obstacle {
    Obstacle {
        id: 0,
        at: Point { x, y },
        kind,
    }
}
#[test]
fn acceleration_braking_bounded_turning_and_no_uphill() {
    let mut s = running();
    for _ in 0..600 {
        s.step(Input::default(), &[]);
    }
    assert_eq!(s.speed, MAX_SPEED);
    let before = s.position;
    s.step(
        Input {
            heading: MAX_HEADING,
            brake: false,
        },
        &[],
    );
    assert!((s.heading - TURN_RATE * DT).abs() < 1e-10);
    for _ in 0..180 {
        let y = s.position.y;
        s.step(
            Input {
                heading: MAX_HEADING,
                brake: true,
            },
            &[],
        );
        assert!(s.position.y >= y);
    }
    assert!(s.speed < 0.01);
    assert!(s.position.y > before.y);
    assert!(s.position.x.abs() <= world::HALF_WIDTH - RADIUS);
}
#[test]
fn swept_circle_catches_a_hazard_between_endpoints() {
    let hit = circle_interval(
        Point { x: 0., y: 0. },
        Point { x: 0., y: 100. },
        Point { x: 0., y: 50. },
        2.,
    )
    .unwrap();
    assert!((hit.0 - 0.48).abs() < 1e-10);
    assert!((hit.1 - 0.52).abs() < 1e-10);
    assert!(circle_interval(
        Point::default(),
        Point { x: 0., y: 100. },
        Point { x: 3., y: 50. },
        2.
    )
    .is_none());
}
#[test]
fn overlap_causes_one_crash_and_validated_recovery() {
    let items = vec![obstacle(Kind::Tree, 0., 1.), obstacle(Kind::Rock, 0., 1.)];
    let mut s = running();
    assert_eq!(s.step(Input::default(), &items), vec![Event::Crash]);
    assert_eq!(s.crashes, 1);
    assert!(clear(s.position, &items));
    for _ in 0..TUMBLE_TICKS {
        s.step(Input::default(), &items);
    }
    assert_eq!(s.protection, PROTECTION_TICKS);
    assert_eq!(s.crashes, 1);
    s.pause();
    let saved = s.clone();
    for _ in 0..500 {
        s.step(Input::default(), &items);
    }
    assert_eq!(s, saved);
    s.start();
    s.step(Input::default(), &items);
    assert_eq!(s.protection, PROTECTION_TICKS - 1);
}

#[test]
fn tick_segment_ends_at_contact_and_excludes_recovery_relocation() {
    let items = vec![obstacle(Kind::Tree, 0., 1.)];
    let mut sim = running();
    sim.speed = MAX_SPEED;
    let outcome = sim.step_outcome_mode(Input::default(), &items, Mode::Practice, None);
    assert_eq!(outcome.events, vec![Event::Crash]);
    assert!(outcome.segment.crash_fraction.is_some());
    assert!(outcome.segment.proposed_speed > 0.);
    assert_ne!(outcome.segment.end, sim.position);
    assert!(outcome.segment.end.y <= sim.distance);
}
#[test]
fn all_authored_hazards_allow_clear_recovery_without_forward_distance() {
    let items = world::practice();
    for o in items.iter().filter(|o| o.kind != Kind::Ramp) {
        let mut s = running();
        s.position = o.at;
        s.distance = s.position.y;
        s.step(Input::default(), &items);
        assert_eq!(s.crashes, 1);
        assert!(clear(s.position, &items), "{o:?}");
        assert!(s.position.y <= s.distance);
    }
}
#[test]
fn third_crash_ends_and_never_restarts_or_awards_finish() {
    let items = [obstacle(Kind::Tree, 0., world::FINISH)];
    let mut s = running();
    s.position.y = world::FINISH - 0.1;
    s.distance = s.position.y;
    s.speed = MAX_SPEED;
    s.crashes = 2;
    assert_eq!(s.step(Input::default(), &items), vec![Event::Crash]);
    assert_eq!(s.phase, Phase::Crashed);
    let stopped = s.clone();
    for _ in 0..100 {
        s.step(Input::default(), &items);
    }
    assert_eq!(s, stopped);
}
#[test]
fn ramp_launches_once_and_air_clears_rocks_but_not_trees() {
    let ramp = [obstacle(Kind::Ramp, 0., 0.)];
    let mut s = running();
    s.speed = MAX_SPEED;
    assert_eq!(s.step(Input::default(), &ramp), vec![Event::Jump]);
    assert!(s.jump.is_some());
    assert!(s.step(Input::default(), &ramp).is_empty());
    let mut airborne = running();
    airborne.speed = MAX_SPEED;
    airborne.jump = Some(0.6);
    let rocks = [obstacle(Kind::Rock, 0., 0.)];
    assert!(airborne.step(Input::default(), &rocks).is_empty());
    assert_eq!(airborne.crashes, 0);
    let trees = [obstacle(Kind::Tree, 0., airborne.position.y)];
    assert_eq!(airborne.step(Input::default(), &trees), vec![Event::Crash]);
}
#[test]
fn descending_contact_is_not_missed_when_overlap_begins_in_air() {
    let mut s = running();
    s.jump = Some(1.11);
    s.speed = 0.;
    let items = [obstacle(Kind::Rock, 0., 0.)];
    assert!(s.height() > 0.65);
    assert_eq!(
        s.step(
            Input {
                heading: 0.,
                brake: true
            },
            &items
        ),
        vec![Event::Crash]
    );
    assert_eq!(s.crashes, 1);
}
#[test]
fn repeated_pause_is_idempotent_and_ready_does_not_tick() {
    let mut s = Sim::default();
    s.step(Input::default(), &[]);
    assert_eq!(s, Sim::default());
    s.start();
    s.step(Input::default(), &[]);
    s.pause();
    let p = s.clone();
    s.pause();
    s.step(Input::default(), &[]);
    assert_eq!(s, p);
}
#[test]
fn save_and_restore_mid_jump_and_recovery_matches_uninterrupted_simulation() {
    let items = world::practice();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("freeski.json");
    for mut s in [running(), running()]
        .into_iter()
        .enumerate()
        .map(|(i, mut s)| {
            if i == 0 {
                s.position = Point { x: -12., y: 418. };
                s.distance = 418.;
                s.speed = 22.;
            } else {
                s.position = Point { x: -18., y: 100. };
                s.distance = 100.;
            }
            s
        })
    {
        s.step(Input::default(), &items);
        assert!(s.jump.is_some() || s.tumble > 0);
        let save = Save {
            run: s.clone(),
            ..Save::default()
        };
        storage::write(&path, &save, &items).unwrap();
        let mut restored = storage::load(&path, &items).unwrap().run;
        assert_eq!(restored.phase, Phase::Paused);
        restored.start();
        for tick in 0..400 {
            let input = Input {
                heading: if tick % 100 < 50 { 0.2 } else { -0.2 },
                brake: tick % 80 < 10,
            };
            s.step(input, &items);
            restored.step(input, &items);
            assert_eq!(s, restored);
        }
    }
}
#[test]
fn records_are_awarded_once_and_restart_keeps_preferences() {
    let mut save = Save::default();
    save.run.phase = Phase::Finished;
    save.run.position.y = world::FINISH;
    save.run.distance = world::FINISH;
    save.record_result();
    save.record_result();
    assert_eq!(save.completions, 1);
    assert_eq!(save.best_distance, world::FINISH);
    save.reduced_effects = true;
    save.restart();
    assert_eq!(save.completions, 1);
    assert!(save.reduced_effects);
    assert!(!save.result_recorded);
}
#[test]
fn corrupt_future_and_invalid_saves_are_retained_until_explicit_archive() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("freeski.json");
    let items = world::practice();
    let future = Save {
        version: 99,
        ..Save::default()
    };
    let invalid = Save {
        run: Sim {
            speed: 100.,
            ..Sim::default()
        },
        ..Save::default()
    };
    for bytes in [
        b"broken".to_vec(),
        serde_json::to_vec(&future).unwrap(),
        serde_json::to_vec(&invalid).unwrap(),
        vec![b' '; 65537],
    ] {
        std::fs::write(&path, &bytes).unwrap();
        assert!(storage::load(&path, &items).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        let archive = storage::archive(&path).unwrap();
        assert_eq!(std::fs::read(archive).unwrap(), bytes);
        assert!(!path.exists());
    }
    storage::write(&path, &Save::default(), &items).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(path).unwrap().permissions().mode() & 0o077,
            0
        );
    }
}
#[test]
fn practice_reference_run_finishes_using_only_production_controls() {
    let items = world::practice();
    let mut s = running();
    let mut jumps = 0;
    // Authored reference steering follows waypoints with speed-dependent look-ahead.
    let waypoints = [
        (0., 0.),
        (75., 0.),
        (160., 4.),
        (250., -2.),
        (340., -12.),
        (450., -12.),
        (520., 0.),
        (650., 5.),
        (725., 14.),
        (780., 14.),
        (860., 0.),
        (960., 0.),
        (1025., 0.),
        (1110., -4.),
        (1200., 0.),
    ];
    for _ in 0..12000 {
        let target_y = s.position.y + 18.;
        let pair = waypoints
            .windows(2)
            .find(|p| p[1].0 >= target_y)
            .unwrap_or(&waypoints[waypoints.len() - 2..]);
        let t = ((target_y - pair[0].0) / (pair[1].0 - pair[0].0)).clamp(0., 1.);
        let target_x = pair[0].1 + (pair[1].1 - pair[0].1) * t;
        let heading = (target_x - s.position.x).atan2(18.);
        let events = s.step(
            Input {
                heading,
                brake: false,
            },
            &items,
        );
        jumps += events.iter().filter(|e| **e == Event::Jump).count();
        assert!(s.valid(&items));
        if s.ended() {
            break;
        }
    }
    assert_eq!(s.phase, Phase::Finished, "{s:?}");
    assert_eq!(s.crashes, 0);
    assert!(jumps >= 2, "jumped {jumps} times");
    eprintln!(
        "Practice reference: {} ticks, {:.2}s, {jumps} jumps, {} crashes",
        s.ticks,
        s.ticks as f64 / 60.,
        s.crashes
    );
}

#[test]
fn speed_builds_gradually_and_quarter_turn_traverses_without_downhill_drift() {
    let mut s = running();
    for _ in 0..60 {
        s.step(Input::default(), &[]);
    }
    assert!((8.5..9.0).contains(&s.speed));
    for _ in 60..300 {
        s.step(Input::default(), &[]);
    }
    assert!((39.0..41.0).contains(&s.speed));
    for _ in 300..600 {
        s.step(Input::default(), &[]);
    }
    assert_eq!(s.speed, 60.);
    for side in [-1., 1.] {
        let mut turn = running();
        turn.speed = 10.;
        let input = Input {
            heading: side * MAX_HEADING,
            brake: false,
        };
        for _ in 0..60 {
            turn.step(input, &[]);
        }
        assert_eq!(turn.heading, side * std::f64::consts::FRAC_PI_2);
        let at = turn.position;
        for _ in 0..30 {
            turn.step(input, &[]);
        }
        assert!((turn.position.y - at.y).abs() < 1e-10);
        assert!((turn.position.x - at.x) * side > 1.);
        for _ in 0..60 {
            turn.step(Input::default(), &[]);
        }
        assert_eq!(turn.heading, 0.);
        assert!(turn.position.y > at.y);
    }
}

#[test]
fn rules_one_migration_retains_original_run_records_and_preferences() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("freeski.json");
    let items = world::practice();
    let mut old = Save {
        version: 3,
        generator_version: 1,
        rules_version: 1,
        best_distance: world::FINISH,
        completions: 3,
        reduced_effects: true,
        ..Save::default()
    };
    old.run = running();
    old.run.speed = 22.;
    old.run.heading = 1.35;
    old.run.jump = Some(0.4);
    let bytes = serde_json::to_vec(&old).unwrap();
    std::fs::write(&path, &bytes).unwrap();
    let migrated = storage::load(&path, &items).unwrap();
    old.rules_version = RULES_VERSION;
    old.run.pause();
    assert_eq!(migrated.run, old.run);
    assert_eq!(migrated.reduced_effects, old.reduced_effects);
    assert_eq!(migrated.legacy_records.as_ref().unwrap().completions, 3);
    assert_eq!(
        migrated.legacy_records.as_ref().unwrap().best_distance,
        world::FINISH
    );
    assert_eq!(migrated.completions, 0);
    assert_eq!(storage::load(&path, &items).unwrap(), migrated);
    assert!(!path.with_extension("rules-1-2.json").exists());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    storage::write(&path, &migrated, &items).unwrap();
    assert_eq!(
        std::fs::read(path.with_extension("rules-1-1.json")).unwrap(),
        bytes
    );
    assert_eq!(storage::load(&path, &items).unwrap(), migrated);
    old.rules_version = 1;
    old.run.speed = 30.;
    let invalid = serde_json::to_vec(&old).unwrap();
    std::fs::write(&path, &invalid).unwrap();
    assert!(storage::load(&path, &items).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), invalid);
}
