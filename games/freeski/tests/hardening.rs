use omarchy_freeski::{
    endless,
    engine::{Event, Input, Mode, Phase, Point, Sim, MAX_ENDLESS_DISTANCE, MAX_HEADING, MAX_SPEED},
    storage::{self, Save},
    world::{self, Kind, Obstacle},
};

fn running_at(y: f64) -> Sim {
    let mut sim = Sim {
        position: Point { x: 0., y },
        distance: y,
        ..Sim::default()
    };
    sim.start();
    sim
}

fn tick(seed: u64, sim: &mut Sim) {
    let obstacles = endless::obstacles(seed, sim.position.y);
    let input = Input {
        heading: endless::reference_heading(seed, sim),
        brake: false,
    };
    sim.step_mode(input, &obstacles, Mode::FreeSki);
}

#[test]
fn endless_replay_continues_exactly_after_a_cross_chunk_save() {
    let seed = 0x51_4b_49;
    let mut uninterrupted = Sim::default();
    uninterrupted.start();
    while uninterrupted.position.y < endless::CHUNK_LENGTH * 5.5 {
        tick(seed, &mut uninterrupted);
        assert!(uninterrupted.valid_mode(&[], Mode::FreeSki));
        assert_ne!(uninterrupted.phase, Phase::Finished);
    }

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("freeski.json");
    let checkpoint = Save {
        mode: Mode::FreeSki,
        seed,
        run: uninterrupted.clone(),
        ..Save::default()
    };
    storage::write(&path, &checkpoint, &world::practice()).unwrap();
    let mut restored = storage::load(&path, &world::practice()).unwrap().run;
    assert_eq!(restored.phase, Phase::Paused);
    restored.start();

    // Capture the exact production inputs against their fixed simulation ticks,
    // then prove the same schedule survives suspend/reopen byte-for-byte.
    let mut planner = uninterrupted.clone();
    let mut replay = Vec::with_capacity(2400);
    for _ in 0..2400 {
        let input = Input {
            heading: endless::reference_heading(seed, &planner),
            brake: false,
        };
        replay.push((planner.ticks + 1, input));
        let obstacles = endless::obstacles(seed, planner.position.y);
        planner.step_mode(input, &obstacles, Mode::FreeSki);
    }
    for (expected_tick, input) in replay {
        assert_eq!(uninterrupted.ticks + 1, expected_tick);
        assert_eq!(restored.ticks + 1, expected_tick);
        let obstacles = endless::obstacles(seed, uninterrupted.position.y);
        uninterrupted.step_mode(input, &obstacles, Mode::FreeSki);
        restored.step_mode(input, &obstacles, Mode::FreeSki);
        assert_eq!(restored, uninterrupted);
    }
    assert_eq!(uninterrupted, planner);
    assert!(uninterrupted.position.y > endless::CHUNK_LENGTH * 10.);
}

#[test]
fn endless_has_no_finish_and_recovers_safely_beyond_practice() {
    let tree = Obstacle {
        id: 7,
        at: Point { x: 0., y: 2_000. },
        kind: Kind::Tree,
    };
    let mut sim = running_at(1_999.5);
    sim.speed = MAX_SPEED;
    let events = sim.step_mode(Input::default(), &[tree], Mode::FreeSki);
    assert_eq!(events, vec![omarchy_freeski::engine::Event::Crash]);
    assert_eq!(sim.phase, Phase::Running);
    assert!(sim.position.y > world::FINISH);
    assert!(sim.position.y <= sim.distance);
    assert!(sim.valid_mode(&[], Mode::FreeSki));

    let mut skier = running_at(world::FINISH - 0.1);
    skier.speed = MAX_SPEED;
    assert!(skier
        .step_mode(Input::default(), &[], Mode::FreeSki)
        .is_empty());
    assert_eq!(skier.phase, Phase::Running);
    assert!(skier.position.y > world::FINISH);
}

#[test]
fn high_speed_turn_pause_and_mid_jump_resume_keep_exact_state() {
    let seed = 91;
    let mut turn = running_at(1_500.);
    turn.speed = MAX_SPEED;
    for _ in 0..60 {
        turn.step_mode(
            Input {
                heading: MAX_HEADING,
                brake: false,
            },
            &[],
            Mode::FreeSki,
        );
    }
    assert_eq!(turn.heading, MAX_HEADING);
    assert_eq!(turn.phase, Phase::Running);
    let paused = {
        turn.pause();
        turn.clone()
    };
    turn.step_mode(Input::default(), &[], Mode::FreeSki);
    assert_eq!(turn, paused);

    let ramp = (1..100)
        .flat_map(|chunk| endless::obstacles(seed, chunk as f64 * endless::CHUNK_LENGTH))
        .find(|obstacle| obstacle.kind == Kind::Ramp)
        .unwrap();
    let mut original = running_at(ramp.at.y - 1.);
    original.position.x = ramp.at.x;
    original.speed = MAX_SPEED;
    let obstacles = endless::obstacles(seed, original.position.y);
    assert_eq!(
        original.step_mode(Input::default(), &obstacles, Mode::FreeSki),
        vec![Event::Jump]
    );
    assert!(original.jump.is_some());

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("freeski.json");
    let save = Save {
        mode: Mode::FreeSki,
        seed,
        run: original.clone(),
        ..Save::default()
    };
    storage::write(&path, &save, &world::practice()).unwrap();
    let mut restored = storage::load(&path, &world::practice()).unwrap().run;
    assert_eq!(restored.phase, Phase::Paused);
    restored.start();
    for _ in 0..120 {
        let obstacles = endless::obstacles(seed, original.position.y);
        original.step_mode(Input::default(), &obstacles, Mode::FreeSki);
        restored.step_mode(Input::default(), &obstacles, Mode::FreeSki);
        assert_eq!(restored, original);
    }
}

#[test]
fn practice_and_endless_records_remain_independent_across_switches() {
    let mut save = Save::default();
    save.run.phase = Phase::Finished;
    save.run.position.y = world::FINISH;
    save.run.distance = world::FINISH;
    save.record_result();
    assert_eq!((save.best_distance, save.completions), (world::FINISH, 1));

    save.select_mode(Mode::FreeSki, 42);
    save.run = running_at(4_321.);
    assert_eq!(save.best(), 4_321.);
    save.restart();
    assert_eq!(save.free_best_distance, 4_321.);
    assert_eq!((save.best_distance, save.completions), (world::FINISH, 1));
    assert_eq!((save.mode, save.seed), (Mode::FreeSki, 42));

    save.run = running_at(5_000.);
    save.select_mode(Mode::Practice, 0);
    assert_eq!(save.free_best_distance, 5_000.);
    assert_eq!(save.best(), world::FINISH);
}

#[test]
fn schema_one_migration_is_lossless_and_retains_original_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("freeski.json");
    let mut value = serde_json::to_value(Save::default()).unwrap();
    let object = value.as_object_mut().unwrap();
    object.insert("version".into(), 1.into());
    object.insert("rules_version".into(), 2.into());
    for field in ["mode", "seed", "generator_version", "free_best_distance"] {
        object.remove(field);
    }
    object["best_distance"] = 300.into();
    object["reduced_effects"] = true.into();
    let bytes = serde_json::to_vec(&value).unwrap();
    std::fs::write(&path, &bytes).unwrap();

    let migrated = storage::load(&path, &world::practice()).unwrap();
    assert_eq!(migrated.version, 4);
    assert_eq!(migrated.mode, Mode::Practice);
    assert_eq!(
        migrated.legacy_records.as_ref().unwrap().best_distance,
        300.
    );
    assert_eq!(migrated.best_distance, 0.);
    assert!(migrated.reduced_effects);
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(
        std::fs::read(path.with_extension("schema-1-1.json")).unwrap(),
        bytes
    );
}

#[test]
fn schema_two_migration_preserves_active_endless_run_and_original_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("freeski.json");
    let mut save = Save::default();
    save.select_mode(Mode::FreeSki, 0x5152);
    save.run = running_at(2_345.);
    save.run.pause();
    save.free_best_distance = 1_999.;
    save.reduced_effects = true;
    let mut value = serde_json::to_value(&save).unwrap();
    let object = value.as_object_mut().unwrap();
    object.insert("version".into(), 2.into());
    object.insert("rules_version".into(), 2.into());
    object.insert("generator_version".into(), 1.into());
    for field in [
        "chase_enabled",
        "chase",
        "chase_best_distance",
        "course_index",
        "slalom",
        "slalom_best",
        "unlocked_courses",
        "muted",
    ] {
        object.remove(field);
    }
    let bytes = serde_json::to_vec(&value).unwrap();
    std::fs::write(&path, &bytes).unwrap();

    let migrated = storage::load(&path, &world::practice()).unwrap();
    assert_eq!(migrated.version, 4);
    assert_eq!(migrated.mode, Mode::FreeSki);
    assert_eq!(migrated.seed, 0x5152);
    assert_eq!(migrated.run.position.y, 2_345.);
    assert_eq!(
        migrated.legacy_records.as_ref().unwrap().free_best_distance,
        1_999.
    );
    assert_eq!(migrated.free_best_distance, 0.);
    assert_eq!(migrated.generator_version, 1);
    assert!(!migrated.chase_enabled);
    assert_eq!(migrated.unlocked_courses, 1);
    assert!(migrated.reduced_effects);
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(
        std::fs::read(path.with_extension("schema-2-1.json")).unwrap(),
        bytes
    );
}

#[test]
fn invalid_future_generator_and_extreme_position_are_retained_without_panicking() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("freeski.json");
    for save in [
        Save {
            mode: Mode::FreeSki,
            generator_version: endless::GENERATOR_VERSION + 1,
            ..Save::default()
        },
        Save {
            mode: Mode::FreeSki,
            run: Sim {
                position: Point { x: 0., y: 1e300 },
                distance: 1e300,
                ..Sim::default()
            },
            ..Save::default()
        },
        Save {
            mode: Mode::FreeSki,
            free_best_distance: MAX_ENDLESS_DISTANCE + 1.,
            ..Save::default()
        },
    ] {
        let bytes = serde_json::to_vec(&save).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        assert!(storage::load(&path, &world::practice()).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
}

#[test]
fn stale_valid_ramp_ids_survive_windows_but_forged_ids_do_not() {
    let seed = 23;
    let ramp = (1..100)
        .flat_map(|chunk| endless::obstacles(seed, chunk as f64 * endless::CHUNK_LENGTH))
        .find(|obstacle| obstacle.kind == Kind::Ramp)
        .unwrap();
    let mut save = Save {
        mode: Mode::FreeSki,
        seed,
        run: Sim {
            phase: Phase::Paused,
            position: Point {
                x: 0.,
                y: ramp.at.y + endless::CHUNK_LENGTH * 10.,
            },
            distance: ramp.at.y + endless::CHUNK_LENGTH * 10.,
            last_ramp: Some(ramp.id),
            ..Sim::default()
        },
        ..Save::default()
    };
    assert!(save.valid(&world::practice()));
    save.run.last_ramp = Some(ramp.id + 2);
    assert!(!save.valid(&world::practice()));
}

#[test]
fn slalom_validation_uses_its_canonical_course_not_the_callers_window() {
    let practice = world::practice();
    let practice_ramp = practice
        .iter()
        .find(|obstacle| obstacle.kind == Kind::Ramp)
        .unwrap();
    let mut save = Save::default();
    assert!(save.select_course(0));
    save.run.phase = Phase::Paused;
    save.run.last_ramp = Some(practice_ramp.id);
    assert!(!save.valid(&practice));
}
