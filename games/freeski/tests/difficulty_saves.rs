use omarchy_freeski::{
    endless,
    engine::{Input, Mode, Phase},
    session::Session,
    storage::{self, Save},
    world,
};

#[test]
fn old_endless_attempt_keeps_terrain_and_banks_only_into_retained_records() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("freeski.json");
    let mut old = Save::default();
    old.select_mode(Mode::FreeSki, 17);
    old.version = 3;
    old.rules_version = 2;
    old.generator_version = 1;
    old.free_best_distance = 1234.;
    old.run.start();
    old.run.position.y = 1400.;
    old.run.distance = 1400.;
    old.run.speed = 50.;
    let original = serde_json::to_vec(&old).unwrap();
    std::fs::write(&path, &original).unwrap();
    let mut restored = storage::load(&path, &world::practice()).unwrap();
    assert_eq!(restored.version, 4);
    assert!(restored.legacy_run);
    assert_eq!(restored.run.phase, Phase::Paused);
    assert_eq!(restored.run.speed, 50.);
    assert_eq!(restored.generator_version, 1);
    assert_eq!(
        restored.obstacles(),
        endless::obstacles_versioned(17, 1400., 1)
    );
    assert_eq!(restored.free_best_distance, 0.);
    assert_eq!(restored.best(), 1400.);
    assert_eq!(
        std::fs::read(path.with_extension("schema-3-1.json")).unwrap(),
        original
    );
    storage::write(&path, &restored, &world::practice()).unwrap();
    assert_eq!(storage::load(&path, &world::practice()).unwrap(), restored);
    restored.restart();
    assert!(!restored.legacy_run);
    assert_eq!(restored.generator_version, endless::GENERATOR_VERSION);
    assert_eq!(
        restored.legacy_records.as_ref().unwrap().free_best_distance,
        1400.
    );
    assert_eq!(restored.free_best_distance, 0.);
    assert!(restored.valid(&world::practice()));
}

#[test]
fn old_slalom_unlocks_survive_without_reusing_old_medal_times() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("freeski.json");
    let old = Save {
        version: 3,
        rules_version: 2,
        generator_version: 1,
        slalom_best: [Some(1800), Some(2100), None, None, None],
        unlocked_courses: 3,
        ..Save::default()
    };
    std::fs::write(&path, serde_json::to_vec(&old).unwrap()).unwrap();
    let mut restored = storage::load(&path, &world::practice()).unwrap();
    assert_eq!(restored.unlocked_courses, 3);
    assert_eq!(restored.slalom_best, [None; 5]);
    assert_eq!(
        restored.legacy_records.as_ref().unwrap().slalom_best,
        old.slalom_best
    );
    assert!(restored.select_course(2));
    assert!(!restored.legacy_run);
    assert!(restored.valid(&world::practice()));
    assert!(!restored.select_course(3));
}

#[test]
fn finishing_a_resumed_legacy_course_unlocks_without_awarding_current_medals() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("freeski.json");
    let old = Save {
        version: 3,
        rules_version: 2,
        generator_version: 1,
        mode: Mode::Slalom,
        ..Save::default()
    };
    std::fs::write(&path, serde_json::to_vec(&old).unwrap()).unwrap();
    let mut session = Session::new(storage::load(&path, &world::practice()).unwrap());
    session.state.run.start();
    for _ in 0..3000 {
        let input = omarchy_freeski::course::reference_input(0, &session.state.run);
        session.step(input);
        if session.state.run.ended() {
            break;
        }
    }
    assert_eq!(session.state.run.phase, Phase::Finished);
    assert_eq!(session.state.unlocked_courses, 2);
    assert!(session.state.legacy_records.as_ref().unwrap().slalom_best[0].is_some());
    assert_eq!(session.state.slalom_best, [None; 5]);
    assert_eq!(session.state.slalom_medal(), None);
    assert!(session.state.valid(&session.obstacles));
    let recorded = session.state.clone();
    session.state.record_result();
    assert_eq!(session.state, recorded);
}

#[test]
fn fast_mode_and_causal_state_survive_save_and_exact_continuation() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("freeski.json");
    let mut session = Session::new(Save::default());
    session.state.run.start();
    session.state.run.toggle_fast_mode();
    for _ in 0..180 {
        session.step(Input::default());
    }
    assert!(session.state.run.fast_mode);
    session.state.run.pause();
    storage::write(&path, &session.state, &session.obstacles).unwrap();
    let mut restored = Session::new(storage::load(&path, &world::practice()).unwrap());
    assert_eq!(restored.state, session.state);
    session.state.run.start();
    restored.state.run.start();
    for _ in 0..120 {
        assert_eq!(
            session.step(Input::default()),
            restored.step(Input::default())
        );
        assert_eq!(session.state, restored.state);
        assert!(restored.state.valid(&restored.obstacles));
    }
}

#[test]
fn old_schema_cannot_smuggle_new_fast_speed_or_replace_terrain() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("freeski.json");
    for case in 0..5 {
        let mut invalid = Save {
            version: 3,
            rules_version: 2,
            generator_version: 1,
            ..Save::default()
        };
        invalid.run.start();
        match case {
            0 => invalid.run.speed = 80.,
            1 => invalid.run.fast_mode = true,
            2 => invalid.generator_version = endless::GENERATOR_VERSION,
            3 => invalid.rules_version = 3,
            _ => invalid.chase.speed = 80.,
        }
        let bytes = serde_json::to_vec(&invalid).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        assert!(storage::load(&path, &world::practice()).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert!(!path.with_extension("schema-3-1.json").exists());
    }
}
